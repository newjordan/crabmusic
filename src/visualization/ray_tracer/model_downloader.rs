//! Model downloader for fetching glTF models from the internet
//!
//! This module handles downloading glTF/GLB files from URLs and caching them
//! locally for reuse.

use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Default directory for cached models
const MODELS_DIR: &str = "models";

/// Download a model from a URL and cache it locally
///
/// # Arguments
/// * `url` - The URL to download from
/// * `filename` - The local filename to save as (e.g., "cube.glb")
///
/// # Returns
/// The path to the downloaded file
pub fn download_model(url: &str, filename: &str) -> Result<PathBuf> {
    // Create models directory if it doesn't exist
    let models_dir = Path::new(MODELS_DIR);
    if !models_dir.exists() {
        fs::create_dir_all(models_dir)
            .context("Failed to create models directory")?;
    }

    let file_path = models_dir.join(filename);

    // If file already exists, return it (cached)
    if file_path.exists() {
        println!("✓ Using cached model: {}", file_path.display());
        return Ok(file_path);
    }

    // Download the file
    println!("⬇ Downloading model from: {}", url);
    println!("  Saving to: {}", file_path.display());

    let response = reqwest::blocking::get(url)
        .context("Failed to download model")?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }

    let bytes = response.bytes()
        .context("Failed to read response body")?;

    println!("  Downloaded {} KB", bytes.len() / 1024);

    // Write to file
    let mut file = fs::File::create(&file_path)
        .context("Failed to create file")?;

    file.write_all(&bytes)
        .context("Failed to write file")?;

    println!("✓ Model saved successfully!");

    Ok(file_path)
}

/// Download a model with progress indication
///
/// Similar to `download_model` but with more detailed progress output.
/// For glTF files, also downloads referenced resources (bin files, textures).
pub fn download_model_with_progress(url: &str, filename: &str, model_name: &str) -> Result<PathBuf> {
    let models_dir = Path::new(MODELS_DIR);
    if !models_dir.exists() {
        fs::create_dir_all(models_dir)
            .context("Failed to create models directory")?;
    }

    let file_path = models_dir.join(filename);

    // If file already exists, ensure resources are present for .gltf, then return it
    if file_path.exists() {
        println!("✓ '{}' already cached at: {}", model_name, file_path.display());
        if filename.ends_with(".gltf") {
            if let Ok(bytes) = fs::read(&file_path) {
                // This will fetch any missing .bin/images referenced by the glTF
                if let Err(e) = download_gltf_resources(url, &file_path, &bytes) {
                    eprintln!("Warning: failed to ensure referenced resources for '{}': {}", model_name, e);
                }
            }
        }
        return Ok(file_path);
    }

    // Download the main file
    println!("⬇ Downloading '{}'...", model_name);
    println!("  URL: {}", url);

    let response = reqwest::blocking::get(url)
        .context("Failed to download model")?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }

    let bytes = response.bytes()
        .context("Failed to read response body")?;

    let size_kb = bytes.len() / 1024;
    println!("  Downloaded {} KB", size_kb);

    // Write to file
    let mut file = fs::File::create(&file_path)
        .context("Failed to create file")?;

    file.write_all(&bytes)
        .context("Failed to write file")?;

    println!("✓ '{}' saved to: {}", model_name, file_path.display());

    // If it's a glTF file (not GLB), download referenced resources
    if filename.ends_with(".gltf") {
        download_gltf_resources(url, &file_path, &bytes)?;
    }

    Ok(file_path)
}

/// Download resources referenced by a glTF file (bin files, textures)
fn download_gltf_resources(base_url: &str, gltf_path: &Path, gltf_content: &[u8]) -> Result<()> {
    // Parse the glTF JSON to find referenced resources
    let gltf_json: serde_json::Value = serde_json::from_slice(gltf_content)
        .context("Failed to parse glTF JSON")?;

    // Get the base URL (directory containing the glTF file)
    let base_url = base_url.rsplit_once('/').map(|(base, _)| base).unwrap_or(base_url);
    let models_dir = gltf_path.parent().unwrap();

    // Download buffer files (.bin)
    if let Some(buffers) = gltf_json.get("buffers").and_then(|b| b.as_array()) {
        for buffer in buffers {
            if let Some(uri) = buffer.get("uri").and_then(|u| u.as_str()) {
                // Skip data URIs (embedded data)
                if !uri.starts_with("data:") {
                    let resource_url = format!("{}/{}", base_url, uri);
                    let resource_path = models_dir.join(uri);

                    if !resource_path.exists() {
                        println!("  ⬇ Downloading resource: {}", uri);
                        download_resource(&resource_url, &resource_path)?;
                    }
                }
            }
        }
    }

    // Download image files (.png, .jpg, etc.)
    if let Some(images) = gltf_json.get("images").and_then(|i| i.as_array()) {
        for image in images {
            if let Some(uri) = image.get("uri").and_then(|u| u.as_str()) {
                // Skip data URIs (embedded data)
                if !uri.starts_with("data:") {
                    let resource_url = format!("{}/{}", base_url, uri);
                    let resource_path = models_dir.join(uri);

                    if !resource_path.exists() {
                        println!("  ⬇ Downloading resource: {}", uri);
                        download_resource(&resource_url, &resource_path)?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Download a single resource file
fn download_resource(url: &str, path: &Path) -> Result<()> {
    let response = reqwest::blocking::get(url)
        .with_context(|| format!("Failed to download resource from {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!("Resource download failed with status: {}", response.status());
    }

    let bytes = response.bytes()
        .context("Failed to read resource")?;

    // Ensure parent directories exist for nested paths like "textures/foo.png"
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directories for {}", parent.display()))?;
    }

    let mut file = fs::File::create(path)
        .with_context(|| format!("Failed to create file: {}", path.display()))?;

    file.write_all(&bytes)
        .context("Failed to write resource")?;

    println!("    ✓ Saved: {}", path.file_name().unwrap().to_string_lossy());

    Ok(())
}

/// Get the path to a cached model (without downloading)
pub fn get_cached_model_path(filename: &str) -> Option<PathBuf> {
    let file_path = Path::new(MODELS_DIR).join(filename);
    if file_path.exists() {
        Some(file_path)
    } else {
        None
    }
}

/// Check if a model is cached locally
pub fn is_model_cached(filename: &str) -> bool {
    get_cached_model_path(filename).is_some()
}

/// Extract a filename from a URL (last path segment). Fallbacks to "model.gltf".
pub fn filename_from_url(url: &str) -> String {
    // Trim query/fragment if any
    let no_frag = url.split('#').next().unwrap_or(url);
    let no_query = no_frag.split('?').next().unwrap_or(no_frag);
    no_query
        .rsplit('/')
        .next()
        .unwrap_or("model.gltf")
        .to_string()
}


/// List all cached models
pub fn list_cached_models() -> Result<Vec<PathBuf>> {
    let models_dir = Path::new(MODELS_DIR);

    if !models_dir.exists() {
        return Ok(Vec::new());
    }

    let mut models = Vec::new();

    for entry in fs::read_dir(models_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "glb" || ext == "gltf" {
                    models.push(path);
                }
            }
        }
    }

    Ok(models)
}

/// Clear the model cache (delete all downloaded models)
pub fn clear_cache() -> Result<()> {
    let models_dir = Path::new(MODELS_DIR);

    if !models_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(models_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete {}", path.display()))?;
        }
    }

    println!("✓ Cache cleared");
    Ok(())
}

/// Get the total size of cached models in bytes
pub fn get_cache_size() -> Result<u64> {
    let models_dir = Path::new(MODELS_DIR);

    if !models_dir.exists() {
        return Ok(0);
    }

    let mut total_size = 0u64;

    for entry in fs::read_dir(models_dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            total_size += metadata.len();
        }
    }

    Ok(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_models_dir_constant() {
        assert_eq!(MODELS_DIR, "models");
    }

    #[test]
    fn test_is_model_cached() {
        // This test just checks the function doesn't panic
        // Actual caching behavior depends on filesystem state
        let _ = is_model_cached("nonexistent.glb");
    }

    #[test]
    fn test_get_cache_size() {
        // Should not panic even if directory doesn't exist
        let result = get_cache_size();
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_cached_models() {
        // Should not panic even if directory doesn't exist
        let result = list_cached_models();
        assert!(result.is_ok());
    }
}

