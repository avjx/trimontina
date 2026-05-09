use std::path::PathBuf;
use std::process::Command;

pub fn cut_video(input: PathBuf, output: PathBuf, start: f64, end: f64) -> Result<(), String> {
    //optimized command:
    //  -ss before -i uses input seeking (much faster)
    // -to defines the stopping point
    //-c copy avoids re-encoding the video (preserves original quality and is nearly instantaneous

    let status = Command::new("ffmpeg")
        .arg("-ss")
        .arg(format!("{:.3}", start))
        .arg("-i")
        .arg(input)
        .arg("-to")
        .arg(format!("{:.3}", end - start))
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg(&output)
        .status()
        .map_err(|e| {
            format!(
                "error trying to execute ffmpeg: {}. check if it is in PATH.",
                e
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg error.".to_string())
    }
}
