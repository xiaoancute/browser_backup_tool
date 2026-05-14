use browser_backup_tool::{discovery::BrowserId, process::browser_running_in_proc_root};
use tempfile::TempDir;

#[test]
fn detects_matching_browser_process_from_proc_comm() {
    let proc_root = TempDir::new().expect("proc root");
    let pid_dir = proc_root.path().join("1234");
    std::fs::create_dir_all(&pid_dir).expect("pid dir");
    std::fs::write(pid_dir.join("comm"), "chrome\n").expect("comm");

    assert!(browser_running_in_proc_root(
        &BrowserId::Chrome,
        proc_root.path()
    ));
}

#[test]
fn detects_matching_browser_process_from_proc_cmdline() {
    let proc_root = TempDir::new().expect("proc root");
    let pid_dir = proc_root.path().join("5678");
    std::fs::create_dir_all(&pid_dir).expect("pid dir");
    std::fs::write(
        pid_dir.join("cmdline"),
        b"/usr/bin/brave-browser\0--profile-directory=Default",
    )
    .expect("cmdline");

    assert!(browser_running_in_proc_root(
        &BrowserId::Brave,
        proc_root.path()
    ));
}

#[test]
fn ignores_non_matching_processes() {
    let proc_root = TempDir::new().expect("proc root");
    let pid_dir = proc_root.path().join("42");
    std::fs::create_dir_all(&pid_dir).expect("pid dir");
    std::fs::write(pid_dir.join("comm"), "zsh\n").expect("comm");

    assert!(!browser_running_in_proc_root(
        &BrowserId::Edge,
        proc_root.path()
    ));
}

#[test]
fn ignores_chrome_crashpad_helper_when_checking_chrome() {
    let proc_root = TempDir::new().expect("proc root");
    let pid_dir = proc_root.path().join("99");
    std::fs::create_dir_all(&pid_dir).expect("pid dir");
    std::fs::write(pid_dir.join("comm"), "chrome_crashpad\n").expect("comm");
    std::fs::write(
        pid_dir.join("cmdline"),
        b"/opt/google/chrome/chrome_crashpad_handler\0",
    )
    .expect("cmdline");

    assert!(!browser_running_in_proc_root(
        &BrowserId::Chrome,
        proc_root.path()
    ));
}

#[test]
fn detects_chrome_executable_path_without_matching_helpers() {
    let proc_root = TempDir::new().expect("proc root");
    let pid_dir = proc_root.path().join("100");
    std::fs::create_dir_all(&pid_dir).expect("pid dir");
    std::fs::write(
        pid_dir.join("cmdline"),
        b"/opt/google/chrome/chrome\0--type=gpu-process",
    )
    .expect("cmdline");

    assert!(browser_running_in_proc_root(
        &BrowserId::Chrome,
        proc_root.path()
    ));
}

#[test]
fn handles_missing_proc_data_as_not_running() {
    let proc_root = TempDir::new().expect("proc root");
    assert!(!browser_running_in_proc_root(
        &BrowserId::Chromium,
        proc_root.path().join("missing")
    ));
}
