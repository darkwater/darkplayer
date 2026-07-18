use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use eframe::glow::{self, HasContext, PixelUnpackData};
use libmpv2::render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType};
use libmpv2::Mpv;

use crate::message::Message;
use crate::mpv::event::MpvEvent;

pub struct MpvPlayer {
    // IMPORTANT: render_ctx must be declared before mpv so it is dropped first.
    // RenderContext borrows from Mpv internally (we transmuted the lifetime to 'static).
    render_ctx: RenderContext<'static>,
    mpv: Mpv,
    fbo: glow::Framebuffer,
    texture: glow::Texture,
    fbo_size: (i32, i32),
    new_frame: Arc<AtomicBool>,
    gl: Arc<glow::Context>,
}

/// Wrapper type to pass as OpenGL context to libmpv2.
struct GlProcAddressCtx {
    get_proc_address: Arc<dyn Fn(&CStr) -> *const c_void + Send + Sync>,
}

/// Function that libmpv2 calls to resolve OpenGL function pointers.
fn mpv_get_proc_address(ctx: &GlProcAddressCtx, name: &str) -> *mut c_void {
    let cstr = std::ffi::CString::new(name).unwrap();
    (ctx.get_proc_address)(&cstr) as *mut c_void
}

unsafe impl Send for MpvPlayer {}

impl MpvPlayer {
    pub fn new(
        gl: Arc<glow::Context>,
        get_proc_address: Arc<dyn Fn(&CStr) -> *const c_void + Send + Sync>,
        egui_ctx: &egui::Context,
        initial_size: (i32, i32),
    ) -> Self {
        let mpv = Mpv::new().expect("Failed to create Mpv instance");
        mpv.set_property("vo", "libmpv")
            .expect("Failed to set vo=libmpv");

        // Enable mpv log messages forwarded to the log crate
        unsafe {
            let level = CString::new("v").unwrap(); // "v" = verbose
            libmpv2_sys::mpv_request_log_messages(mpv.ctx.as_ptr(), level.as_ptr());
        }

        // Spawn event loop thread to forward mpv log messages
        let ctx_ptr = mpv.ctx.as_ptr() as usize; // usize is Send
        std::thread::spawn(move || {
            let ctx = ctx_ptr as *mut libmpv2_sys::mpv_handle;
            loop {
                let event = unsafe { &*libmpv2_sys::mpv_wait_event(ctx, -1.0) };
                let event = MpvEvent::from(event);
                let shutdown = matches!(event, MpvEvent::Shutdown);
                Message::MpvEvent(event).send();
                if shutdown {
                    break;
                }
            }
        });

        let opengl_init = OpenGLInitParams {
            get_proc_address: mpv_get_proc_address,
            ctx: GlProcAddressCtx {
                get_proc_address: get_proc_address.clone(),
            },
        };

        let render_ctx = mpv
            .create_render_context(vec![
                RenderParam::ApiType(RenderParamApiType::OpenGl),
                RenderParam::InitParams(opengl_init),
            ])
            .expect("Failed to create RenderContext");

        // Transmute to 'static lifetime - sound because MpvPlayer owns both mpv and render_ctx
        let mut render_ctx: RenderContext<'static> = unsafe { std::mem::transmute(render_ctx) };

        let new_frame = Arc::new(AtomicBool::new(false));

        let flag = new_frame.clone();
        let ctx = egui_ctx.clone();
        render_ctx.set_update_callback(move || {
            flag.store(true, Ordering::Relaxed);
            ctx.request_repaint();
        });

        let (fbo, texture) = unsafe { create_fbo(&gl, initial_size.0, initial_size.1) };

        Self {
            mpv,
            render_ctx,
            fbo,
            texture,
            fbo_size: initial_size,
            new_frame,
            gl,
        }
    }

    pub fn texture(&self) -> glow::Texture {
        self.texture
    }

    pub fn resize_if_needed(&mut self, width: i32, height: i32) {
        if self.fbo_size != (width, height) && width > 0 && height > 0 {
            log::debug!("Resizing FBO texture to {}x{}", width, height);

            unsafe { resize_fbo_texture(&self.gl, self.texture, width, height) };
            self.fbo_size = (width, height);
        }
    }

    pub fn render_frame_if_ready(&mut self) -> bool {
        if !self.new_frame.swap(false, Ordering::Relaxed) {
            return false;
        }

        let (w, h) = self.fbo_size;
        let fbo_raw = unsafe {
            std::mem::transmute::<glow::Framebuffer, std::num::NonZeroU32>(self.fbo).get() as i32
        };

        self.render_ctx
            .render::<GlProcAddressCtx>(fbo_raw, w, h, true)
            .expect("mpv render failed");

        true
    }

    pub fn load_file(&self, path: &str) {
        self.mpv
            .command("loadfile", &[path])
            .expect("Failed to load file");
    }

    pub fn destroy(&self) {
        unsafe {
            destroy_fbo(&self.gl, self.fbo, self.texture);
        }
    }

    pub fn command(&self, name: &str, args: &[&str]) -> libmpv2::Result<()> {
        self.mpv.command(name, args)
    }

    pub fn command_async(&self, name: &str, args: &[&str]) -> libmpv2::Result<()> {
        let userdata = 0;

        // copied from self.mpv.command() but with added userdata argument

        let mut cstr_args: Vec<CString> = Vec::with_capacity(args.len() + 1);
        cstr_args.push(CString::new(name)?);

        for arg in args {
            cstr_args.push(CString::new(*arg)?);
        }

        let mut ptrs: Vec<_> = cstr_args.iter().map(|cstr| cstr.as_ptr()).collect();
        ptrs.push(std::ptr::null());

        match unsafe {
            libmpv2_sys::mpv_command_async(self.mpv.ctx.as_ptr(), userdata, ptrs.as_mut_ptr())
        } {
            0 => Ok(()),
            err_code => Err(libmpv2::Error::Raw(err_code)),
        }
    }
}

/// Creates an FBO with an RGBA8 texture attachment of the given dimensions.
pub unsafe fn create_fbo(
    gl: &glow::Context,
    width: i32,
    height: i32,
) -> (glow::Framebuffer, glow::Texture) {
    let texture = gl.create_texture().unwrap();
    gl.bind_texture(glow::TEXTURE_2D, Some(texture));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA8 as i32,
        width,
        height,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        PixelUnpackData::Slice(None),
    );
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
    gl.bind_texture(glow::TEXTURE_2D, None);

    let fbo = gl.create_framebuffer().unwrap();
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::TEXTURE_2D,
        Some(texture),
        0,
    );
    gl.bind_framebuffer(glow::FRAMEBUFFER, None);

    (fbo, texture)
}

/// Resize the FBO texture in-place (same handle, new dimensions).
pub unsafe fn resize_fbo_texture(
    gl: &glow::Context,
    texture: glow::Texture,
    width: i32,
    height: i32,
) {
    gl.bind_texture(glow::TEXTURE_2D, Some(texture));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA8 as i32,
        width,
        height,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        PixelUnpackData::Slice(None),
    );
    gl.bind_texture(glow::TEXTURE_2D, None);
}

/// Deletes FBO and texture.
pub unsafe fn destroy_fbo(gl: &glow::Context, fbo: glow::Framebuffer, texture: glow::Texture) {
    gl.delete_framebuffer(fbo);
    gl.delete_texture(texture);
}
