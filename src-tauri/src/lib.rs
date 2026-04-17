// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde_json::Value;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::format;
use stream_download::source::DecodeError;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tokio::io::AsyncWriteExt;

#[derive(Deserialize)]
struct Show {
    path: String,
    episode: u32,
    season: String,
    show: bool,
    url: String,
    #[serde(rename = "episode links")]  // CHANGED: handle the space in key
    episode_links: HashMap<String, HashMap<String, String>>,
}
fn get_json_data() -> String {
    return std::fs::read_to_string("shows.json").unwrap()
}

fn get_link(show: &str) -> String {
    let content = get_json_data();
    let shows: HashMap<String, Show> = serde_json::from_str(&content).unwrap();
    let show_info = &shows[show];
    let season = &show_info.season;
    let episode = &show_info.episode.to_string();
    let episode_link = &show_info.episode_links[season][episode];

    if episode_link.contains("https://") {
        episode_link.to_string()
    } else {
        format!("https://a.111477.xyz{}", episode_link)
    }
}
#[tauri::command]
#[tokio::main]
async fn download(show: &str) -> Result<Response, String>{
    let link = get_link(show);
    let json_data = get_json_data();
    let shows: HashMap<String, Show> = serde_json::from_str(&json_data).unwrap();
    let show = &shows[show];
    let file_name = format!("{}/{}{}{}", show.path, show.season, format!("{:0>2}", show.episode.to_string()), link.split(".").last().unwrap().to_string());
    let mut file = std::fs::File::create(file_name).map_err(|e| e.to_string())?;

    Ok(())

}

#[tauri::command]
fn get_options() -> Vec<String> {
    let content = get_json_data();
    let shows: HashMap<String, Show> = serde_json::from_str(&content).unwrap();
    let mut options: Vec<String> = Vec::new();
    for (show, _) in shows.iter() {
        options.push(show.to_string());
    }
    options
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![download, get_options])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
