use ratatui::Frame;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::app::{App, SettingsField};

pub fn render(app: &App, frame: &mut Frame) {
    let Some(edit) = &app.settings else {
        return;
    };
    let inner = super::popup_area(frame, "Settings", 52, 7);
    let mut lines: Vec<Line> = SettingsField::all()
        .into_iter()
        .map(|field| {
            let selected = field == edit.selected;
            let marker = if selected { "> " } else { "  " };
            let line = Line::from(format!(
                "{marker}{:<12} {:>3} min",
                field.label(),
                edit.draft.minutes(field.phase())
            ));
            if selected {
                line.style(
                    Style::new()
                        .fg(field.phase().color())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                line
            }
        })
        .collect();
    lines.push(Line::default());
    lines.push(Line::from(super::SETTINGS_HINT).dark_gray());
    frame.render_widget(Paragraph::new(lines), inner);
}
