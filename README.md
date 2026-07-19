# LRC Downloader

A fast, asynchronous Rust CLI utility that scans your local music library, fetches missing `.lrc` lyrics (synchronized or plain text) using the [LRCLIB API](https://lrclib.net/), and can optionally trigger a Jellyfin library refresh.

## Features

* **Automatic Lyrics Downloading:** Recursively scans directories for supported audio formats (`flac`, `mp3`, `m4a`, `ogg`, `wav`).
* **Smart Skipping:** Ignores tracks that already have an `.lrc` file next to them.
* **LRCLIB Integration:** Reads internal metadata via `lofty` and matches tracks accurately using Artist, Album, Title, and Duration.
* **Rate Limiting:** Implements safe request pacing (150ms delays) to respect LRCLIB's rate limits.
* **Configurable Jellyfin Trigger:** Optionally send a webhook refresh request to a Jellyfin instance of your choice once processing is complete.

## Prerequisites & Configuration

* **Rust Toolchain:** (Cargo & Rustc)
* **Jellyfin API Key (Optional):** If you plan to use the Jellyfin auto-refresh feature, you need to provide your API token. 

The application looks for a file named `api.txt` containing just your token in the following order:
1. In the **current working directory** (where you run the tool).
2. In your OS-specific **user configuration directory**:
   * **Linux:** `~/.config/lrc_downloader/api.txt`
   * **Windows:** `C:\Users\<USER>\AppData\Roaming\quix\lrc_downloader\config\api.txt`
   * **macOS:** `~/Library/Application Support/com.quix.lrc_downloader/api.txt`

## Build Instructions

### Standard Build (Cargo)

If you have Rust installed via `rustup` or your system package manager:

1. Clone the repository and navigate into the project directory.
2. Build the project in release mode for optimal performance:
```bash
cargo build --release
```

## Usage

Run the compiled binary, providing the path to your music directory using the `-p` or `--path` flag. If no path is provided, it defaults to the current directory.

Bash

```
# Scan a specific directory (lyrics only, no Jellyfin sync)
./target/release/lrc_downloader --path /path/to/your/music/folder

# Scan and trigger Jellyfin update on the default local port (http://localhost:8096)
./target/release/lrc_downloader --path /path/to/music --jellyfin

# Scan and trigger Jellyfin update on a custom URL
./target/release/lrc_downloader --path /path/to/music -j --jellyfin-url "[http://192.168.1.100:8096](http://192.168.1.100:8096)"
```

### CLI Options

- `-p, --path <PATH>` : Path to your music library (default: `.`)
    
- `-j, --jellyfin` : Enable sending a refresh request to Jellyfin after scanning
    
- `-u, --jellyfin-url <URL>` : Set a custom Jellyfin server URL (default: `http://localhost:8096`)
    
- `-h, --help` : Print help information
    
- `-V, --version` : Print version information
