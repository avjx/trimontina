mod error;
mod surface;

pub use error::PlayerError;
pub use surface::VideoSurface;

use std::{
    ffi::{CStr, CString, c_void},
    path::Path,
    sync::Arc,
};

use eframe::egui;
use libmpv2::{
    Mpv,
    render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType},
};

type ProcAddressLoader = Arc<dyn Fn(&CStr) -> *const c_void + Send + Sync>;

fn resolve_gl(loader: &ProcAddressLoader, name: &str) -> *mut c_void {
    let Ok(name) = CString::new(name) else {
        return std::ptr::null_mut();
    };
    loader(&name) as *mut c_void
}

pub struct Player {
    render: RenderContext<'static>,
    mpv: Mpv,
    surface: Option<VideoSurface>,
}

impl Player {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, PlayerError> {
        let loader = cc
            .get_proc_address
            .clone()
            .ok_or(PlayerError::MissingGlowContext)?;

        // `vo=libmpv` is what stops mpv from opening a window, making the render
        // API its output
        let mpv = Mpv::with_initializer(|init| {
            init.set_property("vo", "libmpv")?;
            init.set_property("hwdec", "auto-safe")?;
            init.set_property("keep-open", "yes")?;
            init.set_property("idle", "yes")?;
            Ok(())
        })?;

        let render = mpv.create_render_context(vec![
            RenderParam::ApiType(RenderParamApiType::OpenGl),
            RenderParam::InitParams(OpenGLInitParams {
                get_proc_address: resolve_gl,
                ctx: loader,
            }),
        ])?;

        let render: RenderContext<'static> = unsafe { std::mem::transmute(render) };

        Ok(Self {
            render,
            mpv,
            surface: None,
        })
    }

    pub fn repaint_on_new_frames(&mut self, ctx: egui::Context) {
        self.render.set_update_callback(move || ctx.request_repaint());
    }

    pub fn ensure_surface(&mut self, frame: &mut eframe::Frame) -> Result<(), PlayerError> {
        if self.surface.is_some() {
            return Ok(());
        }
        let gl = frame.gl().ok_or(PlayerError::MissingGlowContext)?.clone();
        self.surface = Some(VideoSurface::new(&gl, frame)?);
        Ok(())
    }

    pub fn surface(&self) -> Option<&VideoSurface> {
        self.surface.as_ref()
    }

    pub fn render(&self) -> Result<(), PlayerError> {
        let Some(surface) = &self.surface else {
            return Ok(());
        };
        let [width, height] = surface.size();
        // `false` here, not `true`: mpv's flag compensates for OpenGL's
        // bottom-left origin when drawing to the screen. egui instead maps the
        // top edge of the image to the texture's first row, which for an FBO is
        // the bottom one. Flipping in both places shows the video upside down.
        self.render
            .render::<ProcAddressLoader>(surface.framebuffer_id(), width, height, false)?;
        Ok(())
    }

    pub fn load(&self, path: &Path) -> Result<(), PlayerError> {
        let path = path.to_string_lossy();
        self.mpv.command("loadfile", &[path.as_ref(), "replace"])?;
        Ok(())
    }

    pub fn play(&self) -> Result<(), PlayerError> {
        self.mpv.set_property("pause", false)?;
        Ok(())
    }

    pub fn pause(&self) -> Result<(), PlayerError> {
        self.mpv.set_property("pause", true)?;
        Ok(())
    }

    pub fn set_volume(&self, volume: f64) -> Result<(), PlayerError> {
        self.mpv.set_property("volume", volume)?;
        Ok(())
    }

    pub fn set_muted(&self, muted: bool) -> Result<(), PlayerError> {
        self.mpv.set_property("mute", muted)?;
        Ok(())
    }

    pub fn seek(&self, seconds: f64) -> Result<(), PlayerError> {
        self.mpv
            .command("seek", &[&format!("{seconds:.3}"), "absolute"])?;
        Ok(())
    }

    pub fn position(&self) -> f64 {
        self.mpv.get_property::<f64>("time-pos").unwrap_or(0.0)
    }

    pub fn duration(&self) -> f64 {
        self.mpv.get_property::<f64>("duration").unwrap_or(0.0)
    }

    pub fn is_paused(&self) -> bool {
        self.mpv.get_property::<bool>("pause").unwrap_or(true)
    }

    pub fn is_eof(&self) -> bool {
        self.mpv
            .get_property::<bool>("eof-reached")
            .unwrap_or(false)
    }
}
