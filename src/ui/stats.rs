use chrono::Local;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Paragraph, Sparkline};

use crate::app::App;

const DAYS: usize = 14;
/// Bar width 3 + gap 1 per day, plus the block borders.
const BAR_CHART_MIN_WIDTH: u16 = DAYS as u16 * 4 + 2;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    // The only impure bit: the current date, read once per render.
    let today = Local::now().date_naive();
    let [summary, chart] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

    render_summary(app, frame, summary, today);
    render_chart(app, frame, chart, today);
}

fn render_summary(app: &App, frame: &mut Frame, area: Rect, today: chrono::NaiveDate) {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    let focused = app.stats.focused_today(today).as_secs();
    let (h, m) = (focused / 3600, focused % 3600 / 60);
    let line = Line::from(vec![
        Span::raw("Today: "),
        Span::styled(app.stats.count_on(today).to_string(), bold),
        Span::raw("   This week: "),
        Span::styled(app.stats.count_this_week(today).to_string(), bold),
        Span::raw("   Total: "),
        Span::styled(app.stats.total().to_string(), bold),
        Span::raw("   This session: "),
        Span::styled(app.work_completed.to_string(), bold),
        Span::raw("   Focused today: "),
        Span::styled(format!("{h}h {m:02}m"), bold),
    ]);
    frame.render_widget(
        Paragraph::new(line).block(Block::bordered().title(" Stats ")),
        area,
    );
}

fn render_chart(app: &App, frame: &mut Frame, area: Rect, today: chrono::NaiveDate) {
    let data = app.stats.per_day(today, DAYS);
    let max = data.iter().map(|&(_, c)| c).max().unwrap_or(0).max(1);
    let style = Style::new().fg(app.phase.color());
    let block = Block::bordered().title(" Pomodoros, last 14 days ");

    if area.width >= BAR_CHART_MIN_WIDTH {
        let bars: Vec<Bar> = data
            .iter()
            .map(|(day, count)| Bar::with_label(day.format("%d").to_string(), *count))
            .collect();
        let chart = BarChart::default()
            .block(block)
            .data(BarGroup::new(bars))
            .bar_width(3)
            .bar_gap(1)
            .bar_style(style)
            .max(max);
        frame.render_widget(chart, area);
    } else {
        let counts: Vec<u64> = data.iter().map(|&(_, c)| c).collect();
        let spark = Sparkline::default()
            .block(block)
            .data(counts)
            .style(style)
            .max(max);
        frame.render_widget(spark, area);
    }
}
