use eframe::egui::{self, Rounding, Ui};
use flux_core::{FocusMode, Translator};
use flux_protocol::{Request, Response};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::client::DaemonClient;
use crate::theme::Theme;

const STATUS_POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub enum SessionStatus {
    Unknown,
    DaemonUnavailable,
    NoSession,
    Active {
        remaining_seconds: u64,
        mode: Option<FocusMode>,
    },
    Paused {
        remaining_seconds: u64,
        mode: Option<FocusMode>,
    },
}

pub enum SessionCommand {
    Start { duration: u64, mode: FocusMode },
    Stop,
    Pause,
    Resume,
    RefreshStatus,
}

pub struct SessionController {
    command_sender: mpsc::Sender<SessionCommand>,
    status_receiver: mpsc::Receiver<SessionStatus>,
    current_status: SessionStatus,
    last_poll: Instant,
    pending_action: bool,
    session_just_ended: bool,
}

impl SessionController {
    pub fn new(runtime: &tokio::runtime::Handle) -> Self {
        let (command_sender, command_receiver) = mpsc::channel::<SessionCommand>();
        let (status_sender, status_receiver) = mpsc::channel::<SessionStatus>();

        runtime.spawn(Self::background_task(command_receiver, status_sender));

        Self {
            command_sender,
            status_receiver,
            current_status: SessionStatus::Unknown,
            last_poll: Instant::now() - STATUS_POLL_INTERVAL,
            pending_action: false,
            session_just_ended: false,
        }
    }

    async fn background_task(
        command_receiver: mpsc::Receiver<SessionCommand>,
        status_sender: mpsc::Sender<SessionStatus>,
    ) {
        let client = DaemonClient::new();

        loop {
            match command_receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(command) => {
                    let status = Self::execute_command(&client, command).await;
                    let _ = status_sender.send(status);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    async fn execute_command(client: &DaemonClient, command: SessionCommand) -> SessionStatus {
        let request = match command {
            SessionCommand::Start { duration, mode } => Request::StartSession {
                duration: Some(duration),
                mode: Some(mode),
            },
            SessionCommand::Stop => Request::StopSession,
            SessionCommand::Pause => Request::PauseSession,
            SessionCommand::Resume => Request::ResumeSession,
            SessionCommand::RefreshStatus => Request::GetStatus,
        };

        match client.send(request).await {
            Ok(Response::SessionStatus {
                active,
                remaining_seconds,
                mode,
                paused,
            }) => {
                if !active {
                    SessionStatus::NoSession
                } else if paused {
                    SessionStatus::Paused {
                        remaining_seconds,
                        mode,
                    }
                } else {
                    SessionStatus::Active {
                        remaining_seconds,
                        mode,
                    }
                }
            }
            Ok(Response::Ok) => SessionStatus::Unknown,
            Ok(Response::Error { .. }) => SessionStatus::Unknown,
            Ok(Response::Pong) => SessionStatus::Unknown,
            Err(_) => SessionStatus::DaemonUnavailable,
        }
    }

    pub fn poll(&mut self, context: &egui::Context) {
        self.session_just_ended = false;

        while let Ok(status) = self.status_receiver.try_recv() {
            let was_active = self.has_active_session();
            let is_now_inactive = matches!(status, SessionStatus::NoSession);

            if was_active && is_now_inactive {
                self.session_just_ended = true;
            }

            self.current_status = status;
            self.pending_action = false;
            context.request_repaint();
        }

        if self.last_poll.elapsed() >= STATUS_POLL_INTERVAL {
            let _ = self.command_sender.send(SessionCommand::RefreshStatus);
            self.last_poll = Instant::now();
        }

        if self.has_active_session() {
            context.request_repaint_after(STATUS_POLL_INTERVAL);
        }
    }

    fn has_active_session(&self) -> bool {
        matches!(
            self.current_status,
            SessionStatus::Active { .. } | SessionStatus::Paused { .. }
        )
    }

    pub fn session_just_ended(&self) -> bool {
        self.session_just_ended
    }

    pub fn status(&self) -> &SessionStatus {
        &self.current_status
    }

    pub fn start_session(&mut self, duration: u64, mode: FocusMode) {
        self.pending_action = true;
        let _ = self
            .command_sender
            .send(SessionCommand::Start { duration, mode });
    }

    pub fn stop_session(&mut self) {
        self.pending_action = true;
        let _ = self.command_sender.send(SessionCommand::Stop);
    }

    pub fn pause_session(&mut self) {
        self.pending_action = true;
        let _ = self.command_sender.send(SessionCommand::Pause);
    }

    pub fn resume_session(&mut self) {
        self.pending_action = true;
        let _ = self.command_sender.send(SessionCommand::Resume);
    }

    pub fn is_pending(&self) -> bool {
        self.pending_action
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationPreset {
    Short,
    Pomodoro,
    Long,
    DeepWork,
    Custom,
}

impl DurationPreset {
    pub fn all() -> &'static [DurationPreset] {
        &[
            DurationPreset::Short,
            DurationPreset::Pomodoro,
            DurationPreset::Long,
            DurationPreset::DeepWork,
            DurationPreset::Custom,
        ]
    }

    pub fn minutes(&self) -> u64 {
        match self {
            DurationPreset::Short => 15,
            DurationPreset::Pomodoro => 25,
            DurationPreset::Long => 45,
            DurationPreset::DeepWork => 90,
            DurationPreset::Custom => 25,
        }
    }

    pub fn label(&self, translator: &Translator) -> String {
        match self {
            DurationPreset::Short => translator.get("gui.duration_short"),
            DurationPreset::Pomodoro => translator.get("gui.duration_pomodoro"),
            DurationPreset::Long => translator.get("gui.duration_long"),
            DurationPreset::DeepWork => translator.get("gui.duration_deep_work"),
            DurationPreset::Custom => translator.get("gui.duration_custom"),
        }
    }
}

pub struct StartSessionForm {
    pub selected_mode: FocusMode,
    pub selected_duration: DurationPreset,
    pub custom_minutes: u64,
}

impl Default for StartSessionForm {
    fn default() -> Self {
        Self {
            selected_mode: FocusMode::Prompting,
            selected_duration: DurationPreset::Pomodoro,
            custom_minutes: 25,
        }
    }
}

impl StartSessionForm {
    pub fn duration_minutes(&self) -> u64 {
        match self.selected_duration {
            DurationPreset::Custom => self.custom_minutes,
            preset => preset.minutes(),
        }
    }
}

pub fn render_session_control(
    ui: &mut Ui,
    controller: &mut SessionController,
    form: &mut StartSessionForm,
    translator: &Translator,
    theme: &Theme,
) {
    let status = controller.status().clone();

    theme.card_frame().show(ui, |ui| match &status {
        SessionStatus::Unknown | SessionStatus::DaemonUnavailable => {
            render_daemon_status(ui, &status, translator, theme);
        }
        SessionStatus::NoSession => {
            render_start_form(ui, controller, form, translator, theme);
        }
        SessionStatus::Active {
            remaining_seconds,
            mode,
        } => {
            render_active_session(
                ui,
                controller,
                *remaining_seconds,
                mode.as_ref(),
                false,
                translator,
                theme,
            );
        }
        SessionStatus::Paused {
            remaining_seconds,
            mode,
        } => {
            render_active_session(
                ui,
                controller,
                *remaining_seconds,
                mode.as_ref(),
                true,
                translator,
                theme,
            );
        }
    });
}

fn render_daemon_status(
    ui: &mut Ui,
    status: &SessionStatus,
    translator: &Translator,
    theme: &Theme,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(theme.spacing.md);

        let (icon, message) = match status {
            SessionStatus::DaemonUnavailable => ("⚠️", translator.get("gui.daemon_error")),
            _ => ("⏳", translator.get("gui.starting")),
        };

        ui.label(
            egui::RichText::new(icon)
                .size(theme.typography.heading)
                .color(theme.colors.warning),
        );

        ui.add_space(theme.spacing.sm);

        ui.label(
            egui::RichText::new(message)
                .size(theme.typography.body)
                .color(theme.colors.text_secondary),
        );

        ui.add_space(theme.spacing.md);
    });
}

fn render_start_form(
    ui: &mut Ui,
    controller: &mut SessionController,
    form: &mut StartSessionForm,
    translator: &Translator,
    theme: &Theme,
) {
    ui.label(
        egui::RichText::new(translator.get("gui.start_session"))
            .size(theme.typography.title)
            .color(theme.colors.text_primary)
            .strong(),
    );

    ui.add_space(theme.spacing.md);

    ui.label(
        egui::RichText::new(translator.get("command.status_mode"))
            .size(theme.typography.label)
            .color(theme.colors.text_secondary),
    );

    ui.add_space(theme.spacing.xs);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = theme.spacing.sm;

        let modes = [
            (
                FocusMode::Prompting,
                "Prompting",
                theme.colors.mode_prompting,
            ),
            (FocusMode::Review, "Review", theme.colors.mode_review),
            (
                FocusMode::Architecture,
                "Architecture",
                theme.colors.mode_architecture,
            ),
        ];

        for (mode, label, color) in modes {
            let is_selected = form.selected_mode == mode;

            let (bg_color, text_color, stroke) = if is_selected {
                (
                    color,
                    theme.colors.text_primary,
                    egui::Stroke::new(2.0, color),
                )
            } else {
                (
                    theme.colors.surface_elevated,
                    theme.colors.text_secondary,
                    egui::Stroke::new(1.0, theme.colors.border),
                )
            };

            let button = egui::Button::new(
                egui::RichText::new(label)
                    .size(theme.typography.body)
                    .color(text_color),
            )
            .fill(bg_color)
            .stroke(stroke)
            .rounding(Rounding::same(theme.rounding.md));

            if ui.add(button).clicked() {
                form.selected_mode = mode;
            }
        }
    });

    ui.add_space(theme.spacing.md);

    ui.label(
        egui::RichText::new(translator.get("command.status_duration"))
            .size(theme.typography.label)
            .color(theme.colors.text_secondary),
    );

    ui.add_space(theme.spacing.xs);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = theme.spacing.sm;

        for preset in DurationPreset::all() {
            let is_selected = form.selected_duration == *preset;
            let label = if *preset == DurationPreset::Custom {
                preset.label(translator)
            } else {
                format!("{} min", preset.minutes())
            };

            let (bg_color, text_color, stroke) = if is_selected {
                (
                    theme.colors.accent,
                    theme.colors.text_primary,
                    egui::Stroke::new(2.0, theme.colors.accent),
                )
            } else {
                (
                    theme.colors.surface_elevated,
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
                form.selected_duration = *preset;
            }
        }
    });

    if form.selected_duration == DurationPreset::Custom {
        ui.add_space(theme.spacing.sm);

        ui.horizontal(|ui| {
            let mut minutes_string = form.custom_minutes.to_string();
            let text_edit = egui::TextEdit::singleline(&mut minutes_string)
                .desired_width(60.0)
                .font(egui::TextStyle::Body);

            if ui.add(text_edit).changed() {
                if let Ok(value) = minutes_string.parse::<u64>() {
                    form.custom_minutes = value.clamp(1, 480);
                }
            }

            ui.label(
                egui::RichText::new("min")
                    .size(theme.typography.body)
                    .color(theme.colors.text_secondary),
            );
        });
    }

    ui.add_space(theme.spacing.lg);

    let start_button = egui::Button::new(
        egui::RichText::new(if controller.is_pending() {
            translator.get("gui.starting")
        } else {
            translator.get("gui.start_session")
        })
        .size(theme.typography.body)
        .color(egui::Color32::WHITE),
    )
    .fill(theme.colors.success)
    .rounding(Rounding::same(theme.rounding.md))
    .min_size(egui::vec2(ui.available_width(), 36.0));

    let button_enabled = !controller.is_pending();

    if ui.add_enabled(button_enabled, start_button).clicked() {
        controller.start_session(form.duration_minutes(), form.selected_mode.clone());
    }
}

fn render_active_session(
    ui: &mut Ui,
    controller: &mut SessionController,
    remaining_seconds: u64,
    mode: Option<&FocusMode>,
    paused: bool,
    translator: &Translator,
    theme: &Theme,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(translator.get("gui.session_active"))
                .size(theme.typography.title)
                .color(theme.colors.text_primary)
                .strong(),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(mode) = mode {
                let mode_str = mode.to_string();
                let mode_color = theme.colors.mode_color(&mode_str);

                let mode_frame = egui::Frame::none()
                    .fill(mode_color.linear_multiply(0.2))
                    .stroke(egui::Stroke::new(1.0, mode_color))
                    .rounding(Rounding::same(theme.rounding.sm))
                    .inner_margin(egui::Margin::symmetric(theme.spacing.sm, theme.spacing.xs));

                mode_frame.show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&mode_str)
                            .size(theme.typography.label)
                            .color(mode_color),
                    );
                });
            }

            if paused {
                ui.label(
                    egui::RichText::new("⏸️")
                        .size(theme.typography.title)
                        .color(theme.colors.warning),
                );
            }
        });
    });

    ui.add_space(theme.spacing.md);

    ui.vertical_centered(|ui| {
        let minutes = remaining_seconds / 60;
        let seconds = remaining_seconds % 60;
        let time_display = format!("{:02}:{:02}", minutes, seconds);

        ui.label(
            egui::RichText::new(time_display)
                .size(48.0)
                .color(if paused {
                    theme.colors.warning
                } else {
                    theme.colors.accent
                })
                .strong()
                .monospace(),
        );

        ui.label(
            egui::RichText::new(translator.get("gui.remaining_time"))
                .size(theme.typography.label)
                .color(theme.colors.text_muted),
        );
    });

    ui.add_space(theme.spacing.lg);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme.spacing.sm;

        let button_width = (ui.available_width() - theme.spacing.sm) / 2.0;

        if paused {
            let resume_button = egui::Button::new(
                egui::RichText::new("▶️ Reprendre")
                    .size(theme.typography.body)
                    .color(egui::Color32::WHITE),
            )
            .fill(theme.colors.success)
            .rounding(Rounding::same(theme.rounding.md))
            .min_size(egui::vec2(button_width, 36.0));

            if ui
                .add_enabled(!controller.is_pending(), resume_button)
                .clicked()
            {
                controller.resume_session();
            }
        } else {
            let pause_button = egui::Button::new(
                egui::RichText::new("⏸️ Pause")
                    .size(theme.typography.body)
                    .color(theme.colors.text_primary),
            )
            .fill(theme.colors.surface_elevated)
            .stroke(egui::Stroke::new(1.0, theme.colors.border))
            .rounding(Rounding::same(theme.rounding.md))
            .min_size(egui::vec2(button_width, 36.0));

            if ui
                .add_enabled(!controller.is_pending(), pause_button)
                .clicked()
            {
                controller.pause_session();
            }
        }

        let stop_button = egui::Button::new(
            egui::RichText::new("⏹️ Arrêter")
                .size(theme.typography.body)
                .color(egui::Color32::WHITE),
        )
        .fill(theme.colors.error)
        .rounding(Rounding::same(theme.rounding.md))
        .min_size(egui::vec2(button_width, 36.0));

        if ui
            .add_enabled(!controller.is_pending(), stop_button)
            .clicked()
        {
            controller.stop_session();
        }
    });
}
