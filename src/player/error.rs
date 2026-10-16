use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("eframe must run with the glow (OpenGL) renderer")]
    MissingGlowContext,

    #[error("libmpv: {0}")]
    Mpv(#[from] libmpv2::Error),

    #[error("OpenGL: {0}")]
    Gl(String),

    #[error("incomplete framebuffer (status 0x{status:X})")]
    IncompleteFramebuffer { status: u32 },
}
