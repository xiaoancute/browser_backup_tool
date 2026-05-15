use crate::app::{AppFocus, AppMode, AppState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame<'_>, app: &AppState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, root[0]);
    render_body(frame, app, root[1]);
    render_footer(frame, root[2]);
}

fn render_header(frame: &mut Frame<'_>, area: Rect) {
    let header = Paragraph::new("Browser Backup Tool")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

fn render_body(frame: &mut Frame<'_>, app: &AppState, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(area);

    render_browser_list(frame, app, columns[0]);
    render_secondary_panel(frame, app, columns[1]);
}

fn render_browser_list(frame: &mut Frame<'_>, app: &AppState, area: Rect) {
    let lines = if app.browsers().is_empty() {
        vec![Line::from("没有检测到支持的浏览器 Profile")]
    } else {
        app.browsers()
            .iter()
            .enumerate()
            .map(|(index, browser)| {
                selectable_line(index == app.selected_browser_index(), &browser.display_name)
            })
            .collect()
    };

    let title = if app.focus() == AppFocus::Browser {
        "浏览器 *"
    } else {
        "浏览器"
    };
    let widget = Paragraph::new(lines)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn render_secondary_panel(frame: &mut Frame<'_>, app: &AppState, area: Rect) {
    let (title, lines) = match app.mode() {
        AppMode::BrowserList => {
            let title = if app.focus() == AppFocus::Profile {
                "Profile *"
            } else {
                "Profile"
            };
            (title, profile_lines(app))
        }
        AppMode::ProfileDetail => ("Profile Detail", detail_lines(app)),
        AppMode::BackupConfirm => ("Backup", backup_lines(app)),
        AppMode::BackupRunning => ("Backup Running", running_lines(app)),
        AppMode::BackupResult => ("Backup Result", backup_result_lines(app)),
        AppMode::RestoreSelect => ("Restore", restore_lines(app)),
    };

    let widget = Paragraph::new(lines)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let footer = Paragraph::new(footer_text())
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

fn selectable_line(selected: bool, text: &str) -> Line<'static> {
    if selected {
        Line::from(vec![Span::styled(
            format!("> {text}"),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])
    } else {
        Line::from(format!("  {text}"))
    }
}

fn profile_lines(app: &AppState) -> Vec<Line<'static>> {
    match app.selected_browser() {
        Some(browser) => browser
            .profiles
            .iter()
            .enumerate()
            .map(|(index, profile)| {
                let selected = index == app.selected_profile_index();
                selectable_line(
                    selected,
                    &format!("{}  {}", profile.name, profile.path.display()),
                )
            })
            .collect(),
        None => vec![Line::from("没有可显示的 Profile")],
    }
}

fn detail_lines(app: &AppState) -> Vec<Line<'static>> {
    let Some(browser) = app.selected_browser() else {
        return vec![Line::from("没有可显示的浏览器")];
    };

    let Some(profile) = app.selected_profile() else {
        return vec![Line::from("没有可显示的 Profile")];
    };

    vec![
        Line::from(format!("Browser: {}", browser.display_name)),
        Line::from(format!("Data dir: {}", browser.data_dir.display())),
        Line::from(format!("Profile: {}", profile.name)),
        Line::from(format!("Profile path: {}", profile.path.display())),
        Line::from(""),
        Line::from("Enter: 返回主界面"),
        Line::from("b: 打开备份确认"),
        Line::from("r: 打开恢复页面"),
    ]
}

fn backup_lines(app: &AppState) -> Vec<Line<'static>> {
    let Some(profile) = app.selected_profile() else {
        return vec![Line::from("没有选中的 Profile")];
    };

    vec![
        Line::from(format!("准备备份: {}", profile.name)),
        Line::from(format!("Profile path: {}", profile.path.display())),
        Line::from(""),
        Line::from("会写入 ~/browser-backups 下的新备份目录。"),
        Line::from("备份内容包括 metadata.json 和 profile.tar.gz。"),
        Line::from(""),
        Line::from("Enter: 开始备份"),
        Line::from("Esc: 返回主界面"),
    ]
}

fn backup_result_lines(app: &AppState) -> Vec<Line<'static>> {
    status_lines(app)
}

fn running_lines(app: &AppState) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(
            app.status_message()
                .unwrap_or("正在备份...")
                .to_string(),
        ),
        Line::from(""),
    ];

    let total = app.backup_total_files();
    let processed = app.backup_processed_files();

    if total > 0 {
        let pct = (processed as f64 / total as f64 * 100.0).min(100.0);
        let bar_width = 36usize;
        let filled = (pct / 100.0 * bar_width as f64) as usize;
        let bar = format!(
            "{}{}",
            "█".repeat(filled),
            "░".repeat(bar_width - filled)
        );
        lines.push(Line::from(format!("[{bar}] {pct:.1}%")));
        lines.push(Line::from(format!(
            "已归档 {} / {} 个文件",
            processed, total
        )));
    }

    if let Some(file) = app.backup_current_file() {
        lines.push(Line::from(""));
        lines.push(Line::from(format!("当前文件: {file}")));
    }

    lines
}

fn status_lines(app: &AppState) -> Vec<Line<'static>> {
    let message = app.status_message().unwrap_or("没有备份结果");

    vec![
        Line::from(message.to_string()),
        Line::from(""),
        Line::from("Enter: 返回主界面"),
        Line::from("Esc: 返回主界面"),
    ]
}

fn restore_lines(app: &AppState) -> Vec<Line<'static>> {
    let Some(profile) = app.selected_profile() else {
        return vec![Line::from("没有选中的 Profile")];
    };

    vec![
        Line::from(format!("准备恢复到: {}", profile.name)),
        Line::from(""),
        Line::from("目前这里只是恢复入口占位，后面会接备份包选择和覆盖确认。"),
        Line::from(""),
        Line::from("Enter: 返回主界面"),
        Line::from("Esc: 返回主界面"),
    ]
}

fn footer_text() -> &'static str {
    "Tab Focus  ↑↓ Move  ←→ Browser  Enter Detail  b Backup  r Restore  q Quit"
}
