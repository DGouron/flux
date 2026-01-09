use chrono::Datelike;
use eframe::egui::{self, Ui};
use egui_plot::{Bar, BarChart, Plot, PlotBounds};

use crate::data::DailyFocus;
use crate::theme::Theme;

pub fn render_focus_chart(ui: &mut Ui, daily_data: &[DailyFocus], theme: &Theme) {
    if daily_data.is_empty() {
        return;
    }

    let max_minutes = daily_data.iter().map(|day| day.minutes).max().unwrap_or(60);
    let y_max = ((max_minutes as f64 * 1.6) / 30.0).ceil() * 30.0;

    let bars: Vec<Bar> = daily_data
        .iter()
        .enumerate()
        .map(|(index, day)| {
            Bar::new(index as f64, day.minutes as f64)
                .width(0.6)
                .fill(theme.colors.accent)
                .name(format!(
                    "{} {}: {}min ({} sessions)",
                    weekday_label(day.date.weekday()),
                    day.date.format("%d/%m"),
                    day.minutes,
                    day.session_count
                ))
        })
        .collect();

    let chart = BarChart::new(bars);

    let x_labels: Vec<String> = daily_data
        .iter()
        .map(|day| {
            format!(
                "{}\n{}",
                weekday_label(day.date.weekday()),
                day.date.format("%d")
            )
        })
        .collect();

    let plot_height = 180.0;
    let x_max = daily_data.len() as f64;

    Plot::new("focus_chart")
        .height(plot_height)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .show_axes([true, true])
        .show_grid(true)
        .include_y(0.0)
        .include_y(y_max)
        .set_margin_fraction(egui::vec2(0.02, 0.05))
        .x_axis_formatter(move |mark, _range| {
            let index = mark.value.round() as usize;
            x_labels.get(index).cloned().unwrap_or_default()
        })
        .y_axis_formatter(|mark, _range| {
            let value = mark.value as i64;
            if value >= 60 {
                format!("{}h", value / 60)
            } else {
                format!("{}min", value)
            }
        })
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
            plot_ui.set_plot_bounds(PlotBounds::from_min_max([-0.5, 0.0], [x_max - 0.5, y_max]));
        });
}

fn weekday_label(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "L",
        chrono::Weekday::Tue => "M",
        chrono::Weekday::Wed => "M",
        chrono::Weekday::Thu => "J",
        chrono::Weekday::Fri => "V",
        chrono::Weekday::Sat => "S",
        chrono::Weekday::Sun => "D",
    }
}
