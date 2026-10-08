use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("ffmpeg not found. Install it or point FFMPEG_PATH at it. ({0})")]
    NotFound(#[source] std::io::Error),

    #[error("ffmpeg exited with code {code}")]
    Failed { code: i32 },

    #[error("invalid range: start {start:.3}s, end {end:.3}s")]
    InvalidRange { start: f64, end: f64 },
}
