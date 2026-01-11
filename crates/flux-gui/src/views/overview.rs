use std::f32::consts::PI;

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

    if stats.sessions_with_metrics > 0 {
        ui.add_space(theme.spacing.lg);

        theme.card_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new(translator.get("command.stats_focus_score"))
                    .size(theme.typography.title)
                    .color(theme.colors.text_primary)
                    .strong(),
            );
            ui.add_space(theme.spacing.md);

            ui.horizontal(|ui| {
                if let Some(score) = stats.average_focus_score {
                    render_focus_score_gauge(ui, score, translator, theme);
                }

                ui.add_space(theme.spacing.lg);

                ui.vertical(|ui| {
                    render_metric_row(
                        ui,
                        &translator.get("command.stats_context_switches"),
                        stats.total_context_switches,
                        theme.colors.warning,
                        theme,
                    );
                    ui.add_space(theme.spacing.md);
                    render_metric_row(
                        ui,
                        &translator.get("command.stats_short_bursts"),
                        stats.total_short_bursts,
                        theme.colors.error,
                        theme,
                    );
                });
            });
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

fn render_focus_score_gauge(ui: &mut Ui, score: u8, _translator: &Translator, theme: &Theme) {
    let gauge_size = 100.0;
    let stroke_width = 8.0;
    let radius = (gauge_size - stroke_width) / 2.0;

    let (rect, _response) = ui.allocate_exact_size(
        egui::vec2(gauge_size, gauge_size * 0.6),
        egui::Sense::hover(),
    );

    let center = egui::pos2(rect.center().x, rect.max.y - stroke_width / 2.0);
    let painter = ui.painter();

    let start_angle = PI;
    let end_angle = 0.0;
    let arc_sweep = start_angle - end_angle;

    draw_arc(
        painter,
        center,
        radius,
        start_angle,
        end_angle,
        stroke_width,
        theme.colors.surface_elevated,
    );

    let progress = score as f32 / 100.0;
    let filled_end_angle = start_angle - (arc_sweep * progress);
    let score_color = focus_score_color(score, theme);

    draw_arc(
        painter,
        center,
        radius,
        start_angle,
        filled_end_angle,
        stroke_width,
        score_color,
    );

    painter.text(
        egui::pos2(center.x, center.y - radius * 0.4),
        egui::Align2::CENTER_CENTER,
        score.to_string(),
        egui::FontId::proportional(theme.typography.heading * 1.2),
        score_color,
    );
}

fn draw_arc(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    stroke_width: f32,
    color: egui::Color32,
) {
    let segments = 32;
    let angle_step = (start_angle - end_angle) / segments as f32;

    let mut points = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let angle = start_angle - (angle_step * i as f32);
        let x = center.x + radius * angle.cos();
        let y = center.y - radius * angle.sin();
        points.push(egui::pos2(x, y));
    }

    let stroke = egui::Stroke::new(stroke_width, color);
    for window in points.windows(2) {
        painter.line_segment([window[0], window[1]], stroke);
    }
}

fn focus_score_color(score: u8, theme: &Theme) -> egui::Color32 {
    if score >= 80 {
        theme.colors.success
    } else if score >= 50 {
        theme.colors.warning
    } else {
        theme.colors.error
    }
}

fn render_metric_row(ui: &mut Ui, label: &str, value: u32, color: egui::Color32, theme: &Theme) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(theme.typography.body)
                .color(theme.colors.text_secondary),
        );
        ui.label(
            egui::RichText::new(value.to_string())
                .size(theme.typography.body)
                .color(color)
                .strong(),
        );
    });
}
