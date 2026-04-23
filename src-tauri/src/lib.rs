use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Write;
use futures_util::StreamExt;
use scraper::{Html, Selector};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use regex::Regex;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();


#[derive(Deserialize, Serialize)]
struct Show {
    path: String,
    episode: u32,
    season: String,
    url: String,
    #[serde(rename = "episode links")]
    episode_links: HashMap<String, HashMap<String, String>>,
}

fn get_json_data() -> String {
    let path = std::env::current_exe()
        .expect("cant get exe path")
        .parent()
        .expect("cant get exe dir")
        .join("shows.json");

    println!("{:?}", path);
        std::fs::read_to_string(path).unwrap()
}
fn emit(data: String) {
    if let Some(app_handle) = APP_HANDLE.get() {
        app_handle.emit("BE", data).unwrap();
    }
}

fn get_link(show: &str) -> String {
    let content = get_json_data();
    let shows: HashMap<String, Show> = serde_json::from_str(&content).unwrap();
    let show_info = &shows[show];
    let season = &show_info.season;
    let episode = &show_info.episode.to_string();
    println!("{}", season);
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
    println!("{:?}", client);
    println!("{:?}", link);
    let response = client.get(&link).send().await.map_err(|e| e.to_string())?;
    println!("{:?}", response);
    if !response.status().is_success() {
        return Err(format!("Request failed: {}", response.status()));
    } else {
        println!("Sucsess: {:?}", response);
    }
    let size = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    println!("Downloading {}...", file_name);

    if let Some(parent) = std::path::Path::new(&file_name).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut file = std::fs::File::create(&file_name).unwrap();
    let mut last_emit = std::time::Instant::now();
    let mut downloaded = 0;
    while let Some(chunk) = &stream.next().await {
        let chunk_error_handler = chunk.as_ref().map_err(|e| e.to_string())?;
        file.write_all(&chunk_error_handler).map_err(|e| format!("Write failed: {}", e)).expect("error while writing");

        downloaded += chunk_error_handler.len();
        if last_emit.elapsed().as_millis() > 5000 {
            let progress = downloaded as f64 / size as f64;
            emit(format!("{:.2}", progress * 100.0));
            println!("{:.2}%", progress * 100.0);
            last_emit = std::time::Instant::now();
        }
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

#[derive(Default)]
#[derive(Debug)]
struct Item {
    url: String,
    size: u64,
}

fn get_lowests(total: u32, items: Vec<Item>) -> HashMap<String, String> {
    let mut lowests: HashMap<String, String> = HashMap::new();
    for episode in 1..=total {
        let epi = format!("E{:0>2}", episode);
        lowests.insert(format!("{}", episode), items.iter().filter(|item| Regex::new(&epi).unwrap().find(&*item.url).is_some())
            .min_by_key(|p| p.size).unwrap().url.clone()
        );
    }
    lowests
}


#[tauri::command]
async fn scrape(show: String) {
    let json = get_json_data();
    let mut content: HashMap<String, Show> = serde_json::from_str(&json).unwrap();
    let show_info = &content[&show];
    let season = format!("{:0>2}", (&show_info.season.parse::<u32>().unwrap() + 1).to_string());
    let url = &show_info.url;
    let response = reqwest::get(url).await.unwrap();
    let html = response.text().await.unwrap();
    if html.contains(&season.parse::<u32>().unwrap().to_string()) {
        println!("found season");
        let episodes = reqwest::get(format!("{}Season%20{}", url, &season.parse::<u32>().unwrap().to_string())).await.unwrap().text().await.unwrap();
        let doc = Html::parse_document(&episodes.as_str());
        let url_selector = Selector::parse("tr td a").unwrap();
        let size_selector = Selector::parse(".size").unwrap();
        let target_url = url.to_string().replace("https://a.111477.xyz", "");
        let mut seen: HashSet<String> = HashSet::new();
        let re = Regex::new(r"S\d+E\d+").unwrap();
        let total_episodes: u32 = doc.select(&url_selector).filter(|x| {
            if let Some(mat) = re.find(&x.inner_html()) {
                seen.insert(mat.as_str().to_string())
            } else { false }
        }).count() as u32;

        let mut links: Vec<String> = doc.select(&url_selector)
            .filter_map(|x| x.value().attr("href"))
            .filter(|href| href.contains(&target_url))
            .map(|href| href.to_string())
            .collect();
        let sizes: Vec<u64> = doc.select(&size_selector)
            .filter_map(|x| {
                let html = x.inner_html();

                if html.contains("GB") {
                  Some(html.replace(" GB", "").parse::<f32>().map(|e| (e * 1024f32) as u64 ).unwrap())

                } else if html.contains("MB") {
                    Some(html.parse::<f32>().map(|e| e as u64).unwrap())
                } else {
                    None
                }
            })
            .collect();

        let mut items: Vec<Item> = Vec::new();

        println!("{:?} {}", links.len(), sizes.len());
        if links.len() != sizes.len() {
            println!("not the same removing first item");
            links.remove(0);
            items = links.into_iter().zip(sizes.into_iter()).map(|(url, size)| Item { url, size }).collect();
        } else if links.len() == sizes.len() {
            println!("{} {}", links.len(), sizes.len());
            items = links.into_iter().zip(sizes.into_iter()).map(|(url, size)| Item { url, size }).collect();
        }
        let lowest = get_lowests(total_episodes, items);

        let mut new = content[&show].episode_links.clone();
        new.insert(format!("{:0>2}", season.to_string()), lowest);
        println!("{:#?}\n\n\n\n", new);
        let Some(show_data) = content.get_mut(&show) else { println!("show not found"); return };
        show_data.episode_links = new;
        let new_json = serde_json::to_string_pretty(&content).unwrap();
        print!("{:?}", new_json);
        let path = std::env::current_exe()
            .expect("cant get exe path")
            .parent()
            .expect("cant get exe dir")
            .join("shows.json");
        std::fs::write(&path, new_json).expect("Writing Failed");
    } else { println!("season not found"); }
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{}", info);
        std::fs::write("crash.log", &msg).ok();
        eprintln!("{}", msg);
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![download, get_options, get_video_path, scrape])
        .setup(|app| {
            APP_HANDLE.set(app.handle().clone()).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}