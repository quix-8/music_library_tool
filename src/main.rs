use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::Accessor;
use reqwest::Client;
use std::error::Error;
use std::path::PathBuf;
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(".") {
        match entry {
            Ok(e) => data.push(e.into_path()),
            Err(err) => eprintln!("Пропущено из-за ошибки: {}", err), // Логируем ошибку, но программа продолжает работать
        }
    }
    let mut apis: Vec<(String, String, String, u64)> = Vec::new();
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
            println!("Файл: {:?}", path);

            if let Some(title) = tag.title() {
                tl = title.into_owned();
            }
            if let Some(artist) = tag.artist() {
                art = artist.into_owned();
            }
            if let Some(album) = tag.album() {
                alb = album.into_owned();
            }
            // if let Some(duration) = tag.duration() {
            //     let dur = duration;
            // }
        }
        apis.push((
            String::from(tl),
            String::from(art),
            String::from(alb),
            duration_secs,
        ));
    }
    for tag in apis {
        let client = Client::new();
        let url = "https://lrclib.net/api/get";
        let params = [
            ("track_name", tag.0),
            ("artist_name", tag.1),
            ("album_name", tag.2),
            ("duration", tag.3.to_string()), // Передаем число как строку
        ];
        let response = client
            .get(url)
            .query(&params) // reqwest сам соберет ?track_name=...&artist_name=...
            .send()
            .await?;
    }
    Ok(())
}
