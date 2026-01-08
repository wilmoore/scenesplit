//! Model download and caching module.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};

use crate::error::{Error, Result};

/// ResNet50 ONNX model from ONNX Model Zoo (feature extraction variant).
/// This is the standard ResNet50 with the final classification layer removed.
const MODEL_URL: &str = "https://github.com/onnx/models/raw/main/validated/vision/classification/resnet/model/resnet50-v2-7.onnx";

/// Expected model filename.
const MODEL_FILENAME: &str = "resnet50-v2-7.onnx";

/// Get the cache directory for SceneSplit.
fn cache_dir() -> Result<PathBuf> {
    let base = dirs::cache_dir()
        .ok_or_else(|| Error::ModelLoad("Could not determine cache directory".to_string()))?;
    Ok(base.join("scenesplit"))
}

/// Get the path to the cached model, downloading if necessary.
///
/// Returns the path to the ONNX model file, downloading it on first run.
pub fn ensure_model(quiet: bool) -> Result<PathBuf> {
    let cache = cache_dir()?;
    let model_path = cache.join(MODEL_FILENAME);

    if model_path.exists() {
        return Ok(model_path);
    }

    // Create cache directory
    fs::create_dir_all(&cache)
        .map_err(|e| Error::ModelLoad(format!("Failed to create cache directory: {}", e)))?;

    if !quiet {
        eprintln!("Downloading model (one-time, ~100MB)...");
    }

    download_model(MODEL_URL, &model_path, quiet)?;

    Ok(model_path)
}

/// Download the model file with progress indication.
fn download_model(url: &str, dest: &PathBuf, quiet: bool) -> Result<()> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| Error::ModelLoad(format!("Failed to download model: {}", e)))?;

    let total_size = response
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let pb = if !quiet && total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    // Download to temporary file first (atomic write)
    let temp_path = dest.with_extension("tmp");
    let mut file = File::create(&temp_path)
        .map_err(|e| Error::ModelLoad(format!("Failed to create temp file: {}", e)))?;

    let mut reader = response.into_reader();
    let mut buffer = [0u8; 8192];
    let mut downloaded: u64 = 0;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| Error::ModelLoad(format!("Failed to read response: {}", e)))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| Error::ModelLoad(format!("Failed to write to file: {}", e)))?;

        downloaded += bytes_read as u64;

        if let Some(ref pb) = pb {
            pb.set_position(downloaded);
        }
    }

    if let Some(pb) = pb {
        pb.finish_with_message("Download complete");
    }

    // Atomic rename
    fs::rename(&temp_path, dest)
        .map_err(|e| Error::ModelLoad(format!("Failed to move model to cache: {}", e)))?;

    if !quiet {
        eprintln!("Model cached at: {}", dest.display());
    }

    Ok(())
}

/// Get the path where the model would be cached (for display purposes).
#[allow(dead_code)]
pub fn model_cache_path() -> Option<PathBuf> {
    cache_dir().ok().map(|c| c.join(MODEL_FILENAME))
}
