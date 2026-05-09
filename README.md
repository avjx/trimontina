# Trimontina

Trimontina is a lightweight video trimmer built with Rust. It provides a modern, immersive interface for selecting and exporting specific segments of video files without quality loss.

## Tech Stack

*   **Dioxus**: Used for the frontend application and desktop window management.
*   **MPV**: Integrated via IPC for real-time video and audio preview.
*   **FFmpeg**: Powers the backend video processing for fast, lossless cutting using stream copy.

## Key Features

*   **Interactive Timeline**: Precise frame seeking and range selection using draggable handles.
*   **Lossless Export**: Instant video trimming without re-encoding, preserving original quality.

## Requirements

To run this project, you must have the following tools installed and available in your system's PATH:

1.  **mpv**: For video playback.
2.  **ffmpeg**: For video exporting.
