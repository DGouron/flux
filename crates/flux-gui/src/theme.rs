use eframe::egui::{self, Color32, Rounding, Stroke};

pub struct Theme {
    pub colors: Colors,
    pub spacing: Spacing,
    pub typography: Typography,
    pub rounding: RoundingScale,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            colors: Colors::dark(),
            spacing: Spacing::default(),
            typography: Typography::default(),
            rounding: RoundingScale::default(),
        }
    }

    #[allow(clippy::field_reassign_with_default)]
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        visuals.override_text_color = Some(self.colors.text_primary);
        visuals.hyperlink_color = self.colors.accent;
        visuals.faint_bg_color = self.colors.surface;
        visuals.extreme_bg_color = self.colors.background;
        visuals.code_bg_color = self.colors.surface_elevated;
        visuals.warn_fg_color = self.colors.warning;
        visuals.error_fg_color = self.colors.error;

        visuals.window_fill = self.colors.background;
        visuals.window_stroke = Stroke::new(1.0, self.colors.border);
        visuals.window_rounding = Rounding::same(self.rounding.lg);

        visuals.panel_fill = self.colors.background;

        visuals.widgets.noninteractive.bg_fill = self.colors.surface;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.colors.text_secondary);
        visuals.widgets.noninteractive.rounding = Rounding::same(self.rounding.md);

        visuals.widgets.inactive.bg_fill = self.colors.surface;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.colors.text_primary);
        visuals.widgets.inactive.rounding = Rounding::same(self.rounding.md);

        visuals.widgets.hovered.bg_fill = self.colors.surface_hover;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.colors.text_primary);
        visuals.widgets.hovered.rounding = Rounding::same(self.rounding.md);

        visuals.widgets.active.bg_fill = self.colors.accent;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.active.rounding = Rounding::same(self.rounding.md);

        visuals.selection.bg_fill = self.colors.accent.linear_multiply(0.3);
        visuals.selection.stroke = Stroke::new(1.0, self.colors.accent);

        let mut style = egui::Style::default();
        style.visuals = visuals;

        style.spacing.item_spacing = egui::vec2(self.spacing.sm, self.spacing.sm);
        style.spacing.window_margin = egui::Margin::same(self.spacing.lg);
        style.spacing.button_padding = egui::vec2(self.spacing.md, self.spacing.sm);

        ctx.set_style(style);
    }

    pub fn card_frame(&self) -> egui::Frame {
        egui::Frame::none()
            .fill(self.colors.surface)
            .stroke(Stroke::new(1.0, self.colors.border))
            .rounding(Rounding::same(self.rounding.md))
            .inner_margin(egui::Margin::same(self.spacing.md))
            .shadow(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 2.0),
                blur: 8.0,
                spread: 0.0,
                color: Color32::from_black_alpha(40),
            })
    }
}

pub struct Colors {
    pub background: Color32,
    pub surface: Color32,
    pub surface_elevated: Color32,
    pub surface_hover: Color32,
    pub border: Color32,

    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,

    pub accent: Color32,

    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,

    pub mode_ai_assisted: Color32,
    pub mode_review: Color32,
    pub mode_architecture: Color32,
    pub mode_custom: Color32,
}

impl Colors {
    pub fn dark() -> Self {
        Self {
            background: Color32::from_rgb(15, 15, 15),
            surface: Color32::from_rgb(26, 26, 26),
            surface_elevated: Color32::from_rgb(32, 32, 32),
            surface_hover: Color32::from_rgb(38, 38, 38),
            border: Color32::from_rgb(45, 45, 45),

            text_primary: Color32::from_rgb(250, 250, 250),
            text_secondary: Color32::from_rgb(168, 168, 168),
            text_muted: Color32::from_rgb(105, 105, 105),

            accent: Color32::from_rgb(59, 130, 246),

            success: Color32::from_rgb(16, 185, 129),
            warning: Color32::from_rgb(245, 158, 11),
            error: Color32::from_rgb(239, 68, 68),

            mode_ai_assisted: Color32::from_rgb(59, 130, 246),
            mode_review: Color32::from_rgb(168, 85, 247),
            mode_architecture: Color32::from_rgb(6, 182, 212),
            mode_custom: Color32::from_rgb(16, 185, 129),
        }
    }

    pub fn mode_color(&self, mode: &str) -> Color32 {
        match mode.to_lowercase().as_str() {
            "prompting" | "ai-assisted" => self.mode_ai_assisted,
            "review" => self.mode_review,
            "architecture" => self.mode_architecture,
            _ => self.mode_custom,
        }
    }
}

pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xxl: f32,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xxl: 48.0,
        }
    }
}

pub struct Typography {
    pub heading: f32,
    pub title: f32,
    pub body: f32,
    pub label: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            heading: 24.0,
            title: 16.0,
            body: 14.0,
            label: 12.0,
        }
    }
}

pub struct RoundingScale {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
}

impl Default for RoundingScale {
    fn default() -> Self {
        Self {
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
        }
    }
}
