use ratatui::Frame;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::App;

const SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "Global",
        &[
            ("q / ctrl-c", "quit"),
            ("?", "help"),
            (",", "settings"),
            ("tab / shift-tab / 1 / 2", "switch screen"),
        ],
    ),
    (
        "Timer",
        &[
            ("space", "start / pause / resume"),
            ("s", "skip phase"),
            ("r", "reset phase"),
            ("+ / = / -", "+/-1 min on the running timer"),
        ],
    ),
    (
        "Settings",
        &[
            ("↑/↓ or j/k", "select"),
            ("+/- or ←/→ or h/l", "change"),
            ("enter", "save"),
            ("esc", "cancel"),
        ],
    ),
];

pub fn render(_app: &App, frame: &mut Frame) {
    let inner = super::popup_area(frame, "Help", 60, 21);
    let mut lines: Vec<Line> = Vec::new();
    for (title, bindings) in SECTIONS {
        lines.push(Line::from(*title).underlined());
        for (key, desc) in *bindings {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{key:<24}")).bold(),
                Span::raw(*desc),
            ]));
        }
    }
    lines.push(Line::default());
    lines.push(
        Line::from("Settings apply to the next phase; +/- changes the current one.").dark_gray(),
    );
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}
