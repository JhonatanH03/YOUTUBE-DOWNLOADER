// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn download_video(url: &str, quality: Option<String>) -> Result<String, String> {
    // Stub implementation: in future this will invoke yt-dlp or similar.
    println!("download_video requested: url={} quality={:?}", url, quality);
    Ok(format!("Queued download for {} (quality: {:?})", url, quality))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, download_video])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
