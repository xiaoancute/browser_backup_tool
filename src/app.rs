use crate::discovery::{BrowserInstallation, BrowserProfile};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppMode {
    BrowserList,
    ProfileDetail,
    BackupConfirm,
    BackupResult,
    RestoreSelect,
}

#[derive(Clone, Debug)]
pub struct AppState {
    browsers: Vec<BrowserInstallation>,
    selected_browser: usize,
    selected_profile: usize,
    mode: AppMode,
    status_message: Option<String>,
}

impl AppState {
    pub fn new(browsers: Vec<BrowserInstallation>) -> Self {
        Self {
            browsers,
            selected_browser: 0,
            selected_profile: 0,
            mode: AppMode::BrowserList,
            status_message: None,
        }
    }

    pub fn browsers(&self) -> &[BrowserInstallation] {
        &self.browsers
    }

    pub fn selected_browser_index(&self) -> usize {
        self.selected_browser
    }

    pub fn selected_profile_index(&self) -> usize {
        self.selected_profile
    }

    pub fn mode(&self) -> AppMode {
        self.mode
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn selected_browser(&self) -> Option<&BrowserInstallation> {
        self.browsers.get(self.selected_browser)
    }

    pub fn selected_profile(&self) -> Option<&BrowserProfile> {
        self.selected_browser()
            .and_then(|browser| browser.profiles.get(self.selected_profile))
    }

    pub fn next_browser(&mut self) {
        if self.mode != AppMode::BrowserList {
            return;
        }

        if self.browsers.is_empty() {
            return;
        }

        self.selected_browser = (self.selected_browser + 1) % self.browsers.len();
        self.selected_profile = 0;
    }

    pub fn previous_browser(&mut self) {
        if self.mode != AppMode::BrowserList {
            return;
        }

        if self.browsers.is_empty() {
            return;
        }

        self.selected_browser =
            (self.selected_browser + self.browsers.len() - 1) % self.browsers.len();
        self.selected_profile = 0;
    }

    pub fn next_profile(&mut self) {
        if self.mode != AppMode::BrowserList {
            return;
        }

        let Some(browser) = self.selected_browser() else {
            return;
        };

        if browser.profiles.is_empty() {
            return;
        }

        self.selected_profile = (self.selected_profile + 1) % browser.profiles.len();
    }

    pub fn previous_profile(&mut self) {
        if self.mode != AppMode::BrowserList {
            return;
        }

        let Some(browser) = self.selected_browser() else {
            return;
        };

        if browser.profiles.is_empty() {
            return;
        }

        self.selected_profile =
            (self.selected_profile + browser.profiles.len() - 1) % browser.profiles.len();
    }

    pub fn open_detail(&mut self) {
        if self.selected_profile().is_some() {
            self.status_message = None;
            self.mode = AppMode::ProfileDetail;
        }
    }

    pub fn open_backup(&mut self) {
        if self.selected_profile().is_some() {
            self.status_message = None;
            self.mode = AppMode::BackupConfirm;
        }
    }

    pub fn open_restore(&mut self) {
        self.status_message = None;
        self.mode = AppMode::RestoreSelect;
    }

    pub fn set_backup_result(&mut self, message: String) {
        self.status_message = Some(message);
        self.mode = AppMode::BackupResult;
    }

    pub fn go_back(&mut self) {
        self.status_message = None;
        self.mode = AppMode::BrowserList;
    }
}
