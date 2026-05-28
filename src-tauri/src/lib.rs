// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

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
    let q = quality.unwrap_or_else(|| "best".to_string());
    let id = Uuid::new_v4().to_string();

    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-f").arg(q).arg("--newline").arg(&url);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("failed to spawn yt-dlp: {}", e))?;

    // store child so it can be cancelled and later removed by the worker thread
    {
        let mut map = DOWNLOADS.lock().unwrap();
        map.insert(id.clone(), child);
    }

    // Move a clone of app handle into thread
    let app_handle = app.clone();
    let id_clone = id.clone();
    std::thread::spawn(move || {
        // read stdout for progress lines
        let re = Regex::new(r"\[download\].*?(\d{1,3}\.\d)%").unwrap();
        if let Ok(mut child_proc) = Command::new("yt-dlp").arg("--version").output() {
            // noop just to satisfy borrow checker in some environments
        }

        // Re-open the process from the map
        let mut guard = DOWNLOADS.lock().unwrap();
        if let Some(mut running) = guard.remove(&id_clone) {
            let stdout = running.stdout.take();
            if let Some(out) = stdout {
                let reader = BufReader::new(out);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        if let Some(cap) = re.captures(&l) {
                            if let Some(pct) = cap.get(1) {
                                let progress = pct.as_str().to_string();
                                let payload = serde_json::json!({"id": id_clone, "progress": progress, "line": l});
                                let _ = app_handle.emit_all("download-progress", payload);
                            }
                        } else {
                            let payload = serde_json::json!({"id": id_clone, "line": l});
                            let _ = app_handle.emit_all("download-log", payload);
                        }
                    }
                }
            }

            // wait for exit
            let _ = running.wait();
            let payload = serde_json::json!({"id": id_clone, "status": "finished"});
            let _ = app_handle.emit_all("download-finished", payload);
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
        .invoke_handler(tauri::generate_handler![greet, download_video, cancel_download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
