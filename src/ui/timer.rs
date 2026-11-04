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

use super::art;

const STATUS_HEIGHT: u16 = 2;
const GAUGE_HEIGHT: u16 = 3;
/// Rows of a full-size `BigText` glyph.
const FULL_DIGITS_HEIGHT: u16 = 8;
/// Quadrant pixels are half-height: 4 rows and 4 columns per glyph, versus 8
/// rows and 8 columns per glyph at full size.
const QUADRANT_DIGITS_HEIGHT: u16 = 4;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    // The art strip only appears when it fits without shrinking the digits
    // below full size; otherwise it takes no space and the layout is unchanged.
    let show_art = area.width >= art::WIDTH
        && area.height >= STATUS_HEIGHT + FULL_DIGITS_HEIGHT + art::HEIGHT + GAUGE_HEIGHT;
    let art_height = if show_art { art::HEIGHT } else { 0 };

    let [status, digits, art_area, gauge] = Layout::vertical([
        Constraint::Length(STATUS_HEIGHT),
        Constraint::Min(0),
        Constraint::Length(art_height),
        Constraint::Length(GAUGE_HEIGHT),
    ])
    .areas(area);

    render_status(app, frame, status);
    render_digits(app, frame, digits);
    if show_art {
        art::render(app, frame, art_area);
    }
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
        spans.push(" PAUSED ".black().on_white().bold());
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
    let pixel_size = if fits(FULL_DIGITS_HEIGHT, FULL_DIGITS_HEIGHT) {
        Some((PixelSize::Full, FULL_DIGITS_HEIGHT))
    } else if fits(QUADRANT_DIGITS_HEIGHT, QUADRANT_DIGITS_HEIGHT) {
        Some((PixelSize::Quadrant, QUADRANT_DIGITS_HEIGHT))
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::config::Config;
    use crate::stats::Stats;

    fn render_to_string(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| crate::ui::view(app, frame)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    fn new_app() -> App {
        App::new(Config::default(), Stats::new(Vec::new()), Instant::now())
    }

    #[test]
    fn art_strip_shown_when_there_is_room() {
        let mut app = new_app();
        assert!(app.timer.is_paused());
        let paused = render_to_string(&app, 80, 24);
        assert!(paused.contains("PAUSED"));
        assert!(paused.contains("▌▌ PAUSED"));
        assert!(paused.contains(art::pictogram(Phase::Work, true)[1].trim()));

        app.timer.resume(app.now);
        let active = render_to_string(&app, 80, 24);
        assert!(active.contains("● ACTIVE"));
        assert!(active.contains(art::pictogram(Phase::Work, false)[1].trim()));
        assert!(!active.contains("PAUSED"));
    }

    #[test]
    fn art_strip_hidden_when_too_small() {
        let app = new_app();
        let small = render_to_string(&app, 40, 12);
        assert!(!small.contains(art::pictogram(Phase::Work, true)[1].trim()));
        assert!(!small.contains("▌▌ PAUSED"));
    }
}
