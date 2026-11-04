//! A small phase pictogram with the phase name and run state beside it,
//! drawn between the big timer digits and the progress gauge.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::phase::Phase;

/// Rows of the strip (pictogram and text column alike).
pub const HEIGHT: u16 = 4;
/// Every pictogram row is exactly this many columns, so the strip never
/// shifts when the phase or run state changes.
const PICTOGRAM_WIDTH: u16 = 12;
const GAP: u16 = 2;
/// Wide enough for the longest phase label, "Short break".
const TEXT_WIDTH: u16 = 11;
/// Total width of the strip.
pub const WIDTH: u16 = PICTOGRAM_WIDTH + GAP + TEXT_WIDTH;

type Pictogram = [&'static str; HEIGHT as usize];

/// Tomato with its leaves up.
const TOMATO: Pictogram = [
    r"    \ | /   ",
    r"   .-=|=-.  ",
    r"  (       ) ",
    r"   `-----'  ",
];
/// Tomato with a drooping stem.
const TOMATO_PAUSED: Pictogram = [
    r"      ,     ",
    r"   .--'--.  ",
    r"  (       ) ",
    r"   `-----'  ",
];
/// Steaming coffee cup.
const COFFEE: Pictogram = [
    r"   ) ) )    ",
    r"  .=====.   ",
    r"  |     |]  ",
    r"  `-----'   ",
];
/// Coffee cup gone cold.
const COFFEE_PAUSED: Pictogram = [
    r"            ",
    r"  .=====.   ",
    r"  |     |]  ",
    r"  `-----'   ",
];
/// Bed with someone snoring.
const BED: Pictogram = [
    r"   _    z Z ",
    r"  | |______ ",
    r"  |_|______|",
    r"  | |      |",
];
/// Empty, quiet bed.
const BED_PAUSED: Pictogram = [
    r"   _        ",
    r"  | |______ ",
    r"  |_|______|",
    r"  | |      |",
];

/// The pictogram for `phase`; the paused variant is the "resting" one.
pub fn pictogram(phase: Phase, paused: bool) -> Pictogram {
    match (phase, paused) {
        (Phase::Work, false) => TOMATO,
        (Phase::Work, true) => TOMATO_PAUSED,
        (Phase::ShortBreak, false) => COFFEE,
        (Phase::ShortBreak, true) => COFFEE_PAUSED,
        (Phase::LongBreak, false) => BED,
        (Phase::LongBreak, true) => BED_PAUSED,
    }
}

/// Draw the strip centered in `area`. The caller ensures `area` is at least
/// `WIDTH` x `HEIGHT`; a smaller area just clips.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let strip = super::centered_rect(area, WIDTH, HEIGHT);
    let [art, _gap, text] = Layout::horizontal([
        Constraint::Length(PICTOGRAM_WIDTH),
        Constraint::Length(GAP),
        Constraint::Length(TEXT_WIDTH),
    ])
    .areas(strip);

    let paused = app.timer.is_paused();
    let color = app.phase.color();

    let mut art_style = Style::new().fg(color);
    if paused {
        art_style = art_style.add_modifier(Modifier::DIM);
    }
    let rows: Vec<Line> = pictogram(app.phase, paused)
        .into_iter()
        .map(Line::from)
        .collect();
    frame.render_widget(Paragraph::new(rows).style(art_style), art);

    let bold = Style::new().add_modifier(Modifier::BOLD);
    let state = if paused {
        Line::styled("▌▌ PAUSED", bold)
    } else {
        Line::styled("● ACTIVE", bold.fg(color))
    };
    let lines = vec![
        Line::default(),
        Line::styled(app.phase.label(), bold.fg(color)),
        state,
        Line::default(),
    ];
    frame.render_widget(Paragraph::new(lines), text);
}

#[cfg(test)]
mod tests {
    use super::*;

    const PHASES: [Phase; 3] = [Phase::Work, Phase::ShortBreak, Phase::LongBreak];

    #[test]
    fn pictograms_are_fixed_size_ascii() {
        for phase in PHASES {
            for paused in [false, true] {
                let art = pictogram(phase, paused);
                assert_eq!(art.len(), usize::from(HEIGHT));
                for row in art {
                    assert!(row.is_ascii(), "{phase:?}/{paused}: non-ASCII row {row:?}");
                    assert_eq!(
                        row.len(),
                        usize::from(PICTOGRAM_WIDTH),
                        "{phase:?}/{paused}: row {row:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn paused_variant_differs_from_active() {
        for phase in PHASES {
            assert_ne!(pictogram(phase, false), pictogram(phase, true), "{phase:?}");
        }
    }

    #[test]
    fn labels_fit_text_column() {
        for phase in PHASES {
            assert!(phase.label().chars().count() <= usize::from(TEXT_WIDTH));
        }
    }
}
