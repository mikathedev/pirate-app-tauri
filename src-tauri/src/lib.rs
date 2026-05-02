use futures_util::{StreamExt, TryFutureExt};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::io::Write;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use tokio::runtime::Runtime;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[derive(Deserialize, Serialize)]
struct Show {
    path: String,
    episode: u32,
    downloaded: u32,
    season: String,
    url: String,
    #[serde(rename = "episode links")]
    episode_links: HashMap<String, HashMap<String, String>>,
}

fn get_json_data() -> HashMap<String, Show> {
    let path = env::current_exe()
        .expect("cant get exe path")
        .parent()
        .expect("cant get exe dir")
        .join("shows.json");
    println!("{:?}", path);
    let shows: HashMap<String, Show> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).expect("cant read json");
    shows
}
fn emit(data: String, event_type: &str) {
    if let Some(app_handle) = APP_HANDLE.get() {
        app_handle.emit(event_type, data).unwrap();
    }
}
async fn resolve_season_episode(
    show: &Show,
    offset: u32
) -> (String, u32, u32) {
    let season = format!("{:0>2}", show.season);
    let episode = show.episode;

    if episode + offset > show.episode_links[&show.season].len() as u32 {
        let next_season = format!("{:0>2}", season.parse::<u32>().unwrap() + 1);
        let nepi = episode + offset - show.episode_links[&show.season].len() as u32;
        (next_season, 1, nepi)
    } else {
        (season, episode, offset)
    }
}

fn resolve_link(show: &Show, season: &str, episode: u32) -> String {
    let raw = &show.episode_links[season][&episode.to_string()];
    if raw.contains("https://") {
        raw.clone()
    } else {
        format!("https://a.111477.xyz{}", raw)
    }
}

fn build_file_name(show: &Show, season: &str, episode: u32, link: &str) -> String {
    let ext = link.split('.').last().unwrap_or("mp4");
    format!(
        "{}/{}{}.{}.tmp",
        show.path,
        season,
        format!("{:0>2}", episode),
        ext
    )
}

async fn stream_to_file(response: reqwest::Response, file_name: &str) -> Result<(), String> {
    let size = response.content_length().unwrap_or(0);

    // ensure parent dirs exist
    if let Some(parent) = std::path::Path::new(file_name).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut file = std::fs::File::create(file_name).map_err(|e| e.to_string())?;
    let mut stream = response.bytes_stream();
    let mut downloaded = 0usize;
    let mut last_download = 0usize;
    let mut last_emit = std::time::Instant::now();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| format!("Write failed: {}", e))?;
        downloaded += chunk.len();

        // throttle progress emit to every 5s
        if last_emit.elapsed().as_millis() > 5000 {
            let progress = downloaded as f64 / size as f64 * 100.0;
            emit(format!("{:.2}", progress), "download");
            println!("{:.2}%   {} MB/s", progress, ((downloaded - last_download)/5)/1024^2);
            last_download = downloaded;
            last_emit = std::time::Instant::now();
        }
    }
    Ok(())
}

fn convert_to_mp4(file_name: &str) -> String {
    let new_name = file_name.split_once('.').unwrap().0.to_string() + ".mp4";
    let output = std::process::Command::new("ffmpeg")
        .args(["-i", file_name, "-c:v", "libx264", "-preset", "slow", "-crf", "22", "-c:a", "aac", "-af", "volume=1.5", &new_name])
        .output()
        .expect("failed to execute ffmpeg");
    println!("{:?}", output);
    new_name
}

async fn get_res(link: &str) -> Result<reqwest::Response, String> {
    let response = reqwest::Client::new()
        .get(&*link)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    println!("Status: {:?}", response.status());
    if response.status() == 429 {
        emit("429".to_string(), "Err");
        Err("429".to_string())
    } else if response.status() == 200 {
        Ok(response)
    } else {
        Err(response.status().to_string())
    }
}


#[tauri::command]
async fn download(showstr: &str, offset: u32,) -> Result<String, String> {
    let mut shows: HashMap<String, Show> = get_json_data();
    let show = &shows[showstr];
    println!("Started downloading: {}", showstr);

    let (season, episode, offset) = resolve_season_episode(show, offset).await;

    // scrape next season data if we rolled over
    if offset == 0 && episode == 1 {
        scrape(showstr.to_string(), false).await;
    }

    let link = resolve_link(&shows[showstr], &season, episode);
    let file_name = build_file_name(&shows[showstr], &season, episode, &link);

    if !std::path::Path::new(&file_name).exists() {
        println!("Starting: {} {}", file_name, link);
        let response = get_res(&link).map_err(|e| emit(e.to_string(), "Err")).await.unwrap();

        stream_to_file(response, &file_name).await?;
        emit(format!("{:.2}", 100.0), "download");
        emit("Download Complete".to_string(), "downloadFinnished");
        convert_to_mp4(&file_name);
        let path = env::current_exe().unwrap().parent().unwrap().join("shows.json");
        // persist incremented download count
        shows.get_mut(showstr).unwrap().downloaded += 1;
        std::fs::write(&path, serde_json::to_string_pretty(&shows).unwrap())
            .expect("Writing Failed");
    } else {
        println!("File already exists");
    }


    Ok(file_name)
}


#[tauri::command]
async fn do_i_download(show: &str) -> Result<(), String> {
    let shows: HashMap<String, Show> = get_json_data();
    let show_info = &shows[show];
    let episode = show_info.episode as i32;
    let downloaded = show_info.downloaded as i32;
    println!("{}: {}/{}", show, downloaded, episode);
    if downloaded - episode < 2_i32 {
        println!("downloading");
        let offset = downloaded - episode;
        let _ = download(show, if offset < 0 { (offset*-1) as u32 } else { offset as u32 }).await;
    } else {
        let offset = downloaded - episode;
        println!("not downloading {} - {} = {}", downloaded, episode, offset);
    }
    Ok(())
}

#[tauri::command]
fn get_options() -> Vec<String> {
    let shows: HashMap<String, Show> = get_json_data();
    let mut options: Vec<String> = Vec::new();
    for (show, _) in shows.iter() {
        options.push(show.to_string());
    }
    options
}

#[tauri::command]
fn get_video_path(show: &str) -> String {
    let shows: HashMap<String, Show> = get_json_data();
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
    let rt = Runtime::new().unwrap();
    rt.block_on(do_i_download(show)).expect("panic on get video path");
    "".to_string()
}

#[derive(Default, Debug)]
struct Item {
    url: String,
    size: u64,
}

fn get_lowests(total: u32, items: Vec<Item>) -> HashMap<String, String> {
    let mut lowests: HashMap<String, String> = HashMap::new();
    for episode in 1..=total {
        let epi = format!("E{:0>2}", episode);
        lowests.insert(
            format!("{}", episode),
            items
                .iter()
                .filter(|item| Regex::new(&epi).unwrap().find(&*item.url).is_some())
                .min_by_key(|p| p.size)
                .unwrap()
                .url
                .clone(),
        );
    }
    lowests
}

fn get_season(show_info: &Show, first: bool) -> String {
    if first {
        format!("{:0>2}", &show_info.season)
    } else {
        format!("{:0>2}", show_info.season.parse::<u32>().unwrap() + 1)
    }
}

async fn fetch_season_html(url: &str, season: &str) -> String {
    reqwest::get(format!(
        "{}Season%20{}",
        url,
        season.parse::<u32>().unwrap()
    ))
    .await
    .unwrap()
    .text()
    .await
    .unwrap()
}

fn parse_items(doc: &Html, target_url: &str) -> (Vec<String>, Vec<u64>) {
    let url_selector = Selector::parse("tr td a").unwrap();
    let size_selector = Selector::parse(".size").unwrap();

    let links: Vec<String> = doc
        .select(&url_selector)
        .filter_map(|x| x.value().attr("href"))
        .filter(|href| href.contains(target_url))
        .map(|href| href.to_string())
        .collect();

    let sizes: Vec<u64> = doc
        .select(&size_selector)
        .filter_map(|x| {
            let html = x.inner_html();
            if html.contains("GB") {
                Some(
                    html.replace(" GB", "")
                        .parse::<f32>()
                        .map(|e| (e * 1024f32) as u64)
                        .unwrap(),
                )
            } else if html.contains("MB") {
                Some(
                    html.replace(" MB", "")
                        .parse::<f32>()
                        .map(|e| e as u64)
                        .unwrap(),
                )
            } else {
                None
            }
        })
        .collect();

    (links, sizes)
}

fn count_episodes(doc: &Html) -> u32 {
    let url_selector = Selector::parse("tr td a").unwrap();
    let re = Regex::new(r"S\d+E\d+").unwrap();
    let mut seen: HashSet<String> = HashSet::new();
    doc.select(&url_selector)
        .filter(|x| {
            if let Some(mat) = re.find(&x.inner_html()) {
                seen.insert(mat.as_str().to_string())
            } else {
                false
            }
        })
        .count() as u32
}

fn build_items(mut links: Vec<String>, sizes: Vec<u64>) -> Vec<Item> {
    if links.len() != sizes.len() {
        println!("not the same, removing first item");
        links.remove(0);
    }
    links
        .into_iter()
        .zip(sizes)
        .map(|(url, size)| Item { url, size })
        .collect()
}

#[tauri::command]
async fn scrape(show: String, first: bool) {
    let mut content: HashMap<String, Show> = get_json_data();
    let season = get_season(&content[&show], first);
    let url = content[&show].url.clone();

    let index_html = reqwest::get(&url).await.unwrap().text().await.unwrap();
    if !index_html.contains(&season.parse::<u32>().unwrap().to_string()) {
        println!("season not found");
        return;
    }

    println!("found season");
    let episodes_html = fetch_season_html(&url, &season).await;
    let doc = Html::parse_document(&episodes_html);
    let target_url = url.replace("https://a.111477.xyz", "");

    let total_episodes = count_episodes(&doc);
    let (links, sizes) = parse_items(&doc, &target_url);
    let items = build_items(links, sizes);
    let lowest = get_lowests(total_episodes, items);

    let mut new = content[&show].episode_links.clone();
    new.insert(format!("{:0>2}", season), lowest);

    let Some(show_data) = content.get_mut(&show) else {
        println!("show not found");
        return;
    };
    show_data.episode_links = new;

    let new_json = serde_json::to_string_pretty(&content).unwrap();
    let path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("shows.json");
    std::fs::write(&path, new_json).expect("Writing Failed");
}

#[tauri::command]
async fn add_show(name: String, url: String, path: String) -> bool {
    let mut content: HashMap<String, Show> = get_json_data();
    let new = Show {
        path,
        episode: 1,
        downloaded: 0,
        season: "01".to_string(),
        url,
        episode_links: HashMap::new(),
    };
    content.insert(name.clone(), new);
    let new_json = serde_json::to_string_pretty(&content).unwrap();
    let path = env::current_exe()
        .expect("cant get exe path")
        .parent()
        .expect("cant get exe dir")
        .join("shows.json");
    std::fs::write(&path, new_json).expect("Writing Failed");
    scrape(name, true).await;
    true
}

#[tauri::command]
async fn ended(show: &str) -> Result<(), String> {
    let mut shows: HashMap<String, Show> = get_json_data();
    let mut shows2: HashMap<String, Show> = get_json_data();
    let show_info = &shows[show];

    let path = env::current_exe().unwrap().parent().unwrap().join("shows.json");
    let season_len = &show_info.episode_links[&show_info.season].len();
    if &show_info.episode.clone() + 1 > *season_len as u32 {
        scrape(show.to_string(), false).await;
        let new_season = format!("{:0>2}", show_info.season.parse::<u32>().unwrap() + 1);
        shows2.get_mut(show).unwrap().episode = 1;
        shows2.get_mut(show).unwrap().season = new_season;
    } else {
        shows2.get_mut(show).unwrap().episode += 1;
    }
    let file_name = format!("{}{:0>2}", shows2[show].season, shows2[show].episode);
    let re = Regex::new(&file_name).unwrap();
    let episodes = std::fs::read_dir(&show_info.path).expect("cant read dir")
        .map(|x| x.unwrap().path().display().to_string())
        .collect::<Vec<_>>();
    println!("looking for {} \n {:#?}", file_name, episodes);

    for ep in episodes {
        if re.is_match(&*ep) {
            emit(ep.clone(), "NextEpisode");
            println!("found {}", &*ep);
            shows.get_mut(show).unwrap().episode += 1;
            std::fs::write(&path,
                           serde_json::to_string_pretty(&shows).unwrap()).expect("FAILED WRITING AT ENDED");
            println!("wrote to file");
            do_i_download(show).await.expect("panic");
        }
    }

    do_i_download(show).await.expect("panic");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{}", info);
        emit(msg.clone(), "ERROR");
        std::fs::write("crash.log", &msg).ok();
        eprintln!("{}", msg);
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            download,
            get_options,
            get_video_path,
            scrape,
            add_show,
            ended,
            do_i_download
        ])
        .setup(|app| {
            APP_HANDLE.set(app.handle().clone()).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


