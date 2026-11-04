mod art;
mod help;
mod settings;
mod stats;
mod timer;

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Tabs};

use crate::app::{App, Popup, Screen};

/// Key hints for the settings popup (footer and popup body).
pub const SETTINGS_HINT: &str = "↑/↓ or j/k select · +/- or ←/→ change · enter save · esc cancel";

/// Render the whole UI. A pure function of `app`: no clocks, no I/O.
pub fn view(app: &App, frame: &mut Frame) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(app, frame, header);
    match app.screen {
        Screen::Timer => timer::render(app, frame, body),
        Screen::Stats => stats::render(app, frame, body),
    }
    render_footer(app, frame, footer);

    match app.popup {
        Some(Popup::Help) => help::render(app, frame),
        Some(Popup::Settings) => settings::render(app, frame),
        None => {}
    }
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let titles = Screen::ALL
        .iter()
        .enumerate()
        .map(|(i, screen)| format!("{} [{}]", screen.title(), i + 1));
    let tabs = Tabs::new(titles)
        .select(app.screen.index())
        .highlight_style(
            Style::new()
                .fg(app.phase.color())
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::bordered().title(" pomodoro "));
    frame.render_widget(tabs, area);
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let hints = match (app.popup, app.screen) {
        (Some(Popup::Help), _) => "esc / ? / q close",
        (Some(Popup::Settings), _) => SETTINGS_HINT,
        (None, Screen::Timer) => {
            "space start/pause · s skip · r reset · +/- 1 min · tab stats · , settings · ? help · q quit"
        }
        (None, Screen::Stats) => "tab timer · , settings · ? help · q quit",
    };
    let mut spans = Vec::new();
    if let Some(error) = &app.last_error {
        spans.push(Span::styled(
            format!("{error} "),
            Style::new().fg(Color::Red),
        ));
    }
    spans.push(Span::raw(hints).dark_gray());
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// A `width` x `height` rectangle centered in `area` (shrunk to fit).
pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

/// Clear a centered `width` x `height` box over the frame, draw a bordered
/// block titled `title` in it, and return the inner area for the content.
pub fn popup_area(frame: &mut Frame, title: &str, width: u16, height: u16) -> Rect {
    let area = centered_rect(frame.area(), width, height);
    let block = Block::bordered().title(format!(" {title} ")).bold();
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    inner
}
