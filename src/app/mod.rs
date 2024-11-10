use egui::{Context, ViewportCommand};
use libmpv2::{events::Event, render::RenderContext, Mpv};

use crate::GlContext;

pub struct MyApp {
    mpv: Mpv,
    render_context: RenderContext,
    shutdown: bool,
}

impl MyApp {
    pub fn new(mpv: Mpv, render_context: RenderContext, cc: &eframe::CreationContext) -> Self {
        cc.egui_ctx.set_zoom_factor(2.0);

        Self { mpv, render_context, shutdown: false }
    }

    fn mpv_properties(&self) -> MpvProperties {
        MpvProperties {
            idle_active: self.mpv.get_property("idle-active").unwrap_or_default(),
            time_pos: self.mpv.get_property("time-pos").unwrap_or_default(),
            duration: self.mpv.get_property("duration").unwrap_or_default(),
        }
    }

    fn render_mpv(&self, ctx: &Context) {
        let screen_rect = ctx.screen_rect() * ctx.native_pixels_per_point().unwrap_or(1.);
        self.render_context
            .render::<GlContext<'static>>(
                0,
                screen_rect.width() as i32,
                screen_rect.height() as i32,
                true,
            )
            .unwrap();
    }
}

struct MpvProperties {
    idle_active: bool,
    time_pos: f64,
    duration: f64,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        while let Some(Ok(ev)) = self.mpv.event_context_mut().wait_event(0.) {
            if let Event::Shutdown = ev {
                self.shutdown = true;
                ctx.request_repaint();
            }
        }

        self.render_mpv(ctx);

        let mut props = self.mpv_properties();

        if self.shutdown {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label("Shutting down...");
                });
            });
        } else if props.idle_active {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Core idle");
                if ui.button("Load file").clicked() {
                    self.mpv
                        .command("loadfile", &["/home/dark/oshi-no-ko"])
                        .unwrap();
                }
            });

            if self.shutdown {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        } else {
            egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
                ui.spacing_mut().slider_width = ui.available_width();

                let res = ui.add(
                    egui::Slider::new(&mut props.time_pos, 0.0..=props.duration).show_value(false),
                );

                if res.changed() {
                    self.mpv
                        .set_property("time-pos", props.time_pos)
                        .expect("Failed setting time-pos");
                }
            });
        }

        if !self.shutdown && ctx.input(|i| i.viewport().close_requested()) {
            self.mpv.command("quit", &[]).unwrap();
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
    }
}
