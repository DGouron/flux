use eframe::egui::{self, Rounding, ScrollArea};

use crate::data::{Period, Stats, StatsData};
use crate::theme::Theme;
use crate::views;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    History,
}

pub struct FluxApp {
    data: StatsData,
    selected_period: Period,
    current_stats: Stats,
    current_view: View,
    theme: Theme,
    theme_applied: bool,
    show_clear_modal: bool,
}

impl FluxApp {
    pub fn new(data: StatsData) -> Self {
        let current_stats = data.stats_for_period(Period::Today);

        Self {
            data,
            selected_period: Period::Today,
            current_stats,
            current_view: View::Overview,
            theme: Theme::dark(),
            theme_applied: false,
            show_clear_modal: false,
        }
    }

    fn update_stats(&mut self) {
        self.current_stats = self.data.stats_for_period(self.selected_period);
    }
}

impl eframe::App for FluxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            self.theme.apply(ctx);
            self.theme_applied = true;
        }

        let panel_frame = egui::Frame::none()
            .fill(self.theme.colors.background)
            .inner_margin(egui::Margin::same(self.theme.spacing.lg));

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚡").size(self.theme.typography.heading));
                    ui.label(
                        egui::RichText::new("Flux Dashboard")
                            .size(self.theme.typography.heading)
                            .color(self.theme.colors.text_primary)
                            .strong(),
                    );
                });

                ui.add_space(self.theme.spacing.md);

                self.render_view_tabs(ui);

                ui.add_space(self.theme.spacing.md);

                let previous_period = self.selected_period;
                views::overview::render_period_selector(
                    ui,
                    &mut self.selected_period,
                    &self.data.translator,
                    &self.theme,
                );

                if self.selected_period != previous_period {
                    self.update_stats();
                }

                ui.add_space(self.theme.spacing.lg);

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.current_view {
                        View::Overview => self.render_overview(ui),
                        View::History => self.render_history(ui),
                    });
            });

        self.render_clear_modal(ctx);
    }
}

impl FluxApp {
    fn render_view_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = self.theme.spacing.sm;

            let tabs = [
                (
                    View::Overview,
                    &self.data.translator.get("gui.tab_overview"),
                ),
                (View::History, &self.data.translator.get("gui.tab_history")),
            ];

            for (view, label) in tabs {
                let is_selected = self.current_view == view;

                let (bg_color, text_color, stroke) = if is_selected {
                    (
                        self.theme.colors.surface_elevated,
                        self.theme.colors.text_primary,
                        egui::Stroke::new(2.0, self.theme.colors.accent),
                    )
                } else {
                    (
                        egui::Color32::TRANSPARENT,
                        self.theme.colors.text_secondary,
                        egui::Stroke::NONE,
                    )
                };

                let button = egui::Button::new(
                    egui::RichText::new(label.clone())
                        .size(self.theme.typography.body)
                        .color(text_color),
                )
                .fill(bg_color)
                .stroke(stroke)
                .rounding(Rounding::same(self.theme.rounding.sm));

                if ui.add(button).clicked() {
                    self.current_view = view;
                }
            }
        });
    }

    fn render_overview(&self, ui: &mut egui::Ui) {
        if self.data.has_sessions() {
            views::overview::render_stats_cards(
                ui,
                &self.current_stats,
                &self.data.translator,
                &self.theme,
            );

            let daily_data = self.data.daily_focus_for_period(self.selected_period);
            if !daily_data.is_empty() {
                ui.add_space(self.theme.spacing.lg);

                self.theme.card_frame().show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(self.data.translator.get("gui.chart_title"))
                            .size(self.theme.typography.title)
                            .color(self.theme.colors.text_primary)
                            .strong(),
                    );
                    ui.add_space(self.theme.spacing.md);

                    views::chart::render_focus_chart(ui, &daily_data, &self.theme);
                });
            }
        } else {
            views::overview::render_empty_state(ui, &self.data.translator, &self.theme);
        }
    }

    fn render_history(&mut self, ui: &mut egui::Ui) {
        let sessions = self.data.sessions_for_period(self.selected_period);
        let session_count = sessions.len();

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let clear_button = egui::Button::new(
                    egui::RichText::new(self.data.translator.get("gui.clear_all"))
                        .size(self.theme.typography.label)
                        .color(self.theme.colors.error),
                )
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::new(1.0, self.theme.colors.error))
                .rounding(Rounding::same(self.theme.rounding.sm));

                if session_count > 0 && ui.add(clear_button).clicked() {
                    self.show_clear_modal = true;
                }
            });
        });

        ui.add_space(self.theme.spacing.md);

        let action =
            views::history::render_session_list(ui, &sessions, &self.data.translator, &self.theme);

        if let views::history::HistoryAction::DeleteSession(id) = action {
            if self.data.delete_session(id).is_ok() {
                self.update_stats();
            }
        }
    }

    fn render_clear_modal(&mut self, ctx: &egui::Context) {
        if !self.show_clear_modal {
            return;
        }

        let session_count = self.data.sessions.len();

        egui::Window::new(self.data.translator.get("gui.clear_confirm_title"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.add_space(self.theme.spacing.md);

                let message = self
                    .data
                    .translator
                    .get("gui.clear_confirm_message")
                    .replace("{count}", &session_count.to_string());

                ui.label(
                    egui::RichText::new(message)
                        .size(self.theme.typography.body)
                        .color(self.theme.colors.text_primary),
                );

                ui.add_space(self.theme.spacing.lg);

                ui.horizontal(|ui| {
                    let cancel_button = egui::Button::new(
                        egui::RichText::new(self.data.translator.get("gui.clear_cancel"))
                            .size(self.theme.typography.body),
                    )
                    .rounding(Rounding::same(self.theme.rounding.sm));

                    if ui.add(cancel_button).clicked() {
                        self.show_clear_modal = false;
                    }

                    ui.add_space(self.theme.spacing.md);

                    let confirm_button = egui::Button::new(
                        egui::RichText::new(self.data.translator.get("gui.clear_confirm"))
                            .size(self.theme.typography.body)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(self.theme.colors.error)
                    .rounding(Rounding::same(self.theme.rounding.sm));

                    if ui.add(confirm_button).clicked() {
                        if self.data.clear_sessions().is_ok() {
                            self.update_stats();
                        }
                        self.show_clear_modal = false;
                    }
                });
            });
    }
}
