{ pkgs ? import <nixpkgs> {} }:

let
  # Runtime + link libraries.
  #
  # Note what is NOT here anymore: WebKitGTK, GTK, GStreamer, libsoup, xdotool.
  # The preview no longer goes through a webview; libmpv decodes and we render
  # the frames ourselves through OpenGL.
  libraries = with pkgs; [
    # --- libmpv: the decoding backend (render API) ---
    mpv

    # --- OpenGL / EGL (eframe's glow backend + libmpv's render API) ---
    libGL
    libglvnd
    libgbm

    # --- Windowing for winit/glutin (X11 + Wayland in a single binary) ---
    wayland
    wayland-protocols
    libxkbcommon
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    xorg.libXext
    xorg.libXfixes
    libxcb

    # --- Text rendering for egui ---
    fontconfig
    freetype

    # --- Misc deps of the above ---
    dbus
    zlib
  ];

  # Directory holding libmpv.so, needed on LIBRARY_PATH so the linker can
  # resolve `-lmpv` (libmpv2-sys emits a bare `cargo:rustc-link-lib=mpv`).
  mpvLibPath = pkgs.lib.makeLibraryPath [ pkgs.mpv ];
in
pkgs.mkShell {
  # NOTE: no `cargo`/`rustc` here on purpose. nixpkgs' rustc (1.91) is older
  # than what eframe/egui 0.34 requires (>= 1.92), so we deliberately use the
  # toolchain already present on the system (rustup) while this shell provides
  # only the native libraries and their search paths.
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    # ffmpeg is still required: lossless export runs `ffmpeg -c copy`.
    ffmpeg
  ] ++ libraries;

  shellHook = ''
    # Linker search path: resolves `-lmpv`.
    export LIBRARY_PATH="${mpvLibPath}:$LIBRARY_PATH"

    # Runtime search path for the produced binary.
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH"

    # Let fontconfig find system fonts.
    export XDG_DATA_DIRS="${pkgs.fontconfig}/share:$XDG_DATA_DIRS"

    echo "Trimontina | egui + libmpv (OpenGL render API)"
    echo "   mpv   : $(command -v mpv || echo n/a)"
    echo "   ffmpeg: $(command -v ffmpeg || echo n/a)"
  '';
}
