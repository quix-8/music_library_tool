use lofty::file::TaggedFileExt;
use lofty::read_from_path;
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
    for path in data {
        let file = read_from_path(path)?;
        let id3v2 = file.primary_tag();
    }
    Ok(())
}
