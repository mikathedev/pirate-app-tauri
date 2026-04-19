// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{Read, Write};
use futures_util::StreamExt;
use scraper::{Html, Selector};


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
    std::fs::read_to_string("shows.json").unwrap()
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
async fn download(show: &str) -> Result<String, String>{
    let link = get_link(show);
    let json_data = get_json_data();
    let shows: HashMap<String, Show> = serde_json::from_str(&json_data).unwrap();
    let show = &shows[show];
    let file_name = format!("{}/{}{}.{}", show.path, show.season, format!("{:0>2}", show.episode.to_string()), link.split(".").last().unwrap().to_string());
    //client logic
    println!("starting {:?}", file_name);
    let client = reqwest::Client::new();
    let mut response = client.get(&link).send().await.map_err(|e| e.to_string())?;
    let size = response.content_length().unwrap_or(0);
    let mut file = std::fs::File::create(&file_name).unwrap();
    let mut stream = response.bytes_stream();
    println!("Downloading {}...", file_name);

    if let Some(parent) = std::path::Path::new(&file_name).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut downloaded = 0;
    while let Some(chunk) = &stream.next().await {
        let chunk_error_handler = chunk.as_ref().map_err(|e| e.to_string())?;
        file.write_all(&chunk_error_handler).map_err(|e| format!("Write failed: {}", e)).expect("error while writing");

        downloaded += chunk_error_handler.len();
        let progress = downloaded as f64 / size as f64;
        println!("{:.2}%", progress * 100.0);
    }

    println!(
        "Downloaded {}",
        file_name
    );

    Ok(file_name)

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

#[tauri::command]
fn get_video_path(show: &str) -> String {
    let content = get_json_data();
    let shows: HashMap<String, Show> = serde_json::from_str(&content).unwrap();
    let show_info = &shows[show];
    let season = &show_info.season;
    let episode = &show_info.episode.to_string();
    let episodes = std::fs::read_dir(&show_info.path).unwrap();

    println!("season: {}, episode: {}", season, episode);
    for x in episodes {
        let path_str = x.unwrap().path().display().to_string();
        if path_str.contains(&format!("{}{:0>2}", season, episode)) {
            println!("found {:?}", path_str);
            return path_str;
        }
    }
    "".to_string()
}

#[tauri::command]
async fn scrape(show: String) {
    let json = get_json_data();
    let content: HashMap<String, Show> = serde_json::from_str(&json).unwrap();
    let show_info = &content[&show];
    let season = &show_info.season;
    let url = &show_info.url;
    let response = reqwest::get(url).await.unwrap();
    let html = response.text().await.unwrap();
    println!("{}", html);
    if html.contains(&season.parse::<u32>().unwrap().to_string()) {
        println!("found season");
        let episodes = reqwest::get(format!("{}/Season&20{}", url, &season.parse::<u32>().unwrap().to_string())).await.unwrap().text().await.unwrap();
        let doc = Html::parse_document(&episodes.as_str());

        println!("{}", episodes);


    } else {
        println!("season not found");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![download, get_options, get_video_path, scrape])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
