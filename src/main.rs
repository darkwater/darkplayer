use eframe::egui;
use egui::ViewportCommand;
use egui_demo_lib::DemoWindows;
use libmpv2::{
    render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType},
    Mpv,
};
use std::ffi::CString;
use std::{
    ffi::{c_void, CStr},
    sync::Arc,
};

struct MyApp {
    mpv: Arc<Mpv>,
    render_context: RenderContext,
    demo: DemoWindows,
    shutdown: bool,
}

struct GlContext<'a> {
    get_proc_address: &'a dyn Fn(&CStr) -> *const c_void,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut mpv = Mpv::with_initializer(|init| {
            init.set_property("vo", "libmpv")?;
            init.set_property("ao", "pipewire")?;
            init.set_property("video-timing-offset", 0)?;
            init.set_property("loop", true)?;
            Ok(())
        })
        .unwrap();

        let mut render_context = RenderContext::new(
            unsafe { mpv.ctx.as_mut() },
            vec![
                RenderParam::ApiType(RenderParamApiType::OpenGl),
                RenderParam::InitParams(OpenGLInitParams {
                    ctx: GlContext {
                        get_proc_address: cc.get_proc_address.unwrap(),
                    },
                    get_proc_address: |ctx, name| {
                        (ctx.get_proc_address)(&CString::new(name).unwrap()) as *mut _
                    },
                }),
            ],
        )
        .expect("Failed creating render context");

        mpv.event_context_mut().disable_deprecated_events().unwrap();

        let mpv = Arc::new(mpv);

        let ctx = cc.egui_ctx.clone();

        render_context.set_update_callback(move || {
            ctx.request_repaint();
        });

        mpv.command("loadfile", &["/tmp/output.mp4"]).unwrap();

        let demo = DemoWindows::default();

        Self {
            mpv,
            render_context,
            demo,
            shutdown: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Some(Ok(ev)) = Arc::get_mut(&mut self.mpv)
            .unwrap()
            .event_context_mut()
            .wait_event(0.)
        {
            println!("{ev:?}");

            use libmpv2::events::Event;
            if let Event::Shutdown = ev {
                self.shutdown = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }

        let screen_rect = ctx.screen_rect();
        self.render_context
            .render::<GlContext<'static>>(
                0,
                screen_rect.width() as i32,
                screen_rect.height() as i32,
                true,
            )
            .unwrap();

        let mut time_pos = self.mpv.get_property::<f64>("time-pos").unwrap_or_default();
        let duration = self.mpv.get_property::<f64>("duration").unwrap_or_default();

        egui::Window::new("Seek position").show(ctx, |ui| {
            let res = ui.add(egui::Slider::new(&mut time_pos, 0.0..=duration).text("Time Pos"));

            if res.changed() {
                self.mpv
                    .set_property("time-pos", time_pos)
                    .expect("Failed setting time-pos");
            }
        });

        self.demo.ui(ctx);

        if !self.shutdown && ctx.input(|i| i.viewport().close_requested()) {
            self.mpv.command("quit", &[]).unwrap();
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Darkplayer", options, Box::new(|cc| Ok(Box::new(MyApp::new(cc)))))
}
