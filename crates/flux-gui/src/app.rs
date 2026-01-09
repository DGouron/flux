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

    fn render_history(&self, ui: &mut egui::Ui) {
        let sessions = self.data.sessions_for_period(self.selected_period);
        views::history::render_session_list(ui, &sessions, &self.data.translator, &self.theme);
    }
}
