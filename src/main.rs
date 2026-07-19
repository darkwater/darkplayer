mod hermes;
mod message;
mod mpv;
mod ui;

use mpv::player::MpvPlayer;

use crate::{
    hermes::Hermes,
    message::Message,
    mpv::event::{MpvEvent, Properties},
    ui::{
        pages::{Page, empty::EmptyPage},
        widgets::FadeTimer,
    },
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

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

    state: AppState,
}

#[derive(Default)]
struct AppState {
    last_seek: FadeTimer,
    properties: Properties,
}

impl Darkplayer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let inbox = message::init(cc.egui_ctx.clone());

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

        if let Some(path) = std::env::args().nth(1) {
            player.load_file(&path);
        } else {
            player.load_file("~/yofukashi");
        }

        Self {
            inbox,
            player,
            texture_id: None,
            shutting_down: false,
            page: Box::new(EmptyPage::default()),
            hermes: Hermes::init(),
            state: AppState::default(),
        }
    }

    fn handle_event(&mut self, ctx: &egui::Context, msg: Message) {
        match msg {
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
                log::warn!("mpv event: {:?}", event);
            }
            Message::SeekBackward => {
                self.state.last_seek.reset();
                self.player
                    .command("seek", &["-5", "relative"])
                    .expect("Failed to seek backward")
            }
            Message::SeekForward => {
                self.state.last_seek.reset();
                self.player
                    .command("seek", &["5", "relative"])
                    .expect("Failed to seek forward")
            }
            Message::Screenshot => self
                .player
                .command_async("screenshot-raw", &["subtitles", "rgba"])
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
        self.render_player(ui, frame);
        self.page.render(&self.state, ui);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.player.destroy();
    }
}
