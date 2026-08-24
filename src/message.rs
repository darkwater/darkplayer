use std::sync::{OnceLock, mpsc};

use crate::{AppState, ui::pages::Page};

pub type Receiver = mpsc::Receiver<Message>;

static CHANNEL: OnceLock<(mpsc::Sender<Message>, egui::Context)> = OnceLock::new();

pub fn init(egui_ctx: egui::Context) -> mpsc::Receiver<Message> {
    let (tx, rx) = mpsc::channel();
    CHANNEL.set((tx, egui_ctx)).expect("Failed to set CHANNEL");
    rx
}

pub enum Message {
    SetPage(Box<dyn Page>),
    MutateState(Box<dyn FnOnce(&mut AppState) + Send>),

    MpvEvent(crate::mpv::event::MpvEvent),
    MpvCommand(String, Vec<String>),

    SeekBackward,
    SeekForward,
    Screenshot,
    DpadMenu,
    TogglePause,
}

impl Message {
    pub fn mutate_state(f: impl FnOnce(&mut AppState) + Send + 'static) -> Self {
        Message::MutateState(Box::new(f))
    }

    pub fn send(self) {
        let (channel, egui_ctx) = CHANNEL.get().expect("CHANNEL wasn't initialized");

        if let Err(_) = channel.send(self) {
            log::error!("CHANNEL closed, failed to send message");
            return;
        }

        egui_ctx.request_repaint();
    }
}
