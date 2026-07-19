use core::time::Duration;

use egui_material_icons::icons;

use crate::{
    AppState,
    message::Message,
    ui::{
        pages::Page,
        widgets::{
            FadeTimer, Faded,
            dpad::{Dpad, DpadAction},
            seek_bar::SeekBar,
        },
    },
};

#[derive(Default)]
pub struct EmptyPage {
    dpad_timer: FadeTimer,
}

impl Page for EmptyPage {
    fn render(&mut self, app: &AppState, ui: &mut egui::Ui) {
        Faded::new(
            Dpad {
                left: Some(DpadAction::new(icons::ICON_FAST_REWIND, Message::SeekBackward)),
                right: Some(DpadAction::new(icons::ICON_FAST_FORWARD, Message::SeekForward)),
                up: Some(DpadAction::new(icons::ICON_PHOTO_CAMERA, Message::Screenshot)),
                down: Some(DpadAction::new(icons::ICON_MENU, Message::DpadMenu)),

                enter: Some(DpadAction::no_icon(Message::TogglePause)),
                back: Some(DpadAction::new(icons::ICON_PLAY_PAUSE, Message::TogglePause)),
            },
            &mut self.dpad_timer,
            Duration::from_secs(2),
        )
        .render(ui);

        Faded::new(
            SeekBar {
                time_pos: app.properties.time_pos,
                duration: app.properties.duration,
            },
            &mut app.last_seek.clone(),
            Duration::from_secs(2),
        )
        .render(ui);
    }
}
