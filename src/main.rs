use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::Accessor;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(".") {
        match entry {
            Ok(e) => data.push(e.into_path()),
            Err(err) => eprintln!("Пропущено из-за ошибки: {}", err), // Логируем ошибку, но программа продолжает работать
        }
    }
    let mut apis: Vec<(String, String, String)> = Vec::new();
    for path in data {
        let file = match read_from_path(&path) {
            Ok(f) => f,
            Err(_) => continue,
        };
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
        apis.push((String::from(tl), String::from(art), String::from(alb)));
    }
    Ok(())
}
