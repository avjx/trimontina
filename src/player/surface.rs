use eframe::{
    egui,
    glow::{self, HasContext as _},
};

use super::error::PlayerError;

// Fixed: eframe cannot free a registered texture, so resizing it would leak.
const SURFACE_SIZE: [i32; 2] = [1920, 1080];

pub struct VideoSurface {
    framebuffer: glow::Framebuffer,
    texture_id: egui::TextureId,
    size: [i32; 2],
}

impl VideoSurface {
    pub fn new(gl: &glow::Context, frame: &mut eframe::Frame) -> Result<Self, PlayerError> {
        let size = SURFACE_SIZE;
        let texture = unsafe { create_texture(gl, size)? };
        let framebuffer = unsafe { create_framebuffer(gl, texture)? };
        let texture_id = frame.register_native_glow_texture(texture);

        Ok(Self {
            framebuffer,
            texture_id,
            size,
        })
    }

    pub fn texture_id(&self) -> egui::TextureId {
        self.texture_id
    }

    pub fn size(&self) -> [i32; 2] {
        self.size
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.size[0] as f32 / self.size[1] as f32
    }

    pub fn framebuffer_id(&self) -> i32 {
        self.framebuffer.0.get() as i32
    }
}

/// # Safety
/// Requires a current OpenGL context.
unsafe fn create_texture(
    gl: &glow::Context,
    size: [i32; 2],
) -> Result<glow::Texture, PlayerError> {
    unsafe {
        let texture = gl.create_texture().map_err(PlayerError::Gl)?;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            size[0],
            size[1],
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(None),
        );
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
        Ok(texture)
    }
}

/// # Safety
/// Requires a current OpenGL context.
unsafe fn create_framebuffer(
    gl: &glow::Context,
    texture: glow::Texture,
) -> Result<glow::Framebuffer, PlayerError> {
    unsafe {
        let framebuffer = gl.create_framebuffer().map_err(PlayerError::Gl)?;
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(texture),
            0,
        );

        let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);

        if status != glow::FRAMEBUFFER_COMPLETE {
            return Err(PlayerError::IncompleteFramebuffer { status });
        }

        Ok(framebuffer)
    }
}
