mod db;
mod hermes;
mod message;
mod mpv;
mod ui;
mod utils;

use std::time::Instant;

use mpv::player::MpvPlayer;
use serde::{Deserialize, Serialize};

use crate::{
    db::Database,
    hermes::Hermes,
    message::Message,
    mpv::event::{MpvEvent, Properties},
    ui::{
        pages::{Page, player::PlayerPage},
        widgets::{FadeTimer, frame_history::FrameHistory},
    },
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    tokio::task::block_in_place(|| {
        eframe::run_native(
            "darkplayer",
            native_options,
            Box::new(|cc| Ok(Box::new(Darkplayer::new(cc)))),
        )
        .expect("Failed to run eframe");
    })
}

struct Darkplayer {
    inbox: message::Receiver,
    player: MpvPlayer,
    texture_id: Option<egui::TextureId>,
    shutting_down: bool,
    page: Box<dyn Page>,
    hermes: anyhow::Result<Hermes>,
    frame_history: FrameHistory,
    last_event: Instant,

    state: AppState,
}

#[derive(Default, Serialize, Deserialize)]
struct AppState {
    #[serde(skip)]
    last_seek: FadeTimer,
    #[serde(skip)]
    properties: Properties,

    db: Database,
}

impl Darkplayer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let inbox = message::init(cc.egui_ctx.clone());

        cc.egui_ctx.all_styles_mut(|s| *s = ui::style::style());

        egui_material_icons::initialize(&cc.egui_ctx);

        let gl = cc.gl.clone().expect("glow context not available");
        let get_proc_address = cc
            .get_proc_address
            .clone()
            .expect("get_proc_address not available");

        let initial_size = (1280, 720);
        let player = MpvPlayer::new(gl, get_proc_address, &cc.egui_ctx, initial_size);

        player
            .batch_observe_properties(&[
                ("time-pos", libmpv2::Format::Double),
                ("duration", libmpv2::Format::Double),
                // ("playlist", libmpv2::Format::Node),
                // ("track-list", libmpv2::Format::Node),
                // ("chapter-list", libmpv2::Format::Node),
                // ("metadata", libmpv2::Format::Node),
            ])
            .unwrap();

        // if let Err(libmpv2::Error::Raw(d)) = player.mpv().set_property("display-fps-override", 165)
        // {
        //     eprintln!("Failed to set display-fps: {}", libmpv2_sys::mpv_error_str(d));
        // }
        // player
        //     .command("set", &["video-sync", "display-resample"])
        //     .unwrap();
        // player.command("set", &["interpolation", "yes"]).unwrap();

        let state = cc
            .storage
            .and_then(|s| {
                let start = Instant::now();
                let res = eframe::get_value::<AppState>(s, eframe::APP_KEY);
                log::debug!("Loaded state in {:?}", start.elapsed());
                res
            })
            .unwrap_or_default();

        if let Some(path) = std::env::args().nth(1) {
            player.load_file(&path);
        }

        let existing = state.db.index.keys().cloned().collect();
        std::thread::spawn(move || db::indexing::index("/home/dark/anime/".into(), existing));

        Self {
            inbox,
            player,
            texture_id: None,
            shutting_down: false,
            page: Box::new(PlayerPage::default()),
            hermes: Hermes::init(),
            frame_history: FrameHistory::default(),
            last_event: Instant::now(),

            state,
        }
    }

    fn handle_event(&mut self, ctx: &egui::Context, msg: Message) {
        match msg {
            Message::SetPage(page) => {
                self.page = page;
            }
            Message::MutateState(f) => {
                f(&mut self.state);
            }
            Message::MpvEvent(MpvEvent::Shutdown) => {
                log::info!("mpv shutdown event received, sending close to eframe");
                self.shutting_down = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Message::MpvEvent(MpvEvent::LogMessage { prefix, msg, level }) => {
                log::log!(target: &prefix, level, "{msg}");
            }
            Message::MpvEvent(MpvEvent::PropertyChange(change)) => {
                self.state.properties.apply(change);
            }
            Message::MpvEvent(MpvEvent::CommandReplyScreenshot { width, height, stride, data }) => {
                match &self.hermes {
                    Ok(hermes) => {
                        let hermes = hermes.clone();
                        tokio::spawn(async move {
                            if let Err(e) = hermes.send_image(data, width, height, stride).await {
                                log::error!("Failed to send screenshot: {e:?}");
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Can't send screenshot, hermes not initialized: {e}");
                    }
                }
            }
            Message::MpvEvent(event) => {
                // TODO:
                log::warn!("since last event: {:?}", self.last_event.elapsed());
                log::warn!("mpv event: {:?}", event);
                self.last_event = Instant::now();
            }
            Message::MpvCommand(cmd, args) => {
                if let Err(libmpv2::Error::Raw(d)) = self
                    .player
                    .command(&cmd, &args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                {
                    log::error!("Failed to send command to mpv: {}", libmpv2_sys::mpv_error_str(d));
                }
            }
            Message::SeekBackward => {
                self.state.last_seek.reset();
                self.player
                    .command("seek", &["-5", "relative+keyframes"])
                    .expect("Failed to seek backward")
            }
            Message::SeekForward => {
                self.state.last_seek.reset();
                self.player
                    .command("seek", &["5", "relative+keyframes"])
                    .expect("Failed to seek forward")
            }
            Message::Screenshot => self
                .player
                .command("screenshot-raw", &["subtitles", "rgba"])
                .expect("Failed to take screenshot"),
            Message::DpadMenu => {}
            Message::TogglePause => {
                self.player
                    .command("cycle", &["pause"])
                    .expect("Failed to toggle pause");
            }
        }
    }

    fn render_player(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Register the texture on first frame
        let texture_id = *self
            .texture_id
            .get_or_insert_with(|| frame.register_native_glow_texture(self.player.texture()));

        let rect = ui.viewport_rect();

        self.player.resize_if_needed(
            (rect.width() * ui.pixels_per_point()) as i32,
            (rect.height() * ui.pixels_per_point()) as i32,
        );

        self.player.render_frame_if_ready();

        ui.painter().image(
            texture_id,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
            egui::Color32::WHITE,
        );
    }
}

impl eframe::App for Darkplayer {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.inbox.try_recv() {
            self.handle_event(ctx, msg);
        }

        if ctx.input(|i| i.viewport().close_requested()) && !self.shutting_down {
            log::info!("Close requested, sending quit to mpv");

            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);

            self.player
                .command("quit", &[])
                .expect("Failed to quit mpv");
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.frame_history
            .on_new_frame(ui.input(|i| i.time), frame.info().cpu_usage);

        // scale the ui such that we can pretend the window is always 1920x1080
        ui.set_zoom_factor(ui.content_rect().width() * ui.zoom_factor() / 1920.0);

        self.render_player(ui, frame);
        self.page.render(&self.state, ui);
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        let start = Instant::now();
        eframe::set_value(_storage, eframe::APP_KEY, &self.state);
        log::debug!("Saved state in {:?}", start.elapsed());
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.player.destroy();
    }
}
