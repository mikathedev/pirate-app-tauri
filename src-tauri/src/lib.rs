// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use downlowd::Client;
use serde_json::Value;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::env;


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

fn get_link(show: &str) -> String {
    let content = get_json_data();
    let shows: HashMap<String, Show> = serde_json::from_str(&content).unwrap();
    let show_info = &shows[show];
    let season = &show_info.season;
    let episode = &show_info.episode.to_string();
    let episode_link = &show_info.episode_links[season][episode];

    episode_link.to_string()
}
#[tauri::command]
fn download(show: &str) -> String {
   get_link(show)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![download])
        .invoke_handler(tauri::generate_handler![get_options])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
