use egui_material_icons::icons;

use crate::{
    message::Message,
    ui::{
        dpad::{Dpad, DpadAction, DpadTimer},
        pages::Page,
    },
    AppState,
};

#[derive(Default)]
pub struct EmptyPage {
    dpad_timer: DpadTimer,
}

impl Page for EmptyPage {
    fn render(&mut self, _app: &AppState, ui: &mut egui::Ui) {
        Dpad {
            left: Some(DpadAction::new(icons::ICON_FAST_REWIND, Message::SeekBackward)),
            right: Some(DpadAction::new(icons::ICON_FAST_FORWARD, Message::SeekForward)),
            up: Some(DpadAction::new(icons::ICON_PHOTO_CAMERA, Message::Screenshot)),
            down: Some(DpadAction::new(icons::ICON_MENU, Message::DpadMenu)),
            enter: Some(DpadAction::new(icons::ICON_PLAY_PAUSE, Message::TogglePause)),
            back: Some(DpadAction::new(icons::ICON_PLAY_PAUSE, Message::TogglePause)),

            fade_timer: Some(&mut self.dpad_timer),
        }
        .render(ui);
    }
}
