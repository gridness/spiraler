pub mod domain;
pub mod export;
pub mod provider;
mod queue;
pub mod storage;

use domain::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use storage::{load_image, save_image, Store};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

pub struct AppState {
    pub store: Mutex<Store>,
    pub active: Mutex<HashMap<String, Arc<AtomicBool>>>,
    pub signing_in: AtomicBool,
    pub shutting_down: AtomicBool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSummary {
    id: String,
    name: String,
    asset_type: String,
    count: usize,
    updated_at: u64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Library {
    projects: Vec<ProjectSummary>,
    deleted_projects: Vec<ProjectSummary>,
    last_project_id: Option<String>,
    root: String,
    warnings: Vec<String>,
}
fn lock_store(state: &AppState) -> Result<std::sync::MutexGuard<'_, Store>> {
    state
        .store
        .lock()
        .map_err(|_| "Project storage is unavailable. Restart Spiraler.".into())
}

#[tauri::command]
fn library(state: tauri::State<AppState>) -> Result<Library> {
    let store = lock_store(&state)?;
    let mut projects: Vec<_> = store
        .projects
        .values()
        .map(|p| ProjectSummary {
            id: p.id.clone(),
            name: p.name.clone(),
            asset_type: p.asset_type.clone(),
            count: p.assets.len(),
            updated_at: p.updated_at,
        })
        .collect();
    projects.sort_by_key(|p| std::cmp::Reverse(p.updated_at));
    let mut deleted_projects: Vec<_> = store
        .deleted_projects
        .values()
        .map(|p| ProjectSummary {
            id: p.id.clone(),
            name: p.name.clone(),
            asset_type: p.asset_type.clone(),
            count: p.assets.len(),
            updated_at: p.updated_at,
        })
        .collect();
    deleted_projects.sort_by_key(|p| std::cmp::Reverse(p.updated_at));
    Ok(Library {
        projects,
        deleted_projects,
        last_project_id: store.preferences.last_project_id.clone(),
        root: store.root.to_string_lossy().into(),
        warnings: store.warnings.clone(),
    })
}
#[tauri::command]
async fn create_project(
    app: tauri::AppHandle,
    name: String,
    description: String,
    asset_type: String,
) -> Result<Project> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut store = lock_store(&state)?;
        let project = Project::new(name, description, asset_type)?;
        let project = store.put(project)?;
        store.select(&project.id)
    })
    .await
    .map_err(|_| "Could not create project.")?
}
#[tauri::command]
async fn open_project(app: tauri::AppHandle, project_id: String) -> Result<Project> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let result = lock_store(&state)?.select(&project_id);
        result
    })
    .await
    .map_err(|_| "Could not open project.")?
}
#[tauri::command]
async fn delete_project(app: tauri::AppHandle, project_id: String) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut store = lock_store(&state)?;
        let job_ids: Vec<_> = store
            .project(&project_id)?
            .jobs
            .iter()
            .map(|job| job.id.clone())
            .collect();
        store.delete(&project_id)?;
        if let Ok(active) = state.active.lock() {
            for id in job_ids {
                if let Some(flag) = active.get(&id) {
                    flag.store(true, Ordering::SeqCst);
                }
            }
        }
        Ok(())
    })
    .await
    .map_err(|_| "Could not delete the collection.")?
}
#[tauri::command]
async fn restore_project(app: tauri::AppHandle, project_id: String) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let result = lock_store(&state)?.restore(&project_id);
        result
    })
    .await
    .map_err(|_| "Could not restore the collection.")?
}
#[tauri::command]
async fn mutate(app: tauri::AppHandle, project_id: String, action: Action) -> Result<Project> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let mut store = lock_store(&state)?;
        let mut project = store.project(&project_id)?.clone();
        project.apply(action.clone())?;
        let project = store.put(project)?;
        if let Action::Cancel { ids } = action {
            if let Ok(active) = state.active.lock() {
                for id in ids {
                    if let Some(flag) = active.get(&id) {
                        flag.store(true, Ordering::SeqCst);
                    }
                }
            }
        }
        app.emit("project-changed", &project)
            .map_err(|_| "Could not refresh the window.")?;
        Ok(project)
    })
    .await
    .map_err(|_| "The change could not be saved. Try again.")?
}
#[tauri::command]
async fn provider_status() -> provider::ProviderStatus {
    provider::status().await
}
#[tauri::command]
async fn connect_provider(app: tauri::AppHandle) -> Result<provider::ProviderStatus> {
    let state = app.state::<AppState>();
    if state.signing_in.swap(true, Ordering::SeqCst) {
        return Err("A sign-in window is already open. Complete it in your browser.".into());
    }
    let result = provider::login().await;
    state.signing_in.store(false, Ordering::SeqCst);
    result
}
#[tauri::command]
async fn import_images(
    app: tauri::AppHandle,
    project_id: String,
    asset_id: Option<String>,
    paths: Option<Vec<String>>,
) -> Result<Option<Project>> {
    tauri::async_runtime::spawn_blocking(move || {
        let paths = if let Some(paths) = paths {
            paths.into_iter().map(PathBuf::from).collect()
        } else {
            let Some(files) = app
                .dialog()
                .file()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                .set_title(if asset_id.is_some() {
                    "Import results"
                } else {
                    "Add style references"
                })
                .blocking_pick_files()
            else {
                return Ok(None);
            };
            files
                .into_iter()
                .map(|f| {
                    f.into_path()
                        .map_err(|_| "Choose a local image.".to_string())
                })
                .collect::<Result<Vec<_>>>()?
        };
        if paths.is_empty() || paths.len() > 20 {
            return Err("Import between 1 and 20 images at a time.".into());
        }
        let state = app.state::<AppState>();
        let (dir, remaining) = {
            let store = lock_store(&state)?;
            (
                store.dir(&project_id)?,
                5usize.saturating_sub(store.project(&project_id)?.style.references.len()),
            )
        };
        if asset_id.is_none() && paths.len() > remaining {
            return Err("A style can have up to five references.".into());
        }
        let mut images = Vec::new();
        for path in paths {
            let image = load_image(&path)?;
            images.push(save_image(
                &dir,
                &image,
                &path.file_stem().unwrap_or_default().to_string_lossy(),
            )?);
        }
        let mut store = lock_store(&state)?;
        let mut project = store.project(&project_id)?.clone();
        for image in images {
            if let Some(asset_id) = &asset_id {
                let subject = project.asset_mut(asset_id)?.subject.clone();
                let request =
                    GenerationRequest::build(&project.style, &subject, &project.asset_type, None);
                let result = Generation {
                    id: id(),
                    image,
                    kind: JobKind::Generate,
                    created_at: now(),
                    request,
                    provider: "Imported image; generation provenance unknown".into(),
                    source_result_id: None,
                };
                let asset = project.asset_mut(asset_id)?;
                asset.preferred_id = Some(result.id.clone());
                asset.approved = false;
                asset.results.push(result);
            } else {
                project.add_reference(image)?;
            }
        }
        let project = store.put(project)?;
        let _ = app.emit("project-changed", &project);
        Ok(Some(project))
    })
    .await
    .map_err(|_| "Image import stopped. Try a smaller file.")?
}
#[tauri::command]
async fn import_asset_list(app: tauri::AppHandle) -> Result<Option<String>> {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(file) = app
            .dialog()
            .file()
            .add_filter("Asset lists", &["txt", "csv", "tsv"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = file.into_path().map_err(|_| "Choose a local file.")?;
        if std::fs::metadata(&path).map_err(storage::io_error)?.len() > 1024 * 1024 {
            return Err("Asset lists must be smaller than 1 MB.".into());
        }
        Ok(Some(
            std::fs::read_to_string(path).map_err(|_| "Choose a UTF-8 text, CSV, or TSV file.")?,
        ))
    })
    .await
    .map_err(|_| "Could not import the list.")?
}
#[tauri::command]
async fn export_assets(
    app: tauri::AppHandle,
    project_id: String,
    options: export::ExportOptions,
) -> Result<Option<export::ExportReport>> {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(folder) = app
            .dialog()
            .file()
            .set_title("Export assets to folder")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let destination = folder.into_path().map_err(|_| "Choose a local folder.")?;
        let state = app.state::<AppState>();
        let (project, dir) = {
            let store = lock_store(&state)?;
            (store.project(&project_id)?.clone(), store.dir(&project_id)?)
        };
        export::export(&project, &dir, &destination, options).map(Some)
    })
    .await
    .map_err(|_| "Export stopped. Your originals are safe; try again.")?
}
#[tauri::command]
fn reveal(app: tauri::AppHandle, project_id: String, image_id: Option<String>) -> Result<()> {
    let state = app.state::<AppState>();
    let store = lock_store(&state)?;
    let project = store.project(&project_id)?;
    let path = if let Some(image_id) = image_id {
        let image = project
            .assets
            .iter()
            .flat_map(|a| a.results.iter().map(|r| &r.image))
            .chain(project.style.references.iter())
            .find(|i| i.id == image_id)
            .ok_or("Image no longer exists.")?;
        store.image_path(&project_id, &image.file)?
    } else {
        store.dir(&project_id)?
    };
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|_| "Could not reveal the file in your file manager.".into())
}
#[tauri::command]
fn open_help(app: tauri::AppHandle) -> Result<()> {
    app.opener()
        .open_url("https://learn.chatgpt.com/docs/cli", None::<&str>)
        .map_err(|_| "Could not open the Codex installation guide.".into())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            let store = Store::open(root).map_err(std::io::Error::other)?;
            app.manage(AppState {
                store: Mutex::new(store),
                active: Mutex::new(HashMap::new()),
                signing_in: AtomicBool::new(false),
                shutting_down: AtomicBool::new(false),
            });
            queue::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            library,
            create_project,
            open_project,
            delete_project,
            restore_project,
            mutate,
            provider_status,
            connect_provider,
            import_images,
            import_asset_list,
            export_assets,
            reveal,
            open_help
        ])
        .build(tauri::generate_context!())
        .expect("Unable to launch Spiraler")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                let state = app.state::<AppState>();
                if !state.shutting_down.swap(true, Ordering::SeqCst) {
                    api.prevent_exit();
                    if let Ok(active) = state.active.lock() {
                        for flag in active.values() {
                            flag.store(true, Ordering::SeqCst);
                        }
                    }
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        for _ in 0..40 {
                            let idle = app
                                .state::<AppState>()
                                .active
                                .lock()
                                .map(|a| a.is_empty())
                                .unwrap_or(true);
                            if idle {
                                break;
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                        }
                        app.exit(0);
                    });
                }
            }
        });
}
