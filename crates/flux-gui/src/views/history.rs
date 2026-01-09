use chrono::{DateTime, Local, Utc};
use eframe::egui::{self, Rounding, ScrollArea, Ui};
use flux_core::{Session, Translator};

use crate::data::format_duration;
use crate::theme::Theme;

pub fn render_session_list(
    ui: &mut Ui,
    sessions: &[&Session],
    translator: &Translator,
    theme: &Theme,
) {
    if sessions.is_empty() {
        render_empty_state(ui, translator, theme);
        return;
    }

    let mut sorted_sessions: Vec<_> = sessions.iter().collect();
    sorted_sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            for session in sorted_sessions {
                render_session_row(ui, session, theme);
                ui.add_space(theme.spacing.sm);
            }
        });
}

fn render_session_row(ui: &mut Ui, session: &Session, theme: &Theme) {
    let frame = egui::Frame::none()
        .fill(theme.colors.surface)
        .stroke(egui::Stroke::new(1.0, theme.colors.border))
        .rounding(Rounding::same(theme.rounding.md))
        .inner_margin(egui::Margin::same(theme.spacing.md));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width() - theme.spacing.md * 2.0);

        ui.horizontal(|ui| {
            let mode_color = theme.colors.mode_color(&session.mode.to_string());

            ui.label(
                egui::RichText::new(mode_icon(&session.mode.to_string()))
                    .size(theme.typography.heading)
                    .color(mode_color),
            );

            ui.add_space(theme.spacing.sm);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(session.mode.to_string())
                            .size(theme.typography.body)
                            .color(theme.colors.text_primary)
                            .strong(),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format_datetime(session.started_at))
                                .size(theme.typography.label)
                                .color(theme.colors.text_muted),
                        );
                    });
                });

                ui.add_space(theme.spacing.xs);

                ui.horizontal(|ui| {
                    render_session_stat(
                        ui,
                        theme,
                        "⏱",
                        &format_duration(session.duration_seconds.unwrap_or(0)),
                    );

                    ui.add_space(theme.spacing.md);

                    render_session_stat(
                        ui,
                        theme,
                        "✓",
                        &format!("{} check-ins", session.check_in_count),
                    );
                });
            });
        });
    });
}

fn render_session_stat(ui: &mut Ui, theme: &Theme, icon: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(icon)
                .size(theme.typography.label)
                .color(theme.colors.text_secondary),
        );

        ui.label(
            egui::RichText::new(value)
                .size(theme.typography.label)
                .color(theme.colors.text_secondary),
        );
    });
}

fn render_empty_state(ui: &mut Ui, translator: &Translator, theme: &Theme) {
    ui.vertical_centered(|ui| {
        ui.add_space(theme.spacing.xxl);

        ui.label(egui::RichText::new("📋").size(48.0));

        ui.add_space(theme.spacing.md);

        ui.label(
            egui::RichText::new(translator.get("gui.history_empty"))
                .size(theme.typography.title)
                .color(theme.colors.text_primary),
        );
    });
}

fn format_datetime(datetime: DateTime<Utc>) -> String {
    let local: DateTime<Local> = datetime.into();
    local.format("%d/%m/%Y %H:%M").to_string()
}

fn mode_icon(mode: &str) -> &'static str {
    match mode.to_lowercase().as_str() {
        "code" => "💻",
        "architecture" => "🏗️",
        "review" => "👁️",
        "learning" => "📚",
        "writing" => "✍️",
        _ => "⚡",
    }
}
