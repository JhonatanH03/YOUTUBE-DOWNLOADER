use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use uuid::Uuid;

static DOWNLOADS: Lazy<Mutex<HashMap<String, Child>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[tauri::command]
fn download_video(app: tauri::AppHandle, url: String, quality: Option<String>) -> Result<String, String> {
    // Validate URL format
    if !url.contains("youtube.com") && !url.contains("youtu.be") {
        return Err("Invalid YouTube URL".to_string());
    }

    // Validate quality format
    let q = quality.unwrap_or_else(|| "best".to_string());
    if !matches!(q.as_str(), "best" | "bestvideo" | "bestaudio") {
        return Err(format!("Invalid quality: {}", q));
    }

    let id = Uuid::new_v4().to_string();

    // Check if yt-dlp is installed
    match Command::new("yt-dlp").arg("--version").output() {
        Ok(_) => {},
        Err(_) => return Err("yt-dlp not found. Please install it: pip install yt-dlp".to_string()),
    }

    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-f")
       .arg(&q)
       .arg("--newline")
       .arg("-o")
       .arg("%(title)s.%(ext)s")
       .arg(&url);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let child = cmd.spawn().map_err(|e| format!("Failed to spawn yt-dlp: {}", e))?;

    // Store child process
    {
        let mut map = DOWNLOADS.lock().unwrap();
        map.insert(id.clone(), child);
    }

    // Spawn worker thread to handle process
    let app_handle = app.clone();
    let id_clone = id.clone();
    
    std::thread::spawn(move || {
        let re = Regex::new(r"\[download\].*?(\d{1,3}\.\d)%").unwrap();

        let mut guard = DOWNLOADS.lock().unwrap();
        if let Some(mut running) = guard.remove(&id_clone) {
            if let Some(stdout) = running.stdout.take() {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        if let Some(cap) = re.captures(&l) {
                            if let Some(pct) = cap.get(1) {
                                let progress = pct.as_str();
                                let payload = serde_json::json!({"id": id_clone, "progress": progress});
                                let _ = app_handle.emit_all("download-progress", payload);
                            }
                        }
                    }
                }
            }

            // Wait for process to complete
            match running.wait() {
                Ok(status) => {
                    if status.success() {
                        let payload = serde_json::json!({"id": id_clone, "status": "finished"});
                        let _ = app_handle.emit_all("download-finished", payload);
                    } else {
                        let error_msg = format!("yt-dlp exited with code: {:?}", status.code());
                        let payload = serde_json::json!({"id": id_clone, "error": error_msg});
                        let _ = app_handle.emit_all("download-error", payload);
                    }
                }
                Err(e) => {
                    let payload = serde_json::json!({"id": id_clone, "error": format!("Error: {}", e)});
                    let _ = app_handle.emit_all("download-error", payload);
                }
            }
        }
    });

    Ok(id)
}

#[tauri::command]
fn cancel_download(id: String) -> Result<bool, String> {
    let mut map = DOWNLOADS.lock().unwrap();
    if let Some(mut child) = map.remove(&id) {
        match child.kill() {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("failed to kill process: {}", e)),
        }
    } else {
        Err("download id not found".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![download_video, cancel_download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
