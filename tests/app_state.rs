use browser_backup_tool::app::{AppMode, AppState};
use browser_backup_tool::discovery::{BrowserId, BrowserInstallation, BrowserProfile};
use std::path::PathBuf;

fn sample_browsers() -> Vec<BrowserInstallation> {
    vec![
        BrowserInstallation {
            id: BrowserId::Brave,
            display_name: "Brave".to_string(),
            data_dir: PathBuf::from("/tmp/brave"),
            profiles: vec![
                BrowserProfile {
                    name: "Default".to_string(),
                    path: PathBuf::from("/tmp/brave/Default"),
                },
                BrowserProfile {
                    name: "Profile 1".to_string(),
                    path: PathBuf::from("/tmp/brave/Profile 1"),
                },
            ],
        },
        BrowserInstallation {
            id: BrowserId::Chrome,
            display_name: "Google Chrome".to_string(),
            data_dir: PathBuf::from("/tmp/chrome"),
            profiles: vec![BrowserProfile {
                name: "Default".to_string(),
                path: PathBuf::from("/tmp/chrome/Default"),
            }],
        },
    ]
}

#[test]
fn app_state_moves_selection_within_browser_and_profile_lists() {
    let mut app = AppState::new(sample_browsers());

    assert_eq!(app.selected_browser_index(), 0);
    assert_eq!(app.selected_profile_index(), 0);

    app.next_profile();
    assert_eq!(app.selected_profile_index(), 1);

    app.next_profile();
    assert_eq!(app.selected_profile_index(), 0);

    app.next_browser();
    assert_eq!(app.selected_browser_index(), 1);
    assert_eq!(app.selected_profile_index(), 0);

    app.previous_browser();
    assert_eq!(app.selected_browser_index(), 0);
    assert_eq!(app.selected_profile_index(), 0);
}

#[test]
fn app_state_updates_profile_selection_when_browser_changes() {
    let mut app = AppState::new(sample_browsers());

    app.next_profile();
    assert_eq!(app.selected_profile_index(), 1);

    app.next_browser();
    assert_eq!(app.selected_browser_index(), 1);
    assert_eq!(app.selected_profile_index(), 0);
}

#[test]
fn app_state_opens_detail_backup_and_restore_modes_for_selected_profile() {
    let mut app = AppState::new(sample_browsers());

    assert_eq!(app.mode(), AppMode::BrowserList);

    app.open_detail();
    assert_eq!(app.mode(), AppMode::ProfileDetail);
    assert_eq!(app.selected_profile().unwrap().name, "Default");

    app.go_back();
    assert_eq!(app.mode(), AppMode::BrowserList);

    app.open_backup();
    assert_eq!(app.mode(), AppMode::BackupConfirm);

    app.go_back();
    assert_eq!(app.mode(), AppMode::BrowserList);

    app.open_restore();
    assert_eq!(app.mode(), AppMode::RestoreSelect);
}

#[test]
fn app_state_stores_backup_result_message_until_returning_to_list() {
    let mut app = AppState::new(sample_browsers());

    app.set_backup_result("backup created".to_string());

    assert_eq!(app.mode(), AppMode::BackupResult);
    assert_eq!(app.status_message(), Some("backup created"));

    app.go_back();

    assert_eq!(app.mode(), AppMode::BrowserList);
    assert_eq!(app.status_message(), None);
}

#[test]
fn list_navigation_is_ignored_outside_browser_list_mode() {
    let mut app = AppState::new(sample_browsers());

    app.open_detail();
    app.next_browser();
    app.next_profile();

    assert_eq!(app.selected_browser_index(), 0);
    assert_eq!(app.selected_profile_index(), 0);
}
