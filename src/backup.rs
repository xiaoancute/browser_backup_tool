use crate::discovery::{BrowserId, BrowserInstallation, BrowserProfile};
use anyhow::{Context, Result};
use flate2::{Compression, write::GzEncoder};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::mpsc,
    time::{SystemTime, UNIX_EPOCH},
};
use tar::Builder;

pub fn log_error(message: &str) {
    let log_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("browser-backups");
    let _ = fs::create_dir_all(&log_dir);

    let log_path = log_dir.join("backup.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(file, "[{ts}] {message}");
    }
}

#[derive(Clone, Debug)]
pub struct BackupProgress {
    pub total_files: u64,
    pub processed_files: u64,
    pub current_file: Option<String>,
}

pub enum BackupMessage {
    Progress(BackupProgress),
    Done(Result<BackupResult, String>),
}

#[derive(Debug)]
pub struct BackupRequest<'a> {
    pub browser: &'a BrowserInstallation,
    pub profile: &'a BrowserProfile,
    pub output_root: &'a Path,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupResult {
    pub backup_dir: PathBuf,
    pub metadata_path: PathBuf,
    pub archive_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    let encoder = archive.into_inner()?;
    encoder.finish()?;
    Ok(())
}

pub fn create_backup_with_progress(
    request: BackupRequest<'_>,
    sender: mpsc::Sender<BackupMessage>,
) {
    let result = do_backup_with_progress(&request, &sender);
    if let Err(e) = &result {
        log_error(&format!(
            "备份失败: browser={}, profile={}, error={e}",
            request.browser.display_name, request.profile.name
        ));
    }
    let _ = sender.send(BackupMessage::Done(match &result {
        Ok(r) => Ok(r.clone()),
        Err(e) => Err(format!("{e}")),
    }));
}

fn do_backup_with_progress(
    request: &BackupRequest<'_>,
    sender: &mpsc::Sender<BackupMessage>,
) -> Result<BackupResult> {
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
    create_profile_archive_with_progress(&archive_path, &request.profile.path, sender)?;

    Ok(BackupResult {
        backup_dir,
        metadata_path,
        archive_path,
    })
}

fn create_profile_archive_with_progress(
    archive_path: &Path,
    profile_path: &Path,
    sender: &mpsc::Sender<BackupMessage>,
) -> Result<()> {
    let total_files = count_files(profile_path)?;

    let archive_file = File::create(archive_path)
        .with_context(|| format!("create archive {}", archive_path.display()))?;
    let encoder = GzEncoder::new(archive_file, Compression::fast());
    let mut archive = Builder::new(encoder);

    let mut processed: u64 = 0;
    archive_dir_with_progress(&mut archive, Path::new("profile"), profile_path, sender, &mut processed, total_files)?;

    let encoder = archive.into_inner()?;
    encoder.finish()?;
    Ok(())
}

fn archive_dir_with_progress<W: std::io::Write>(
    archive: &mut Builder<W>,
    archive_prefix: &Path,
    fs_path: &Path,
    sender: &mpsc::Sender<BackupMessage>,
    processed: &mut u64,
    total: u64,
) -> Result<()> {
    archive.append_dir(archive_prefix, fs_path)?;

    for entry in fs::read_dir(fs_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let relative = archive_prefix.join(&name);
        let ft = entry.file_type()?;

        if ft.is_dir() {
            archive_dir_with_progress(archive, &relative, &path, sender, processed, total)?;
        } else if ft.is_file() {
            *processed += 1;
            let _ = sender.send(BackupMessage::Progress(BackupProgress {
                total_files: total,
                processed_files: *processed,
                current_file: Some(relative.to_string_lossy().to_string()),
            }));
            let mut file = File::open(&path)
                .with_context(|| format!("open {}", path.display()))?;
            archive
                .append_file(&relative, &mut file)
                .with_context(|| format!("archive file {}", path.display()))?;
        }
    }
    Ok(())
}

fn count_files(path: &Path) -> Result<u64> {
    let mut total: u64 = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_dir() {
            total += count_files(&entry.path())?;
        } else if ft.is_file() {
            total += 1;
        }
    }
    Ok(total)
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
