use browser_backup_tool::{
    backup::{BackupRequest, create_backup},
    discovery::{BrowserId, BrowserInstallation, BrowserProfile},
};
use flate2::read::GzDecoder;
use std::path::PathBuf;
use tar::Archive;
use tempfile::TempDir;

#[test]
fn creates_metadata_and_profile_archive() {
    let temp = TempDir::new().expect("temp dir");
    let profile_path = temp.path().join("Default");
    let profile_file = profile_path.join("Preferences");
    std::fs::create_dir_all(&profile_path).expect("profile dir");
    std::fs::write(&profile_file, br#"{"profile":"ok"}"#).expect("profile file");

    let browser = BrowserInstallation {
        id: BrowserId::Brave,
        display_name: "Brave".to_string(),
        data_dir: temp.path().join("BraveSoftware/Brave-Browser"),
        profiles: vec![],
    };
    let profile = BrowserProfile {
        name: "Default".to_string(),
        path: profile_path,
    };
    let output_root = temp.path().join("backups");

    let result = create_backup(BackupRequest {
        browser: &browser,
        profile: &profile,
        output_root: &output_root,
    })
    .expect("backup should be created");

    assert!(result.backup_dir.starts_with(output_root));
    assert!(result.metadata_path.exists());
    assert!(result.archive_path.exists());

    let metadata_text = std::fs::read_to_string(result.metadata_path).expect("metadata text");
    assert!(metadata_text.contains("\"browser\": \"brave\""));
    assert!(metadata_text.contains("\"profile_name\": \"Default\""));

    let archive_file = std::fs::File::open(result.archive_path).expect("archive file");
    let decoder = GzDecoder::new(archive_file);
    let mut archive = Archive::new(decoder);
    let entry_paths: Vec<PathBuf> = archive
        .entries()
        .expect("archive entries")
        .map(|entry| {
            entry
                .expect("entry")
                .path()
                .expect("entry path")
                .into_owned()
        })
        .collect();

    assert!(
        entry_paths
            .iter()
            .any(|path| path == &PathBuf::from("profile"))
    );
    assert!(
        entry_paths
            .iter()
            .any(|path| path == &PathBuf::from("profile/Preferences"))
    );
}
