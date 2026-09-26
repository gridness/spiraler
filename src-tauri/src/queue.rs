use crate::{
    domain::*,
    provider::{CodexProvider, ImageProvider, ProviderError},
    storage::{confined, load_image, save_image},
    AppState,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};

pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Subscription limits vary by account. A single worker is deliberately conservative.
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let next = {
                let state = app.state::<AppState>();
                if state.shutting_down.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(mut store) = state.store.lock() else {
                    continue;
                };
                let mut candidates: Vec<_> = store
                    .projects
                    .values()
                    .filter(|p| !p.paused)
                    .flat_map(|p| {
                        p.jobs
                            .iter()
                            .filter(|j| j.status == JobStatus::Queued && j.retry_at <= now())
                            .map(|j| (p.id.clone(), j.id.clone(), j.created_at))
                    })
                    .collect();
                candidates.sort_by_key(|(_, _, created)| *created);
                if let Some((project_id, job_id, _)) = candidates.first() {
                    let mut project = match store.project(project_id) {
                        Ok(p) => p.clone(),
                        Err(_) => continue,
                    };
                    let Some(job) = project.jobs.iter_mut().find(|j| &j.id == job_id) else {
                        continue;
                    };
                    job.status = JobStatus::Generating;
                    job.attempts += 1;
                    job.error = None;
                    let job = job.clone();
                    let dir = match store.dir(project_id) {
                        Ok(dir) => dir,
                        Err(_) => continue,
                    };
                    match store.put(project) {
                        Ok(project) => {
                            let cancelled = Arc::new(AtomicBool::new(false));
                            if let Ok(mut active) = state.active.lock() {
                                active.insert(job.id.clone(), cancelled.clone());
                            }
                            let _ = app.emit("project-changed", &project);
                            Some((project_id.clone(), dir, job, cancelled))
                        }
                        Err(error) => {
                            let _ = app.emit("studio-error", error);
                            if let Some(p) = store.projects.get_mut(project_id) {
                                p.paused = true;
                            }
                            None
                        }
                    }
                } else {
                    None
                }
            };
            let Some((project_id, dir, job, cancelled)) = next else {
                continue;
            };
            let work =
                dir.join("jobs")
                    .join(&job.id)
                    .join(format!("attempt-{}-{}", job.attempts, id()));
            let result = if job.kind == JobKind::Upscale {
                let dir = dir.clone();
                let job = job.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    let source = job.request.source.as_ref().ok_or_else(|| {
                        ProviderError::permanent("No source image. Choose an existing result.")
                    })?;
                    let image = load_image(
                        &confined(&dir, &source.file).map_err(ProviderError::permanent)?,
                    )
                    .map_err(ProviderError::permanent)?;
                    if image.width() > 4096 || image.height() > 4096 {
                        return Err(ProviderError::permanent(
                            "Source images must be at most 4,096 pixels per side.",
                        ));
                    }
                    let enlarged = image.resize_exact(
                        image.width() * 2,
                        image.height() * 2,
                        image::imageops::FilterType::Lanczos3,
                    );
                    save_image(&dir, &enlarged, &job.asset_name).map_err(ProviderError::permanent)
                })
                .await
                .unwrap_or_else(|_| {
                    Err(ProviderError::permanent(
                        "Image processing stopped. Try again.",
                    ))
                })
            } else {
                match CodexProvider
                    .generate(&job.request, &dir, &work, cancelled.clone())
                    .await
                {
                    Ok(path) => {
                        let dir = dir.clone();
                        let name = job.asset_name.clone();
                        tauri::async_runtime::spawn_blocking(move || {
                            let image = load_image(&path).map_err(ProviderError::permanent)?;
                            save_image(&dir, &image, &name).map_err(ProviderError::permanent)
                        })
                        .await
                        .unwrap_or_else(|_| {
                            Err(ProviderError::permanent(
                                "Could not finish saving the image. Retry this job.",
                            ))
                        })
                    }
                    Err(error) => Err(error),
                }
            };
            let state = app.state::<AppState>();
            if let Ok(mut active) = state.active.lock() {
                active.remove(&job.id);
            }
            let Ok(mut store) = state.store.lock() else {
                continue;
            };
            let Ok(mut project) = store.project(&project_id).cloned() else {
                continue;
            };
            let Some(index) = project.jobs.iter().position(|j| j.id == job.id) else {
                continue;
            };
            if project.jobs[index].status == JobStatus::Cancelled
                || cancelled.load(Ordering::SeqCst)
            {
                continue;
            }
            match result {
                Ok(image) => {
                    let generation = Generation {
                        id: id(),
                        image,
                        kind: job.kind,
                        created_at: now(),
                        request: job.request,
                        provider: if job.kind == JobKind::Upscale {
                            "Local Lanczos3, 2x interpolation"
                        } else {
                            "Codex subscription, built-in image generation"
                        }
                        .into(),
                        source_result_id: job.source_result_id,
                    };
                    if let Ok(asset) = project.asset_mut(&job.asset_id) {
                        asset.preferred_id = Some(generation.id.clone());
                        asset.approved = false;
                        asset.results.push(generation);
                    }
                    project.jobs[index].status = JobStatus::Ready;
                    project.jobs[index].finished_at = Some(now());
                }
                Err(error) => {
                    if error.pause_queue {
                        project.paused = true;
                    }
                    let current = &mut project.jobs[index];
                    current.error = Some(error.message);
                    if error.retryable && current.attempts < 3 {
                        current.status = JobStatus::Queued;
                        current.retry_at = now() + 10 * 2u64.pow(current.attempts);
                    } else {
                        current.status = JobStatus::Failed;
                        current.finished_at = Some(now());
                    }
                }
            }
            match store.put(project) {
                Ok(project) => {
                    let _ = app.emit("project-changed", project);
                }
                Err(error) => {
                    if let Some(p) = store.projects.get_mut(&project_id) {
                        p.paused = true;
                    }
                    let _ = app.emit("studio-error", format!("{error} The queue is paused. Job files are preserved in the project folder."));
                }
            }
        }
    });
}
