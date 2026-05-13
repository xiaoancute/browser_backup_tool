use browser_backup_tool::discovery::{BrowserId, discover_browsers_in};
use tempfile::TempDir;

#[test]
fn discovers_chromium_profiles_from_config_home() {
    let config_home = TempDir::new().expect("temp config home");
    let brave_default = config_home
        .path()
        .join("BraveSoftware/Brave-Browser/Default");
    let brave_profile_1 = config_home
        .path()
        .join("BraveSoftware/Brave-Browser/Profile 1");
    let chrome_system_profile = config_home.path().join("google-chrome/System Profile");

    std::fs::create_dir_all(&brave_default).expect("create Brave Default");
    std::fs::create_dir_all(&brave_profile_1).expect("create Brave Profile 1");
    std::fs::create_dir_all(&chrome_system_profile).expect("create ignored Chrome profile");

    let browsers = discover_browsers_in(config_home.path());

    let brave = browsers
        .iter()
        .find(|browser| browser.id == BrowserId::Brave)
        .expect("Brave should be discovered");

    assert_eq!(brave.display_name, "Brave");
    assert_eq!(brave.profiles.len(), 2);
    assert_eq!(brave.profiles[0].name, "Default");
    assert_eq!(brave.profiles[1].name, "Profile 1");

    assert!(
        browsers
            .iter()
            .all(|browser| browser.id != BrowserId::Chrome),
        "Chrome should be ignored when it only contains System Profile"
    );
}

#[test]
fn returns_empty_list_when_no_known_browser_dirs_exist() {
    let config_home = TempDir::new().expect("temp config home");

    let browsers = discover_browsers_in(config_home.path());

    assert!(browsers.is_empty());
}
