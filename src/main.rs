mod mpv;
mod ui;

use mpv::player::MpvPlayer;

use crate::ui::dpad::Dpad;

fn main() {
    pretty_env_logger::init();

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "darkplayer",
        native_options,
        Box::new(|cc| Ok(Box::new(DarkPlayer::new(cc)))),
    )
    .expect("Failed to run eframe");
}

struct DarkPlayer {
    player: MpvPlayer,
    texture_id: Option<egui::TextureId>,
    dpad: Dpad,
}

impl DarkPlayer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone().expect("glow context not available");
        let get_proc_address = cc
            .get_proc_address
            .clone()
            .expect("get_proc_address not available");

        let initial_size = (1280, 720);
        let player = MpvPlayer::new(gl, get_proc_address, &cc.egui_ctx, initial_size);

        if let Some(path) = std::env::args().nth(1) {
            player.load_file(&path);
        }

        Self {
            player,
            texture_id: None,
            dpad: Dpad::default(),
        }
    }

    fn render_player(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Register the texture on first frame
        let texture_id = *self
            .texture_id
            .get_or_insert_with(|| frame.register_native_glow_texture(self.player.texture()));

        let rect = ui.viewport_rect();

        self.player
            .resize_if_needed(rect.width() as i32, rect.height() as i32);

        self.player.render_frame_if_ready();

        ui.painter().image(
            texture_id,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0)),
            egui::Color32::WHITE,
        );
    }
}

impl eframe::App for DarkPlayer {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.render_player(ui, frame);
        self.dpad.render(ui);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.player.destroy();
    }
}
