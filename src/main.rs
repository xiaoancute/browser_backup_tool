use std::{io, time::Duration};

use anyhow::Result;
use browser_backup_tool::{
    app::{AppMode, AppState},
    backup::{BackupRequest, create_backup},
    discovery::discover_browsers,
    ui,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> Result<()> {
    let browsers = discover_browsers();
    let app = AppState::new(browsers);
    run_terminal_app(app)
}

fn run_terminal_app(mut app: AppState) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        match key.code {
            KeyCode::Char('q') => break,
            KeyCode::Esc => {
                if app.mode() == AppMode::BrowserList {
                    break;
                }
                app.go_back();
            }
            KeyCode::Enter => match app.mode() {
                AppMode::BrowserList => app.open_detail(),
                AppMode::BackupConfirm => {
                    if let Some((browser, profile)) = selected_clone(app) {
                        let output_root = backup_output_root();
                        match create_backup(BackupRequest {
                            browser: &browser,
                            profile: &profile,
                            output_root: &output_root,
                        }) {
                            Ok(result) => app.set_backup_result(format!(
                                "备份完成: {}",
                                result.backup_dir.display()
                            )),
                            Err(error) => app.set_backup_result(format!("备份失败: {error}")),
                        }
                    } else {
                        app.set_backup_result("没有可备份的 Profile".to_string());
                    }
                }
                AppMode::BackupResult | AppMode::ProfileDetail | AppMode::RestoreSelect => {
                    app.go_back();
                }
            },
            KeyCode::Char('b') => app.open_backup(),
            KeyCode::Char('r') => app.open_restore(),
            KeyCode::Down => app.next_profile(),
            KeyCode::Up => app.previous_profile(),
            KeyCode::Right => app.next_browser(),
            KeyCode::Left => app.previous_browser(),
            _ => {}
        }
    }

    Ok(())
}

fn selected_clone(
    app: &AppState,
) -> Option<(
    browser_backup_tool::discovery::BrowserInstallation,
    browser_backup_tool::discovery::BrowserProfile,
)> {
    let browser = app.selected_browser()?.clone();
    let profile = app.selected_profile()?.clone();
    Some((browser, profile))
}

fn backup_output_root() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        })
        .join("browser-backups")
}
