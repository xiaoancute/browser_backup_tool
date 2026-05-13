use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrowserId {
    Chrome,
    Chromium,
    Brave,
    Edge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserProfile {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserInstallation {
    pub id: BrowserId,
    pub display_name: String,
    pub data_dir: PathBuf,
    pub profiles: Vec<BrowserProfile>,
}

struct BrowserDefinition {
    id: BrowserId,
    display_name: &'static str,
    relative_data_dir: &'static str,
}

const BROWSER_DEFINITIONS: &[BrowserDefinition] = &[
    BrowserDefinition {
        id: BrowserId::Chrome,
        display_name: "Google Chrome",
        relative_data_dir: "google-chrome",
    },
    BrowserDefinition {
        id: BrowserId::Chromium,
        display_name: "Chromium",
        relative_data_dir: "chromium",
    },
    BrowserDefinition {
        id: BrowserId::Brave,
        display_name: "Brave",
        relative_data_dir: "BraveSoftware/Brave-Browser",
    },
    BrowserDefinition {
        id: BrowserId::Edge,
        display_name: "Microsoft Edge",
        relative_data_dir: "microsoft-edge",
    },
];

pub fn discover_browsers() -> Vec<BrowserInstallation> {
    dirs::config_dir()
        .map(|config_home| discover_browsers_in(config_home))
        .unwrap_or_default()
}

pub fn discover_browsers_in(config_home: impl AsRef<Path>) -> Vec<BrowserInstallation> {
    let config_home = config_home.as_ref();

    BROWSER_DEFINITIONS
        .iter()
        .filter_map(|definition| {
            let data_dir = config_home.join(definition.relative_data_dir);
            let profiles = discover_profiles_in(&data_dir);

            (!profiles.is_empty()).then(|| BrowserInstallation {
                id: definition.id.clone(),
                display_name: definition.display_name.to_string(),
                data_dir,
                profiles,
            })
        })
        .collect()
}

fn discover_profiles_in(data_dir: &Path) -> Vec<BrowserProfile> {
    let Ok(entries) = std::fs::read_dir(data_dir) else {
        return Vec::new();
    };

    let mut profiles: Vec<BrowserProfile> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();

            (path.is_dir() && is_user_profile_name(&name)).then_some(BrowserProfile { name, path })
        })
        .collect();

    profiles.sort_by(|a, b| profile_sort_key(&a.name).cmp(&profile_sort_key(&b.name)));
    profiles
}

fn is_user_profile_name(name: &str) -> bool {
    name == "Default" || name.starts_with("Profile ")
}

fn profile_sort_key(name: &str) -> (u8, String) {
    if name == "Default" {
        (0, String::new())
    } else {
        (1, name.to_string())
    }
}
