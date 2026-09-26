use spiraler_lib::{
    domain::*,
    export::{export, ExportOptions, ExportVersion},
    provider::classify_error,
    storage::*,
};
use std::fs;

fn project_with_assets() -> Project {
    let mut p = Project::new(
        "Forest props".into(),
        "Small objects".into(),
        "game asset".into(),
    )
    .unwrap();
    p.apply(Action::AddAssets {
        items: vec![
            AssetInput {
                name: "Potion".into(),
                subject: "red glass".into(),
            },
            AssetInput {
                name: "Potion".into(),
                subject: "blue glass".into(),
            },
        ],
    })
    .unwrap();
    p
}

#[test]
fn style_history_and_queued_requests_are_immutable() {
    let mut p = project_with_assets();
    p.apply(Action::Enqueue {
        ids: p.assets.iter().map(|a| a.id.clone()).collect(),
        kind: JobKind::Generate,
    })
    .unwrap();
    let snapshot = serde_json::to_string(&p.jobs[0].request).unwrap();
    let mut changed = p.style.clone();
    changed.direction = "Cut paper and soft shadows".into();
    p.apply(Action::SaveStyle {
        style: changed,
        expected_version: 1,
    })
    .unwrap();
    assert_eq!(p.style.version, 2);
    assert_eq!(p.style_history.len(), 1);
    assert_eq!(serde_json::to_string(&p.jobs[0].request).unwrap(), snapshot);
    p.apply(Action::EditAsset {
        asset_id: p.assets[0].id.clone(),
        name: "Changed".into(),
        subject: "green glass".into(),
    })
    .unwrap();
    assert_eq!(p.jobs[0].request.subject, "red glass");
}
#[test]
fn persistent_restart_marks_interrupted_work_and_pauses_remaining_jobs() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path().into()).unwrap();
    let mut p = project_with_assets();
    p.apply(Action::Enqueue {
        ids: p.assets.iter().map(|a| a.id.clone()).collect(),
        kind: JobKind::Generate,
    })
    .unwrap();
    p.jobs[0].status = JobStatus::Generating;
    p.jobs[0].attempts = 1;
    store.put(p.clone()).unwrap();
    store.select(&p.id).unwrap();
    drop(store);
    let recovered = Store::open(tmp.path().into()).unwrap();
    let saved = recovered.project(&p.id).unwrap();
    assert_eq!(saved.jobs[0].status, JobStatus::Failed);
    assert!(saved.jobs[0].error.as_ref().unwrap().contains("closed"));
    assert_eq!(saved.jobs[1].status, JobStatus::Queued);
    assert!(saved.paused);
    assert_eq!(recovered.preferences.last_project_id, Some(p.id));
}
#[test]
fn cancel_retry_and_duplicate_enqueue_do_not_destroy_other_jobs() {
    let mut p = project_with_assets();
    let ids = p.assets.iter().map(|a| a.id.clone()).collect::<Vec<_>>();
    p.apply(Action::Enqueue {
        ids: ids.clone(),
        kind: JobKind::Generate,
    })
    .unwrap();
    p.apply(Action::Enqueue {
        ids,
        kind: JobKind::Generate,
    })
    .unwrap();
    assert_eq!(p.jobs.len(), 2);
    p.apply(Action::Cancel {
        ids: vec![p.jobs[0].id.clone()],
    })
    .unwrap();
    assert_eq!(p.jobs[1].status, JobStatus::Queued);
    p.apply(Action::Retry {
        ids: vec![p.jobs[0].id.clone()],
    })
    .unwrap();
    assert_eq!(p.jobs[0].status, JobStatus::Queued);
}
#[test]
fn atomic_save_keeps_a_previous_manifest_and_failed_edits_leave_disk_intact() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path().into()).unwrap();
    let p = project_with_assets();
    store.put(p.clone()).unwrap();
    let mut p2 = p.clone();
    p2.name = "New name".into();
    store.put(p2).unwrap();
    let backup = read_project(&store.dir(&p.id).unwrap().join("project.backup.json")).unwrap();
    assert_eq!(backup.name, "Forest props");
    let mut invalid = store.project(&p.id).unwrap().clone();
    assert!(invalid
        .apply(Action::Reorder {
            ids: vec!["../escape".into()]
        })
        .is_err());
    assert_eq!(
        read_project(&store.dir(&p.id).unwrap().join("project.json"))
            .unwrap()
            .name,
        "New name"
    );
}
#[test]
fn corrupt_and_future_manifests_are_left_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path().into()).unwrap();
    let p = project_with_assets();
    store.put(p.clone()).unwrap();
    let path = store.dir(&p.id).unwrap().join("project.json");
    fs::write(&path, b"{incomplete").unwrap();
    let recovered = Store::open(tmp.path().into()).unwrap();
    assert_eq!(recovered.warnings.len(), 1);
    assert!(recovered.projects.is_empty());
    assert_eq!(fs::read(&path).unwrap(), b"{incomplete");
    fs::write(&path, b"{\"schemaVersion\":99}").unwrap();
    assert!(read_project(&path).unwrap_err().contains("newer"));
}
#[test]
fn references_are_real_files_and_prompts_assign_roles_in_order() {
    let mut p = project_with_assets();
    let image = ImageFile {
        id: id(),
        name: "Clay".into(),
        file: "images/clay.png".into(),
        thumbnail: "thumbnails/clay.png".into(),
        width: 1024,
        height: 1024,
    };
    p.add_reference(image.clone()).unwrap();
    let request = GenerationRequest::build(&p.style, "red potion", "prop", Some(image));
    assert!(request.prompt.contains("Image 1 is the source"));
    assert!(request.prompt.contains("Image 2 is a STYLE"));
    assert_eq!(
        request.prompt,
        GenerationRequest::build(&p.style, "red potion", "prop", request.source.clone()).prompt
    );
}
#[test]
fn image_validation_export_and_collision_handling_preserve_originals() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path().into()).unwrap();
    let mut p = project_with_assets();
    store.put(p.clone()).unwrap();
    let dir = store.dir(&p.id).unwrap();
    let image = image::DynamicImage::new_rgba8(64, 96);
    for a in &mut p.assets {
        let file = save_image(&dir, &image, &a.name).unwrap();
        let result = Generation {
            id: id(),
            image: file,
            kind: JobKind::Generate,
            created_at: now(),
            request: GenerationRequest::build(&p.style, &a.subject, &p.asset_type, None),
            provider: "test fixture".into(),
            source_result_id: None,
        };
        a.preferred_id = Some(result.id.clone());
        a.results.push(result);
    }
    let destination = tmp.path().join("exports");
    fs::create_dir(&destination).unwrap();
    let options = ExportOptions {
        ids: p.assets.iter().map(|a| a.id.clone()).collect(),
        version: ExportVersion::Preferred,
        metadata: true,
        prefix: "../unsafe/name".into(),
    };
    let one = export(&p, &dir, &destination, options.clone()).unwrap();
    let two = export(&p, &dir, &destination, options).unwrap();
    assert_eq!(one.count, 2);
    assert_ne!(one.directory, two.directory);
    assert_eq!(fs::read_dir(one.directory).unwrap().count(), 4);
    for a in &p.assets {
        let path = confined(&dir, &a.results[0].image.file).unwrap();
        assert_eq!(load_image(&path).unwrap().width(), 64);
    }
    let bad = tmp.path().join("bad.png");
    fs::write(&bad, b"not an image").unwrap();
    assert!(load_image(&bad).is_err());
}
#[test]
fn path_traversal_and_symlinks_cannot_escape_the_project() {
    let tmp = tempfile::tempdir().unwrap();
    let project = tmp.path().join("project");
    fs::create_dir(&project).unwrap();
    fs::write(tmp.path().join("outside.png"), b"secret").unwrap();
    assert!(confined(&project, "../outside.png").is_err());
    assert!(confined(&project, "/etc/passwd").is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(tmp.path().join("outside.png"), project.join("link.png"))
            .unwrap();
        assert!(confined(&project, "link.png").is_err());
    }
    assert_eq!(safe_name("../../CON / My Image"), "con-my-image");
}
#[test]
fn errors_do_not_leak_raw_output_and_rate_limits_do_not_auto_retry() {
    let error = classify_error("401 unauthorized token=very-secret-value");
    assert!(!error.message.contains("very-secret"));
    assert!(!error.retryable);
    assert!(!classify_error("429 rate limit").retryable);
    assert!(classify_error("network connection failed").retryable);
}

#[test]
fn deleting_a_collection_survives_restart_and_restores_all_work_without_running_jobs() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path().into()).unwrap();
    let mut project = project_with_assets();
    project
        .apply(Action::Enqueue {
            ids: project
                .assets
                .iter()
                .map(|asset| asset.id.clone())
                .collect(),
            kind: JobKind::Generate,
        })
        .unwrap();
    project.jobs[0].status = JobStatus::Generating;
    store.put(project.clone()).unwrap();
    store.select(&project.id).unwrap();
    let image = save_image(
        &store.dir(&project.id).unwrap(),
        &image::DynamicImage::new_rgba8(32, 32),
        "Original",
    )
    .unwrap();
    let original = fs::read(store.image_path(&project.id, &image.file).unwrap()).unwrap();
    store.delete(&project.id).unwrap();
    assert!(store.projects.is_empty());
    assert!(store.project(&project.id).is_err());
    drop(store);
    let mut store = Store::open(tmp.path().into()).unwrap();
    assert!(store.projects.is_empty());
    assert_eq!(store.deleted_projects.len(), 1);
    assert!(store.preferences.last_project_id.is_none());
    store.restore(&project.id).unwrap();
    let restored = store.project(&project.id).unwrap();
    assert_eq!(restored.assets.len(), 2);
    assert_eq!(restored.style.direction, project.style.direction);
    assert!(restored.paused);
    assert!(restored
        .jobs
        .iter()
        .all(|job| job.status == JobStatus::Cancelled));
    assert_eq!(
        fs::read(store.image_path(&project.id, &image.file).unwrap()).unwrap(),
        original
    );
    drop(store);
    let store = Store::open(tmp.path().into()).unwrap();
    assert_eq!(store.projects.len(), 1);
    assert!(store.deleted_projects.is_empty());
    assert!(store.preferences.last_project_id.is_none());
}

#[test]
fn deleting_another_collection_preserves_the_selected_collection() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path().into()).unwrap();
    let selected = store.put(project_with_assets()).unwrap();
    let other = store.put(project_with_assets()).unwrap();
    store.select(&selected.id).unwrap();
    assert!(store.delete("../outside").is_err());
    assert!(store.restore("../outside").is_err());
    store.delete(&other.id).unwrap();
    assert_eq!(
        store.preferences.last_project_id.as_deref(),
        Some(selected.id.as_str())
    );
    assert!(store.project(&selected.id).is_ok());
}

#[cfg(unix)]
#[test]
fn deleting_a_collection_rejects_a_replaced_symlink_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path().into()).unwrap();
    let project = store.put(project_with_assets()).unwrap();
    let dir = store.dir(&project.id).unwrap();
    let moved = tmp.path().join("outside");
    fs::rename(&dir, &moved).unwrap();
    std::os::unix::fs::symlink(&moved, &dir).unwrap();
    assert!(store.delete(&project.id).is_err());
    assert!(!moved.join(".deleted").exists());
    assert!(store.project(&project.id).is_ok());
}
