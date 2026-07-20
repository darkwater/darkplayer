use std::sync::{OnceLock, mpsc};

pub type Receiver = mpsc::Receiver<Message>;

static CHANNEL: OnceLock<(mpsc::Sender<Message>, egui::Context)> = OnceLock::new();

pub fn init(egui_ctx: egui::Context) -> mpsc::Receiver<Message> {
    let (tx, rx) = mpsc::channel();
    CHANNEL.set((tx, egui_ctx)).expect("Failed to set CHANNEL");
    rx
}

#[derive(Debug, Clone)]
pub enum Message {
    MpvEvent(crate::mpv::event::MpvEvent),
    MpvCommand(String, Vec<String>),

    SeekBackward,
    SeekForward,
    Screenshot,
    DpadMenu,
    TogglePause,
}

impl Message {
    pub fn send(self) {
        let (channel, egui_ctx) = CHANNEL.get().expect("CHANNEL wasn't initialized");

        if let Err(e) = channel.send(self) {
            log::error!("CHANNEL closed, failed to send {:?}", e.0);
            return;
        }

        egui_ctx.request_repaint();
    }
}
