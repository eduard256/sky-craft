// Asset downloader: fetches PixelPerfectionCE texture pack from GitHub as a ZIP archive,
// extracts only the required files (block textures + entity/steve.png),
// and reports download/extraction progress via an mpsc channel.
//
// No git required — works on Windows, Mac, Linux.
//
// Asset layout on disk (relative to the executable):
//   assets/textures/block/         <- block PNGs
//   assets/textures/entity/        <- steve.png
//   assets/textures/.downloaded    <- marker file

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

/// URL to the PixelPerfectionCE master branch ZIP (CC-BY-SA-4.0 license).
const TEXTURES_ZIP_URL: &str =
    "https://github.com/Athemis/PixelPerfectionCE/archive/refs/heads/master.zip";

/// Prefix inside the ZIP for all assets.
const ZIP_ASSETS_PREFIX: &str = "PixelPerfectionCE-master/assets/minecraft/textures/";

/// Progress messages sent from the background download thread to the UI thread.
#[derive(Debug)]
pub enum DownloadProgress {
    /// Bytes received so far. `total_bytes` is None when Content-Length is missing.
    Downloading {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    /// Number of files extracted so far out of `total`.
    Extracting { current: usize, total: usize },
    /// Download and extraction finished successfully.
    Done,
    /// A fatal error occurred; message is human-readable.
    Error(String),
}

/// Returns the directory where assets should be stored.
///
/// In development (cargo run from repo root): `<repo>/client/assets/textures`
/// In a release build: `<exe_dir>/assets/textures`
///
/// The heuristic: if `client/assets/textures` exists relative to CWD, use that
/// (developer mode). Otherwise use a sibling `assets/textures` next to the exe.
pub fn assets_dir() -> PathBuf {
    let dev_path = Path::new("client/assets/textures");
    if dev_path.exists() {
        return dev_path.to_path_buf();
    }
    // Release / installed mode: place assets next to the executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("assets/textures");
        }
    }
    // Last resort fallback.
    PathBuf::from("assets/textures")
}

/// Returns true if assets are already present (marker file exists).
pub fn check_assets() -> bool {
    assets_dir().join(".downloaded").exists()
}

/// Downloads and extracts assets in a blocking fashion. Designed to run in `std::thread::spawn`.
///
/// Sends `DownloadProgress` messages over `tx`. Always sends either `Done` or `Error` as the
/// last message before returning, so the UI can stop polling.
pub fn download_assets(tx: Sender<DownloadProgress>) {
    if let Err(e) = download_assets_inner(&tx) {
        let _ = tx.send(DownloadProgress::Error(e.to_string()));
    }
}

fn download_assets_inner(tx: &Sender<DownloadProgress>) -> Result<(), Box<dyn std::error::Error>> {
    let dest_textures = assets_dir();

    // ── Phase 1: Download ZIP ────────────────────────────────────────────────

    let client = reqwest::blocking::Client::builder()
        // Follow redirects (GitHub returns 302 -> codeload.github.com)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let mut response = client.get(TEXTURES_ZIP_URL).send()?.error_for_status()?;

    let total_bytes = response.content_length();

    let mut zip_bytes: Vec<u8> = if let Some(total) = total_bytes {
        Vec::with_capacity(total as usize)
    } else {
        Vec::with_capacity(8 * 1024 * 1024) // 8 MB default capacity
    };

    let mut buf = [0u8; 65536]; // 64 KB read buffer
    loop {
        let n = response.read(&mut buf)?;
        if n == 0 {
            break;
        }
        zip_bytes.extend_from_slice(&buf[..n]);

        let _ = tx.send(DownloadProgress::Downloading {
            downloaded_bytes: zip_bytes.len() as u64,
            total_bytes,
        });
    }

    // ── Phase 2: Count relevant files before extracting (for progress) ───────

    let cursor = std::io::Cursor::new(&zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // Collect indices of files we actually want to extract.
    // We want: block/*.png  AND  entity/steve.png
    let relevant: Vec<usize> = (0..archive.len())
        .filter(|&i| {
            if let Ok(f) = archive.by_index_raw(i) {
                let name = f.name();
                if !name.starts_with(ZIP_ASSETS_PREFIX) {
                    return false;
                }
                let rel = &name[ZIP_ASSETS_PREFIX.len()..];
                // block textures OR entity textures (all mobs)
                (rel.starts_with("block/") && rel.ends_with(".png"))
                    || (rel.starts_with("entity/") && rel.ends_with(".png"))
            } else {
                false
            }
        })
        .collect();

    let total_files = relevant.len();

    // ── Phase 3: Extract ─────────────────────────────────────────────────────

    for (extracted, &idx) in relevant.iter().enumerate() {
        let mut file = archive.by_index(idx)?;
        let raw_name = file.name().to_string();

        // Strip the ZIP prefix to get a path relative to our textures dir.
        // raw_name: "PixelPerfectionCE-master/assets/minecraft/textures/block/stone.png"
        // rel:      "block/stone.png"
        let rel = &raw_name[ZIP_ASSETS_PREFIX.len()..];

        let dest = dest_textures.join(rel);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut out = std::fs::File::create(&dest)?;
        io::copy(&mut file, &mut out)?;

        let _ = tx.send(DownloadProgress::Extracting {
            current: extracted + 1,
            total: total_files,
        });
    }

    // ── Phase 4: Write marker ────────────────────────────────────────────────

    std::fs::File::create(dest_textures.join(".downloaded"))?
        .write_all(b"downloaded")?;

    let _ = tx.send(DownloadProgress::Done);

    Ok(())
}
