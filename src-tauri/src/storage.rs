use crate::domain::*;
use image::{DynamicImage, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Cursor, Write},
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub last_project_id: Option<String>,
}
pub struct Store {
    pub root: PathBuf,
    pub projects: HashMap<String, Project>,
    pub deleted_projects: HashMap<String, Project>,
    pub preferences: Preferences,
    pub warnings: Vec<String>,
}

impl Store {
    pub fn open(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(root.join("projects")).map_err(io_error)?;
        let preferences = fs::read(root.join("preferences.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let mut store = Self {
            root,
            projects: HashMap::new(),
            deleted_projects: HashMap::new(),
            preferences,
            warnings: vec![],
        };
        for entry in fs::read_dir(store.root.join("projects"))
            .map_err(io_error)?
            .flatten()
        {
            if !entry.file_type().map_err(io_error)?.is_dir() {
                continue;
            }
            let manifest = entry.path().join("project.json");
            if !manifest.exists() {
                continue;
            }
            match read_project(&manifest) {
                Ok(mut p) => {
                    if p.id != entry.file_name().to_string_lossy() { store.warnings.push("A project folder does not match its ID. It was left untouched.".into()); continue; }
                    if entry.path().join(".deleted").exists() {
                        store.deleted_projects.insert(p.id.clone(), p);
                        continue;
                    }
                    if p.recover() { store.save(&p)?; }
                    store.projects.insert(p.id.clone(), p);
                }
                Err(e) => store.warnings.push(format!("Could not open project {}: {e} Files were left untouched. A previous manifest is available as project.backup.json.", entry.file_name().to_string_lossy())),
            }
        }
        if !store
            .preferences
            .last_project_id
            .as_ref()
            .is_some_and(|id| store.projects.contains_key(id))
        {
            store.preferences.last_project_id = None;
        }
        Ok(store)
    }
    pub fn project(&self, id: &str) -> Result<&Project> {
        self.projects
            .get(id)
            .ok_or("Project no longer exists.".into())
    }
    pub fn dir(&self, id: &str) -> Result<PathBuf> {
        uuid::Uuid::parse_str(id).map_err(|_| "Invalid project ID.")?;
        Ok(self.root.join("projects").join(id))
    }
    pub fn save(&self, project: &Project) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(project).map_err(|_| "Could not encode project.")?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err("This project's history has reached 64 MB. Start a new project before adding more work.".into());
        }
        let dir = self.dir(&project.id)?;
        fs::create_dir_all(&dir).map_err(io_error)?;
        let path = dir.join("project.json");
        if path.exists() {
            let previous = fs::read(&path).map_err(io_error)?;
            atomic_write(&dir.join("project.backup.json"), &previous)?;
        }
        atomic_write(&path, &bytes)
    }
    // ASVS 15.4.1: callers hold the store mutex; replace memory only after durable writes.
    pub fn put(&mut self, mut project: Project) -> Result<Project> {
        project.updated_at = now();
        project.revision += 1;
        self.save(&project)?;
        self.projects.insert(project.id.clone(), project.clone());
        Ok(project)
    }
    pub fn select(&mut self, id: &str) -> Result<Project> {
        let project = self.project(id)?.clone();
        let preferences = Preferences {
            last_project_id: Some(id.into()),
        };
        atomic_write(
            &self.root.join("preferences.json"),
            &serde_json::to_vec(&preferences).map_err(|_| "Could not save settings.")?,
        )?;
        self.preferences = preferences;
        Ok(project)
    }
    // ASVS 5.3.2, 15.4.1: only known UUIDs under the store lock can be deleted.
    // A durable marker keeps images recoverable and prevents an interrupted worker
    // from bringing a deleted collection back when the application reopens.
    pub fn delete(&mut self, id: &str) -> Result<()> {
        let project = self.project(id)?.clone();
        let dir = self.checked_project_dir(id)?;
        if self.preferences.last_project_id.as_deref() == Some(id) {
            atomic_write(
                &self.root.join("preferences.json"),
                &serde_json::to_vec(&Preferences::default())
                    .map_err(|_| "Could not save settings.")?,
            )?;
        }
        atomic_write(
            &dir.join(".deleted"),
            b"Deleted collection; restore through Spiraler.",
        )?;
        self.projects.remove(id);
        self.deleted_projects.insert(id.into(), project);
        if self.preferences.last_project_id.as_deref() == Some(id) {
            self.preferences.last_project_id = None;
        }
        Ok(())
    }
    pub fn restore(&mut self, id: &str) -> Result<()> {
        if !self.deleted_projects.contains_key(id) {
            return Err("Deleted collection no longer exists.".into());
        }
        let dir = self.checked_project_dir(id)?;
        let mut project = read_project(&dir.join("project.json"))?;
        project.paused = true;
        for job in &mut project.jobs {
            if matches!(job.status, JobStatus::Queued | JobStatus::Generating) {
                job.status = JobStatus::Cancelled;
                job.finished_at = Some(now());
                job.error =
                    Some("Cancelled when the collection was deleted. Retry when ready.".into());
            }
        }
        self.save(&project)?;
        fs::remove_file(dir.join(".deleted")).map_err(io_error)?;
        self.deleted_projects.remove(id);
        self.projects.insert(id.into(), project);
        Ok(())
    }
    fn checked_project_dir(&self, id: &str) -> Result<PathBuf> {
        let path = self.dir(id)?;
        let root = self
            .root
            .join("projects")
            .canonicalize()
            .map_err(io_error)?;
        let canonical = path.canonicalize().map_err(io_error)?;
        if canonical != root.join(id)
            || fs::symlink_metadata(&path)
                .map_err(io_error)?
                .file_type()
                .is_symlink()
        {
            return Err("Collection folder is outside the library.".into());
        }
        Ok(canonical)
    }
    pub fn image_path(&self, project_id: &str, relative: &str) -> Result<PathBuf> {
        confined(&self.dir(project_id)?, relative)
    }
}

pub fn read_project(path: &Path) -> Result<Project> {
    if fs::metadata(path).map_err(io_error)?.len() > 64 * 1024 * 1024 {
        return Err("Project manifest is too large.".into());
    }
    let bytes = fs::read(path).map_err(io_error)?;
    let schema: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "The project manifest is damaged.")?;
    // Explicit migration gate: never rewrite a newer schema with an older application.
    if schema.get("schemaVersion").and_then(|v| v.as_u64()) != Some(1) {
        return Err("This project needs a newer version of Spiraler.".into());
    }
    let project: Project =
        serde_json::from_slice(&bytes).map_err(|_| "The project structure is invalid.")?;
    uuid::Uuid::parse_str(&project.id).map_err(|_| "Invalid project identity.")?;
    project.style.validate()?;
    validate_project(&project)?;
    Ok(project)
}

fn validate_project(project: &Project) -> Result<()> {
    required(&project.name, 120)?;
    required(&project.asset_type, 80)?;
    if project.assets.len() > 2000 {
        return Err("The project has too many assets.".into());
    }
    let valid_id = |value: &str| {
        uuid::Uuid::parse_str(value)
            .map(|_| ())
            .map_err(|_| "The project contains an invalid ID.".to_string())
    };
    let mut asset_ids = std::collections::HashSet::new();
    for asset in &project.assets {
        valid_id(&asset.id)?;
        if !asset_ids.insert(&asset.id) {
            return Err("The project contains duplicate asset IDs.".into());
        }
        required(&asset.name, 160)?;
        required(&asset.subject, 8000)?;
        for result in &asset.results {
            valid_id(&result.id)?;
            valid_id(&result.image.id)?;
        }
    }
    for job in &project.jobs {
        valid_id(&job.id)?;
        valid_id(&job.asset_id)?;
        job.request.style.validate()?;
    }
    for reference in &project.style.references {
        valid_id(&reference.id)?;
    }
    Ok(())
}

pub fn io_error(error: std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied => {
            "Spiraler cannot access this folder. Choose a writable location and try again.".into()
        }
        std::io::ErrorKind::NotFound => {
            "The file is missing. Restore it or import the image again.".into()
        }
        std::io::ErrorKind::StorageFull => {
            "There is not enough disk space. Free some space and retry.".into()
        }
        _ => "The file could not be saved or read. Check disk space and folder access, then retry."
            .into(),
    }
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("Invalid destination.")?;
    fs::create_dir_all(parent).map_err(io_error)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(io_error)?;
    tmp.write_all(bytes).map_err(io_error)?;
    tmp.as_file().sync_all().map_err(io_error)?;
    tmp.persist(path).map_err(|e| io_error(e.error))?;
    #[cfg(unix)]
    File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(io_error)?;
    Ok(())
}
// ASVS 5.3.2: generated IDs name files; reject traversal and symlink escapes on reads.
pub fn confined(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty() || !path.components().all(|c| matches!(c, Component::Normal(_)))
    {
        return Err("Invalid project file path.".into());
    }
    let canonical_root = root.canonicalize().map_err(io_error)?;
    let file = root.join(path).canonicalize().map_err(io_error)?;
    if !file.starts_with(&canonical_root) {
        return Err("File is outside this project.".into());
    }
    Ok(file)
}
pub fn load_image(path: &Path) -> Result<DynamicImage> {
    // ASVS 5.2.1, 5.2.2, 5.2.6: bounded content sniffing and bounded decoded dimensions.
    if fs::metadata(path).map_err(io_error)?.len() > 50 * 1024 * 1024 {
        return Err("Images must be smaller than 50 MB.".into());
    }
    let mut reader = ImageReader::open(path)
        .map_err(io_error)?
        .with_guessed_format()
        .map_err(io_error)?;
    if !matches!(
        reader.format(),
        Some(ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP)
    ) {
        return Err("Choose a PNG, JPEG, or WebP image.".into());
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(300 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().map_err(|_| {
        "This image is damaged or too large. Choose an image up to 8,192 pixels per side.".into()
    })
}
pub fn save_image(dir: &Path, image: &DynamicImage, name: &str) -> Result<ImageFile> {
    let image_id = id();
    let file = format!("images/{image_id}.png");
    let thumbnail = format!("thumbnails/{image_id}.png");
    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|_| "Could not encode image.")?;
    atomic_write(&dir.join(&file), buffer.get_ref())?;
    let mut thumb = Cursor::new(Vec::new());
    image
        .thumbnail(480, 480)
        .write_to(&mut thumb, ImageFormat::Png)
        .map_err(|_| "Could not create thumbnail.")?;
    atomic_write(&dir.join(&thumbnail), thumb.get_ref())?;
    Ok(ImageFile {
        id: image_id,
        name: name.chars().take(160).collect(),
        file,
        thumbnail,
        width: image.width(),
        height: image.height(),
    })
}
pub fn safe_name(name: &str) -> String {
    let s = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let s = s
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let s: String = s.chars().take(80).collect();
    if s.is_empty() {
        "asset".into()
    } else {
        s
    }
}
