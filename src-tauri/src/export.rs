use crate::{
    domain::*,
    storage::{atomic_write, confined, io_error, safe_name},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportOptions {
    pub ids: Vec<String>,
    pub version: ExportVersion,
    pub metadata: bool,
    pub prefix: String,
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportVersion {
    Preferred,
    Original,
    Upscaled,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReport {
    pub count: usize,
    pub skipped: usize,
    pub directory: String,
}
pub fn export(
    project: &Project,
    root: &Path,
    destination: &Path,
    options: ExportOptions,
) -> Result<ExportReport> {
    bounded(&options.prefix, 80)?;
    if options.ids.is_empty() {
        return Err("Choose some assets to export.".into());
    }
    let mut selected = vec![];
    let mut skipped = 0;
    for asset in project
        .assets
        .iter()
        .filter(|a| options.ids.contains(&a.id))
    {
        let result = match options.version {
            ExportVersion::Preferred => asset.preferred(),
            ExportVersion::Original => asset.results.iter().find(|r| r.kind != JobKind::Upscale),
            ExportVersion::Upscaled => asset
                .results
                .iter()
                .rev()
                .find(|r| r.kind == JobKind::Upscale),
        };
        if let Some(result) = result {
            selected.push((asset, result));
        } else {
            skipped += 1;
        }
    }
    if selected.is_empty() {
        return Err(
            "No images match these export options. Generate images or choose another version."
                .into(),
        );
    }
    let prefix = if options.prefix.trim().is_empty() {
        String::new()
    } else {
        format!("{}-", safe_name(&options.prefix))
    };
    // Stage the complete export on the destination filesystem, then rename. Never overwrite a batch.
    let staging = tempfile::Builder::new()
        .prefix(".spiraler-export-")
        .tempdir_in(destination)
        .map_err(io_error)?;
    for (index, (asset, result)) in selected.iter().enumerate() {
        let filename = format!(
            "{prefix}{:03}-{}-{}",
            index + 1,
            safe_name(&asset.name),
            &asset.id[..8]
        );
        let image = confined(root, &result.image.file)?;
        let bytes = fs::read(image).map_err(io_error)?;
        atomic_write(&staging.path().join(format!("{filename}.png")), &bytes)?;
        if options.metadata {
            atomic_write(
                &staging.path().join(format!("{filename}.json")),
                &serde_json::to_vec_pretty(result).map_err(|_| "Could not write metadata.")?,
            )?;
        }
    }
    let folder = destination.join(format!(
        "{}-export-{}-{}",
        safe_name(&project.name),
        now(),
        &id()[..8]
    ));
    fs::rename(staging.path(), &folder).map_err(io_error)?;
    Ok(ExportReport {
        count: selected.len(),
        skipped,
        directory: folder.to_string_lossy().into(),
    })
}
