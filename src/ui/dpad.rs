use core::time::Duration;
use std::time::Instant;

use egui::{vec2, Modifiers, Stroke};
use egui_material_icons::MaterialIcon;

use crate::message::Message;

#[derive(Default)]
pub struct Dpad<'a> {
    pub up: Option<DpadAction>,
    pub down: Option<DpadAction>,
    pub left: Option<DpadAction>,
    pub right: Option<DpadAction>,
    pub enter: Option<DpadAction>,
    pub back: Option<DpadAction>,

    pub fade_timer: Option<&'a mut DpadTimer>,
}

pub struct DpadAction {
    pub icon: MaterialIcon,
    pub message: Message,
}

const FADE_OUT_TIME: Duration = Duration::from_secs(2);

#[derive(Default)]
pub struct DpadTimer {
    last_input: Option<Instant>,
}

impl DpadAction {
    pub fn new(icon: MaterialIcon, message: Message) -> Self {
        Self { icon, message }
    }
}

impl Dpad<'_> {
    pub fn render_only(&self, ui: &mut egui::Ui) {
        let visibility = if let Some(timer) = &self.fade_timer {
            if let Some(last_input) = timer.last_input {
                let elapsed = last_input.elapsed();
                if elapsed >= FADE_OUT_TIME {
                    return;
                } else {
                    ui.request_repaint();
                    1.0 - (elapsed.as_secs_f32() / FADE_OUT_TIME.as_secs_f32())
                }
            } else {
                return;
            }
        } else {
            1.0
        };

        ui.scope(|ui| {
            ui.set_opacity(visibility);

            let radius = ui.content_rect().width().min(ui.content_rect().height()) * 0.1;
            let center = ui.content_rect().right_center() + vec2(radius * -1.5, 0.0);

            let stroke = Stroke::new(radius * 0.04, egui::Color32::WHITE);
            let shadow_stroke =
                Stroke::new(radius * 0.04 + 2., egui::Color32::from_black_alpha(127));

            ui.painter()
                .circle_stroke(center, radius - 1., shadow_stroke);

            ui.painter().circle_stroke(center, radius, stroke);

            let ring_icon_distance = 0.7;
            for (action, offset) in [
                (self.up.as_ref(), vec2(0.0, radius * -ring_icon_distance)),
                (self.down.as_ref(), vec2(0.0, radius * ring_icon_distance)),
                (self.left.as_ref(), vec2(radius * -ring_icon_distance, 0.0)),
                (self.right.as_ref(), vec2(radius * ring_icon_distance, 0.0)),
                (self.enter.as_ref(), vec2(0.0, 0.0)),
                (
                    self.back.as_ref(),
                    vec2(radius * -ring_icon_distance, radius * ring_icon_distance * 2.),
                ),
            ] {
                if let Some(action) = action {
                    ui.painter().text(
                        center + offset + vec2(1.0, 1.0),
                        egui::Align2::CENTER_CENTER,
                        <&str>::from(action.icon),
                        egui::FontId::new(radius * 0.5, action.icon.font_family()),
                        egui::Color32::from_black_alpha(127),
                    );

                    ui.painter().text(
                        center + offset,
                        egui::Align2::CENTER_CENTER,
                        <&str>::from(action.icon),
                        egui::FontId::new(radius * 0.5, action.icon.font_family()),
                        egui::Color32::WHITE,
                    );
                }
            }
        });
    }

    pub fn handle_input(self, ui: &mut egui::Ui) {
        let action = ui.input_mut(|i| {
            [
                (egui::Key::ArrowUp, self.up),
                (egui::Key::ArrowDown, self.down),
                (egui::Key::ArrowLeft, self.left),
                (egui::Key::ArrowRight, self.right),
                (egui::Key::Enter, self.enter),
                (egui::Key::Escape, self.back),
            ]
            .into_iter()
            .find(|(key, action)| action.is_some() && i.consume_key(Modifiers::NONE, *key))
            .map(|(_, action)| action.unwrap())
        });

        if let Some(action) = action {
            action.message.send();

            if let Some(timer) = self.fade_timer {
                timer.last_input = Some(Instant::now());
            }
        }
    }

    pub fn render(self, ui: &mut egui::Ui) {
        self.render_only(ui);
        self.handle_input(ui);
    }
}
