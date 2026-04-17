// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use downlowd::Client;
use serde_json::Value;
use std::env;

fn get_link(show: &str) -> String {
    
}
#[tauri::command]
fn download(show: &str) -> String {
   
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![download])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
