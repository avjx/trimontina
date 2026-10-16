use std::{
    ffi::OsString,
    io::{BufRead as _, BufReader},
    path::Path,
    process::{Command, Stdio},
};

use super::error::ExportError;

#[cfg(windows)]
const FFMPEG_BIN: &str = "ffmpeg.exe";
#[cfg(not(windows))]
const FFMPEG_BIN: &str = "ffmpeg";

fn resolve_binary() -> OsString {
    if let Some(path) = std::env::var_os("FFMPEG_PATH") {
        return path;
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let local = dir.join(FFMPEG_BIN);
        if local.is_file() {
            return local.into_os_string();
        }
    }

    OsString::from(FFMPEG_BIN)
}

#[derive(Clone, Copy, Debug)]
pub struct Cut {
    pub start: f64,
    pub end: f64,
}

impl Cut {
    fn length(&self) -> f64 {
        self.end - self.start
    }
}

pub type Progress = f32;

pub fn cut_lossless(
    input: &Path,
    output: &Path,
    cut: Cut,
    mut on_progress: impl FnMut(Progress),
) -> Result<(), ExportError> {
    let length = cut.length();
    if length <= 0.0 {
        return Err(ExportError::InvalidRange {
            start: cut.start,
            end: cut.end,
        });
    }

    let mut child = Command::new(resolve_binary())
        .arg("-hide_banner")
        .arg("-ss")
        .arg(format!("{:.3}", cut.start))
        .arg("-i")
        .arg(input)
        .arg("-t")
        .arg(format!("{length:.3}"))
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg("-progress")
        .arg("pipe:1")
        .arg("-nostats")
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(ExportError::NotFound)?;

    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            // Despite the name, `out_time_ms` is in microseconds.
            if let Some(raw) = line.strip_prefix("out_time_ms=")
                && let Ok(micros) = raw.trim().parse::<f64>()
            {
                let done = (micros / 1_000_000.0) / length;
                on_progress(done.clamp(0.0, 1.0) as Progress);
            }
        }
    }

    let status = child.wait().map_err(ExportError::NotFound)?;
    if !status.success() {
        return Err(ExportError::Failed {
            code: status.code().unwrap_or(-1),
        });
    }

    on_progress(1.0);
    Ok(())
}
