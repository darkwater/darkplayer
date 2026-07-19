use egui::{Modifiers, Pos2, Stroke, vec2};
use egui_material_icons::MaterialIcon;

use crate::{
    message::Message,
    ui::widgets::{FadeTimer, Fadeable},
};

#[derive(Default)]
pub struct Dpad {
    pub up: Option<DpadAction>,
    pub down: Option<DpadAction>,
    pub left: Option<DpadAction>,
    pub right: Option<DpadAction>,
    pub enter: Option<DpadAction>,
    pub back: Option<DpadAction>,
}

pub struct DpadAction {
    pub icon: Option<MaterialIcon>,
    pub message: Message,
}

impl DpadAction {
    pub fn new(icon: MaterialIcon, message: Message) -> Self {
        Self { icon: Some(icon), message }
    }

    pub fn no_icon(message: Message) -> Self {
        Self { icon: None, message }
    }
}

impl Fadeable for Dpad {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let radius = ui.content_rect().width().min(ui.content_rect().height()) * 0.1;
        let center = ui.content_rect().right_center() + vec2(radius * -1.5, 0.0);

        let fg_stroke = Stroke::new(radius * 0.04, egui::Color32::WHITE);
        let shadow_stroke = Stroke::new(radius * 0.04 + 2., egui::Color32::from_black_alpha(127));

        let draw_icon = |icon: MaterialIcon, position: Pos2| {
            ui.painter().text(
                position + vec2(1.0, 1.0),
                egui::Align2::CENTER_CENTER,
                <&str>::from(icon),
                egui::FontId::new(radius * 0.5, icon.font_family()),
                egui::Color32::from_black_alpha(127),
            );

            ui.painter().text(
                position,
                egui::Align2::CENTER_CENTER,
                <&str>::from(icon),
                egui::FontId::new(radius * 0.5, icon.font_family()),
                egui::Color32::WHITE,
            );
        };

        let draw_circle = |position: Pos2, radius: f32| {
            ui.painter()
                .circle_stroke(position, radius - 1., shadow_stroke);
            ui.painter().circle_stroke(position, radius, fg_stroke);
        };

        draw_circle(center, radius);

        let ring_icon_distance = 0.7;

        let back_button_radius = (ring_icon_distance - 1f32).abs();
        let back_button_offset =
            vec2(radius * -ring_icon_distance, radius * ring_icon_distance * 2.);

        draw_circle(center + back_button_offset, radius * back_button_radius);

        for (action, offset) in [
            (self.up.as_ref(), vec2(0.0, radius * -ring_icon_distance)),
            (self.down.as_ref(), vec2(0.0, radius * ring_icon_distance)),
            (self.left.as_ref(), vec2(radius * -ring_icon_distance, 0.0)),
            (self.right.as_ref(), vec2(radius * ring_icon_distance, 0.0)),
            (self.enter.as_ref(), vec2(0.0, 0.0)),
            (self.back.as_ref(), back_button_offset + vec2(0., radius * -0.016)),
        ] {
            if let Some(action) = action
                && let Some(icon) = action.icon
            {
                draw_icon(icon, center + offset);
            }
        }
    }

    fn logic(self, ui: &mut egui::Ui, fade_timer: Option<&mut FadeTimer>) {
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

            if let Some(timer) = fade_timer {
                timer.reset();
            }
        }
    }
}
