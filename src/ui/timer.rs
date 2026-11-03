use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Gauge, Paragraph};
use tui_big_text::{BigText, PixelSize};

use crate::app::App;
use crate::config::LONG_BREAK_EVERY;
use crate::phase::Phase;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let [status, digits, gauge] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(area);

    render_status(app, frame, status);
    render_digits(app, frame, digits);
    render_gauge(app, frame, gauge);
}

fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    let progress = match app.phase {
        Phase::Work => format!("pomodoro #{}", app.work_completed + 1),
        Phase::ShortBreak | Phase::LongBreak => format!("{} done", app.work_completed),
    };
    let mut spans = vec![
        Span::styled(
            app.phase.label(),
            Style::new()
                .fg(app.phase.color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            " · {progress} · cycle {}/{LONG_BREAK_EVERY}",
            app.cycle_position()
        )),
    ];
    if app.timer.is_paused() {
        spans.push(Span::raw("  "));
        spans.push(" PAUSED ".black().on_yellow().bold());
    }
    frame.render_widget(Paragraph::new(Line::from(spans)).centered(), area);
}

fn render_digits(app: &App, frame: &mut Frame, area: Rect) {
    let text = format_mm_ss(app.timer.remaining(app.now));
    let mut style = Style::new()
        .fg(app.phase.color())
        .add_modifier(Modifier::BOLD);
    if app.timer.is_paused() {
        style = style.add_modifier(Modifier::DIM);
    }

    // BigText does not report its size; each glyph is 8x8 font pixels.
    let chars = u16::try_from(text.chars().count()).unwrap_or(u16::MAX);
    let fits = |cell_w: u16, cell_h: u16| {
        chars.saturating_mul(cell_w) <= area.width && cell_h <= area.height
    };
    let pixel_size = if fits(8, 8) {
        Some((PixelSize::Full, 8))
    } else if fits(4, 4) {
        Some((PixelSize::Quadrant, 4))
    } else {
        None
    };

    match pixel_size {
        Some((pixel_size, height)) => {
            let big = BigText::builder()
                .pixel_size(pixel_size)
                .style(style)
                .lines(vec![Line::from(text)])
                .alignment(Alignment::Center)
                .build();
            frame.render_widget(big, vertically_centered(area, height));
        }
        None => {
            let plain = Paragraph::new(text).style(style).centered();
            frame.render_widget(plain, vertically_centered(area, 1));
        }
    }
}

fn render_gauge(app: &App, frame: &mut Frame, area: Rect) {
    // `Timer::progress` guarantees a finite value in 0.0..=1.0, as `Gauge::ratio` requires.
    let ratio = app.timer.progress(app.now);
    let gauge = Gauge::default()
        .block(Block::bordered())
        .gauge_style(Style::new().fg(app.phase.color()).bg(Color::Reset))
        .use_unicode(true)
        .ratio(ratio)
        .label(format!("{:.0}%", ratio * 100.0));
    frame.render_widget(gauge, area);
}

fn vertically_centered(area: Rect, height: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    area
}

/// `mm:ss`, rounding up so the display reads `25:00` at the start and only
/// shows `00:00` once the phase is actually over.
fn format_mm_ss(remaining: Duration) -> String {
    let secs = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
    format!("{:02}:{:02}", secs / 60, secs % 60)
}
