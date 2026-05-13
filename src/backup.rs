use crate::discovery::{BrowserId, BrowserInstallation, BrowserProfile};
use anyhow::{Context, Result};
use flate2::{Compression, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tar::Builder;

#[derive(Debug)]
pub struct BackupRequest<'a> {
    pub browser: &'a BrowserInstallation,
    pub profile: &'a BrowserProfile,
    pub output_root: &'a Path,
}

#[derive(Debug, Eq, PartialEq)]
pub struct BackupResult {
    pub backup_dir: PathBuf,
    pub metadata_path: PathBuf,
    pub archive_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BackupMetadata {
    pub app: String,
    pub version: String,
    pub created_at_unix: u64,
    pub platform: String,
    pub browser: String,
    pub browser_display_name: String,
    pub browser_data_dir: String,
    pub profile_name: String,
    pub profile_path: String,
}

pub fn create_backup(request: BackupRequest<'_>) -> Result<BackupResult> {
    let timestamp = current_unix_timestamp();
    let backup_dir = request.output_root.join(format!(
        "browser-backup-{}-{}",
        browser_slug(&request.browser.id),
        timestamp
    ));
    fs::create_dir_all(&backup_dir)
        .with_context(|| format!("create backup dir {}", backup_dir.display()))?;

    let metadata = BackupMetadata {
        app: "browser-backup-tool".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        created_at_unix: timestamp,
        platform: std::env::consts::OS.to_string(),
        browser: browser_slug(&request.browser.id).to_string(),
        browser_display_name: request.browser.display_name.clone(),
        browser_data_dir: request.browser.data_dir.display().to_string(),
        profile_name: request.profile.name.clone(),
        profile_path: request.profile.path.display().to_string(),
    };

    let metadata_path = backup_dir.join("metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    fs::write(&metadata_path, metadata_json)
        .with_context(|| format!("write metadata {}", metadata_path.display()))?;

    let archive_path = backup_dir.join("profile.tar.gz");
    create_profile_archive(&archive_path, &request.profile.path)?;

    Ok(BackupResult {
        backup_dir,
        metadata_path,
        archive_path,
    })
}

fn create_profile_archive(archive_path: &Path, profile_path: &Path) -> Result<()> {
    let archive_file = File::create(archive_path)
        .with_context(|| format!("create archive {}", archive_path.display()))?;
    let encoder = GzEncoder::new(archive_file, Compression::default());
    let mut archive = Builder::new(encoder);
    archive
        .append_dir_all("profile", profile_path)
        .with_context(|| format!("archive profile {}", profile_path.display()))?;
    archive.finish()?;
    Ok(())
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn browser_slug(browser_id: &BrowserId) -> &'static str {
    match browser_id {
        BrowserId::Chrome => "chrome",
        BrowserId::Chromium => "chromium",
        BrowserId::Brave => "brave",
        BrowserId::Edge => "edge",
    }
}
