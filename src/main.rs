use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::Accessor;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;
use tokio::fs;
use walkdir::WalkDir;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Track {
    id: i32,
    track_name: String,
    artist_name: String,
    album_name: String,
    duration: f64,
    instrumental: bool,
    plain_lyrics: Option<String>,
    synced_lyrics: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut data: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(".") {
        match entry {
            Ok(e) => {
                let path = e.into_path();
                if path.is_file() {
                    data.push(path);
                }
            }
            Err(err) => eprintln!("Пропущено из-за ошибки: {}", err),
        }
    }

    let mut apis: Vec<(PathBuf, String, String, String, u64)> = Vec::new();

    for path in data {
        let file = match read_from_path(&path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let duration = file.properties().duration();
        let duration_secs = duration.as_secs();

        let mut tl = String::new();
        let mut art = String::new();
        let mut alb = String::new();

        if let Some(tag) = file.primary_tag() {
            if let Some(title) = tag.title() {
                tl = title.into_owned();
            }
            if let Some(artist) = tag.artist() {
                art = artist.into_owned();
            }
            if let Some(album) = tag.album() {
                alb = album.into_owned();
            }
        }

        if tl.is_empty() || art.is_empty() {
            continue;
        }

        apis.push((path, tl, art, alb, duration_secs));
    }

    let client = Client::new();
    let url = "https://lrclib.net/api/get";

    for tag in apis {
        let original_path = tag.0;
        let track_name = tag.1;
        let artist_name = tag.2;
        let album_name = tag.3;
        let duration_str = tag.4.to_string();

        let params = [
            ("track_name", &track_name),
            ("artist_name", &artist_name),
            ("album_name", &album_name),
            ("duration", &duration_str),
        ];

        let response = client.get(url).query(&params).send().await?;

        if response.status().is_success() {
            let track_info = response.json::<Track>().await?;

            if let Some(lyrics) = track_info.synced_lyrics {
                let lrc_path = original_path.with_extension("lrc");

                match fs::write(&lrc_path, lyrics).await {
                    Ok(_) => println!("[+] Сохранен LRC для: {} - {}", artist_name, track_name),
                    Err(e) => eprintln!("[-] Ошибка при записи {:?}: {}", lrc_path, e),
                }
            } else {
                println!("[!] Текста нет: {} - {}", artist_name, track_name);
            }
        } else {
            println!(
                "[-] Ошибка {} для: {} - {}",
                response.status(),
                artist_name,
                track_name
            );
        }
    }

    Ok(())
}
