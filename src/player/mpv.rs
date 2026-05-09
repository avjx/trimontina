use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

#[cfg(windows)]
use interprocess::local_socket::LocalSocketStream;
#[cfg(unix)]
use interprocess::local_socket::LocalSocketStream;

pub struct MpvController {
    process: Option<Child>,
    ipc_name: String,
}

impl Drop for MpvController {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
        }
    }
}

impl MpvController {
    pub fn new() -> Self {
        let pid = std::process::id();
        Self {
            process: None,
            ipc_name: format!("trimontina-pipe-{}", pid),
        }
    }

    fn get_full_ipc_path(&self) -> String {
        #[cfg(windows)]
        return format!(r"\\.\pipe\{}", self.ipc_name);
        #[cfg(unix)]
        return format!("/tmp/{}.sock", self.ipc_name);
    }

    pub fn start(&mut self, wid: String) {
        let current_dir = std::env::current_dir().unwrap_or_default();
        let local_mpv = current_dir.join("mpv.exe");

        let exe_path = if local_mpv.exists() {
            local_mpv.to_string_lossy().to_string()
        } else {
            "mpv".to_string()
        };

        let mut cmd = Command::new(&exe_path);

        cmd.arg("--idle=yes")
            .arg(format!("--input-ipc-server={}", self.get_full_ipc_path()))
            .arg(format!("--wid={}", wid))
            .arg("--no-osc")
            .arg("--no-osd-bar")
            .arg("--keep-open=yes")
            .arg("--force-window=yes")
            .arg("--hwdec=auto")
            .arg("--pause=yes")
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        self.process = cmd.spawn().ok();
    }

    fn send_command(&self, command: Value) -> Option<Value> {
        let req = json!({ "command": command }).to_string() + "\n";

        let socket_to_connect = if cfg!(windows) {
            self.ipc_name.clone()
        } else {
            #[cfg(unix)]
            {
                format!("/tmp/{}.sock", self.ipc_name)
            }
            #[cfg(windows)]
            {
                "".to_string()
            }
        };

        if let Ok(mut stream) = LocalSocketStream::connect(socket_to_connect.as_str()) {
            let _ = stream.write_all(req.as_bytes());

            if command[0] == "get_property" {
                let mut reader = BufReader::new(stream);
                let mut response = String::new();
                if let Ok(_) = reader.read_line(&mut response) {
                    if let Ok(val) = serde_json::from_str::<Value>(&response) {
                        return Some(val["data"].clone());
                    }
                }
            }
        }
        None
    }

    pub fn load_file(&self, path: PathBuf) {
        let mut path_str = path.to_string_lossy().to_string();
        if path_str.starts_with(r"\\?\") {
            path_str = path_str.replace(r"\\?\", "");
        }
        self.send_command(json!(["loadfile", path_str]));
    }

    pub fn get_duration(&self) -> f64 {
        self.send_command(json!(["get_property", "duration"]))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    }

    pub fn get_current_time(&self) -> f64 {
        self.send_command(json!(["get_property", "time-pos"]))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
    }

    pub fn seek(&self, seconds: f64) {
        self.send_command(json!(["seek", seconds, "absolute"]));
    }

    pub fn set_geometry(&self, x: i32, y: i32, w: i32, h: i32) {
        let geo = format!("{}x{}+{}+{}", w, h, x, y);
        self.send_command(json!(["set_property", "geometry", geo]));
    }

    pub fn set_pause(&self, pause: bool) {
        self.send_command(json!(["set_property", "pause", pause]));
    }
}
