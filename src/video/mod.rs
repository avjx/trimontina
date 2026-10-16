mod error;
mod ffmpeg;

pub use ffmpeg::{Cut, cut_lossless};

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum ExportEvent {
    Progress(f32),
    Completed(PathBuf),
    Failed(String),
}

pub fn cut_in_background(
    input: PathBuf,
    output: PathBuf,
    cut: Cut,
) -> std::sync::mpsc::Receiver<ExportEvent> {
    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let progress_sender = sender.clone();
        let result = cut_lossless(&input, &output, cut, move |fraction| {
            let _ = progress_sender.send(ExportEvent::Progress(fraction));
        });

        let event = match result {
            Ok(()) => ExportEvent::Completed(output),
            Err(err) => ExportEvent::Failed(err.to_string()),
        };
        let _ = sender.send(event);
    });

    receiver
}

#[cfg(test)]
mod tests {
    use super::{Cut, cut_lossless, error::ExportError};
    use std::path::{Path, PathBuf};

    fn sample_clip() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/sample.mp4")
    }

    #[test]
    fn a_range_without_duration_is_rejected_before_calling_ffmpeg() {
        let result = cut_lossless(
            Path::new("missing-input.mp4"),
            Path::new("output.mp4"),
            Cut {
                start: 5.0,
                end: 5.0,
            },
            |_| {},
        );

        assert!(
            matches!(result, Err(ExportError::InvalidRange { .. })),
            "expected InvalidRange, got {result:?}"
        );
    }

    #[test]
    fn cuts_a_clip_without_re_encoding() {
        let input = sample_clip();
        if !input.is_file() {
            eprintln!("testdata/sample.mp4 missing, skipping test");
            return;
        }

        let output = std::env::temp_dir().join("trimontina-test-cut.mp4");
        let _ = std::fs::remove_file(&output);

        let mut reported: Vec<f32> = Vec::new();
        let result = cut_lossless(
            &input,
            &output,
            Cut {
                start: 1.0,
                end: 3.0,
            },
            |progress| reported.push(progress),
        );

        assert!(result.is_ok(), "cut failed: {result:?}");
        assert!(output.is_file(), "the output file was not created");

        let size = std::fs::metadata(&output)
            .expect("output metadata")
            .len();
        assert!(size > 0, "the output file is empty");

        assert!(!reported.is_empty(), "no progress was reported");
        assert_eq!(
            reported.last().copied(),
            Some(1.0),
            "progress did not reach 100%"
        );
        assert!(
            reported.windows(2).all(|w| w[0] <= w[1]),
            "progress went backwards: {reported:?}"
        );

        let _ = std::fs::remove_file(&output);
    }
}
