use eframe::egui::{self, Rounding, Ui};
use flux_core::Translator;

use crate::data::{format_duration, Period, Stats};
use crate::theme::Theme;

pub fn render_period_selector(
    ui: &mut Ui,
    selected: &mut Period,
    translator: &Translator,
    theme: &Theme,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme.spacing.sm;

        for period in Period::all() {
            let label = period.label(translator);
            let is_selected = *selected == *period;

            let (bg_color, text_color, stroke) = if is_selected {
                (
                    theme.colors.accent,
                    theme.colors.text_primary,
                    egui::Stroke::new(1.0, theme.colors.accent),
                )
            } else {
                (
                    theme.colors.surface,
                    theme.colors.text_secondary,
                    egui::Stroke::new(1.0, theme.colors.border),
                )
            };

            let button = egui::Button::new(
                egui::RichText::new(&label)
                    .size(theme.typography.body)
                    .color(text_color),
            )
            .fill(bg_color)
            .stroke(stroke)
            .rounding(Rounding::same(theme.rounding.md));

            if ui.add(button).clicked() {
                *selected = *period;
            }
        }
    });
}

pub fn render_stats_cards(ui: &mut Ui, stats: &Stats, translator: &Translator, theme: &Theme) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(theme.spacing.md, theme.spacing.md);

        render_stat_card(
            ui,
            theme,
            &translator.get("command.stats_total_sessions"),
            &stats.session_count.to_string(),
            Some("sessions"),
            theme.colors.accent,
        );

        render_stat_card(
            ui,
            theme,
            &translator.get("command.stats_total_time"),
            &format_duration(stats.total_seconds),
            None,
            theme.colors.success,
        );

        if stats.session_count > 0 {
            let average = stats.total_seconds / stats.session_count as i64;
            render_stat_card(
                ui,
                theme,
                &translator.get("command.stats_average_duration"),
                &format_duration(average),
                None,
                theme.colors.mode_architecture,
            );

            render_stat_card(
                ui,
                theme,
                &translator.get("command.stats_check_ins"),
                &stats.total_check_ins.to_string(),
                None,
                theme.colors.mode_review,
            );
        }
    });

    if !stats.by_mode.is_empty() {
        ui.add_space(theme.spacing.lg);

        theme.card_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new(translator.get("command.status_mode"))
                    .size(theme.typography.title)
                    .color(theme.colors.text_primary)
                    .strong(),
            );
            ui.add_space(theme.spacing.md);

            render_mode_breakdown(ui, stats, theme);
        });
    }

    if !stats.focus_applications.is_empty() {
        ui.add_space(theme.spacing.lg);

        theme.card_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new(translator.get("command.stats_focus_apps"))
                    .size(theme.typography.title)
                    .color(theme.colors.text_primary)
                    .strong(),
            );
            ui.add_space(theme.spacing.md);

            render_app_breakdown(ui, &stats.focus_applications, theme, theme.colors.accent);
        });
    }

    if !stats.distraction_applications.is_empty() {
        ui.add_space(theme.spacing.lg);

        let total_tracked: i64 = stats
            .focus_applications
            .values()
            .chain(stats.distraction_applications.values())
            .sum();
        let distraction_percent = if total_tracked > 0 {
            (stats.total_distraction_seconds as f64 / total_tracked as f64 * 100.0) as u32
        } else {
            0
        };

        theme.card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(translator.get("command.stats_distractions"))
                        .size(theme.typography.title)
                        .color(theme.colors.text_primary)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "({}% {})",
                        distraction_percent,
                        translator.get("command.stats_time_lost")
                    ))
                    .size(theme.typography.label)
                    .color(theme.colors.warning),
                );
            });
            ui.add_space(theme.spacing.md);

            render_app_breakdown(
                ui,
                &stats.distraction_applications,
                theme,
                theme.colors.warning,
            );
        });
    }
}

fn render_stat_card(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    value: &str,
    suffix: Option<&str>,
    accent_color: egui::Color32,
) {
    let frame = egui::Frame::none()
        .fill(theme.colors.surface)
        .stroke(egui::Stroke::new(1.0, theme.colors.border))
        .rounding(Rounding::same(theme.rounding.md))
        .inner_margin(egui::Margin::same(theme.spacing.md))
        .shadow(egui::epaint::Shadow {
            offset: egui::vec2(0.0, 2.0),
            blur: 8.0,
            spread: 0.0,
            color: egui::Color32::from_black_alpha(30),
        });

    frame.show(ui, |ui| {
        ui.set_min_width(140.0);

        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new(label)
                    .size(theme.typography.label)
                    .color(theme.colors.text_secondary),
            );

            ui.add_space(theme.spacing.sm);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(value)
                        .size(theme.typography.heading)
                        .color(accent_color)
                        .strong(),
                );

                if let Some(suffix) = suffix {
                    ui.label(
                        egui::RichText::new(suffix)
                            .size(theme.typography.label)
                            .color(theme.colors.text_muted),
                    );
                }
            });
        });
    });
}

fn render_mode_breakdown(ui: &mut Ui, stats: &Stats, theme: &Theme) {
    let mut modes: Vec<_> = stats.by_mode.iter().collect();
    modes.sort_by(|a, b| b.1.cmp(a.1));

    let total = stats.total_seconds.max(1) as f32;

    for (mode, seconds) in modes {
        let percentage = (*seconds as f32 / total * 100.0) as u32;
        let progress = *seconds as f32 / total;
        let mode_color = theme.colors.mode_color(mode);

        ui.horizontal(|ui| {
            ui.set_min_width(ui.available_width());

            ui.label(
                egui::RichText::new(mode)
                    .size(theme.typography.body)
                    .color(theme.colors.text_primary)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{}%", percentage))
                        .size(theme.typography.label)
                        .color(theme.colors.text_muted),
                );

                ui.label(
                    egui::RichText::new(format_duration(*seconds))
                        .size(theme.typography.body)
                        .color(theme.colors.text_secondary),
                );
            });
        });

        ui.add_space(theme.spacing.xs);

        let bar_height = 6.0;
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), bar_height),
            egui::Sense::hover(),
        );

        let rounding = Rounding::same(bar_height / 2.0);

        ui.painter()
            .rect_filled(rect, rounding, theme.colors.surface_elevated);

        let filled_width = rect.width() * progress;
        let filled_rect = egui::Rect::from_min_size(rect.min, egui::vec2(filled_width, bar_height));
        ui.painter().rect_filled(filled_rect, rounding, mode_color);

        ui.add_space(theme.spacing.md);
    }
}

fn render_app_breakdown(
    ui: &mut Ui,
    applications: &std::collections::HashMap<String, i64>,
    theme: &Theme,
    bar_color: egui::Color32,
) {
    let mut apps: Vec<_> = applications.iter().collect();
    apps.sort_by(|a, b| b.1.cmp(a.1));

    let total_app_time: i64 = apps.iter().map(|(_, s)| **s).sum();
    let total = total_app_time.max(1) as f32;

    for (application, seconds) in apps {
        let percentage = (*seconds as f32 / total * 100.0) as u32;
        let progress = *seconds as f32 / total;

        ui.horizontal(|ui| {
            ui.set_min_width(ui.available_width());

            ui.label(
                egui::RichText::new(application)
                    .size(theme.typography.body)
                    .color(theme.colors.text_primary)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{}%", percentage))
                        .size(theme.typography.label)
                        .color(theme.colors.text_muted),
                );

                ui.label(
                    egui::RichText::new(format_duration(*seconds))
                        .size(theme.typography.body)
                        .color(theme.colors.text_secondary),
                );
            });
        });

        ui.add_space(theme.spacing.xs);

        let bar_height = 6.0;
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), bar_height),
            egui::Sense::hover(),
        );

        let rounding = Rounding::same(bar_height / 2.0);

        ui.painter()
            .rect_filled(rect, rounding, theme.colors.surface_elevated);

        let filled_width = rect.width() * progress;
        let filled_rect = egui::Rect::from_min_size(rect.min, egui::vec2(filled_width, bar_height));
        ui.painter().rect_filled(filled_rect, rounding, bar_color);

        ui.add_space(theme.spacing.md);
    }
}

pub fn render_empty_state(ui: &mut Ui, translator: &Translator, theme: &Theme) {
    ui.vertical_centered(|ui| {
        ui.add_space(theme.spacing.xxl);

        ui.label(egui::RichText::new("📊").size(48.0));

        ui.add_space(theme.spacing.md);

        ui.label(
            egui::RichText::new(translator.get("command.stats_no_sessions"))
                .size(theme.typography.title)
                .color(theme.colors.text_primary),
        );

        ui.add_space(theme.spacing.sm);

        ui.label(
            egui::RichText::new("Start your first focus session:")
                .size(theme.typography.body)
                .color(theme.colors.text_secondary),
        );

        ui.add_space(theme.spacing.md);

        egui::Frame::none()
            .fill(theme.colors.surface)
            .stroke(egui::Stroke::new(1.0, theme.colors.border))
            .rounding(Rounding::same(theme.rounding.sm))
            .inner_margin(egui::Margin::symmetric(theme.spacing.md, theme.spacing.sm))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("flux start")
                        .size(theme.typography.body)
                        .color(theme.colors.accent)
                        .monospace(),
                );
            });
    });
}
