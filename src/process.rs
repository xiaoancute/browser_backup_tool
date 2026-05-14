use crate::discovery::BrowserId;
use std::path::Path;

pub fn browser_running(browser_id: &BrowserId) -> bool {
    browser_running_in_proc_root(browser_id, "/proc")
}

pub fn browser_running_in_proc_root(browser_id: &BrowserId, proc_root: impl AsRef<Path>) -> bool {
    let Ok(entries) = std::fs::read_dir(proc_root) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !file_name
            .chars()
            .all(|character| character.is_ascii_digit())
        {
            return false;
        }

        process_matches(browser_id, &entry.path())
    })
}

fn process_matches(browser_id: &BrowserId, pid_dir: &Path) -> bool {
    let needles = browser_process_names(browser_id);
    let comm_matches = std::fs::read_to_string(pid_dir.join("comm"))
        .map(|comm| name_matches(&normalize_process_text(&comm), needles))
        .unwrap_or(false);

    if comm_matches {
        return true;
    }

    std::fs::read(pid_dir.join("cmdline"))
        .map(|cmdline| {
            cmdline
                .split(|byte| *byte == 0)
                .filter_map(|argument| std::str::from_utf8(argument).ok())
                .any(|argument| executable_argument_matches(argument, needles))
        })
        .unwrap_or(false)
}

fn name_matches(name: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| name == *needle)
}

fn executable_argument_matches(argument: &str, needles: &[&str]) -> bool {
    let normalized = normalize_process_text(argument);
    let basename = normalized.rsplit('/').next().unwrap_or(&normalized);
    name_matches(basename, needles)
}

fn normalize_process_text(text: &str) -> String {
    text.trim().to_ascii_lowercase()
}

fn browser_process_names(browser_id: &BrowserId) -> &'static [&'static str] {
    match browser_id {
        BrowserId::Chrome => &["chrome", "google-chrome"],
        BrowserId::Chromium => &["chromium", "chromium-browser"],
        BrowserId::Brave => &["brave", "brave-browser"],
        BrowserId::Edge => &["msedge", "microsoft-edge"],
    }
}
