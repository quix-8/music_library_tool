use clap::Parser;
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    path: PathBuf,
}

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

// Вызывать после завершения основного цикла скачивания
async fn trigger_jellyfin_scan(client: &Client) -> Result<(), reqwest::Error> {
    let jellyfin_url = "http://localhost:8096/Library/Refresh";
    let api_key = "3787ea91574e4082a237df7bd0732b84";

    let response = client
        .post(jellyfin_url)
        .header("X-Emby-Token", api_key)
        .send()
        .await?;

    if response.status().is_success() {
        println!("[+] Jellyfin начал сканирование библиотеки!");
    } else {
        eprintln!("[-] Ошибка сканирования Jellyfin: {}", response.status());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut data: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&args.path) {
        match entry {
            Ok(e) => {
                let path = e.into_path();
                if path.is_file() {
                    let is_audio = path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext_str| {
                            let lower = ext_str.to_lowercase();
                            // Здесь перечисляешь форматы, которые у тебя есть в медиатеке
                            matches!(lower.as_str(), "flac" | "mp3" | "m4a" | "ogg" | "wav")
                        })
                        .unwrap_or(false); // Если расширения нет (например файл "README"), вернет false

                    if is_audio {
                        data.push(path);
                    }
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
        let lrc_path = path.with_extension("lrc");

        if lrc_path.exists() {
            println!("Пропускаем {:?}, LRC уже существует", path);
            continue;
        }
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

            let (lyrics_to_save, is_synced) = if let Some(synced) = track_info.synced_lyrics {
                (Some(synced), true)
            } else if let Some(plain) = track_info.plain_lyrics {
                (Some(plain), false)
            } else {
                (None, false)
            };

            if let Some(text) = lyrics_to_save {
                let lrc_path = original_path.with_extension("lrc");

                match fs::write(&lrc_path, text).await {
                    Ok(_) => {
                        let tag = if is_synced { "[+ SYNC]" } else { "[+ PLAIN]" };
                        println!("{} Сохранен LRC для: {} - {}", tag, artist_name, track_name);
                    }
                    Err(e) => eprintln!("[-] Ошибка при записи {:?}: {}", lrc_path, e),
                }
            } else {
                println!("[!] Текста вообще нет: {} - {}", artist_name, track_name);
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
    trigger_jellyfin_scan(&client).await?;
    Ok(())
}
