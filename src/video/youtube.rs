use anyhow::{anyhow, Context, Result};
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::process::Command;
use url::Url;

const YTDLP_PROGRESSIVE_FORMAT: &str =
    "best[acodec!=none][vcodec!=none][height<=720]/best[acodec!=none][vcodec!=none]/best[height<=720]/best";

#[derive(Debug, Clone)]
pub struct YoutubeResolvedStream {
    pub title: String,
    pub webpage_url: Url,
    pub stream_url: Url,
}

#[derive(Debug, Default, Deserialize)]
struct FlatPlaylistResponse {
    #[serde(default)]
    entries: Vec<FlatPlaylistEntry>,
}

#[derive(Debug, Default, Deserialize)]
struct FlatPlaylistEntry {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    webpage_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct VideoMetadata {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    webpage_url: Option<String>,
}

pub fn is_youtube_url(input: &str) -> bool {
    let Ok(url) = Url::parse(input) else {
        return false;
    };

    matches!(
        url.host_str().map(|host| host.to_ascii_lowercase()),
        Some(host)
            if host == "youtube.com"
                || host == "www.youtube.com"
                || host == "m.youtube.com"
                || host == "youtu.be"
    )
}

pub fn resolve_youtube_input(input: &str) -> Result<Option<YoutubeResolvedStream>> {
    if !is_youtube_url(input) {
        return Ok(None);
    }

    let canonical_video_url = if is_collection_url(input)? {
        choose_random_video_from_collection(input)?
    } else {
        input.to_string()
    };

    Ok(Some(resolve_direct_video(&canonical_video_url)?))
}

/// Fetches all video URLs from a YouTube channel or playlist, picks one at random,
/// and returns its best quality streamable URL.
#[allow(dead_code)]
pub async fn get_random_video_stream_url(channel_url: &str) -> Result<Url> {
    let video_url = choose_random_video_from_collection(channel_url)?;
    Ok(resolve_direct_video(&video_url)?.stream_url)
}

fn is_collection_url(input: &str) -> Result<bool> {
    let url = Url::parse(input)?;
    let path = url.path().trim_end_matches('/');
    Ok(path.starts_with("/channel/")
        || path.starts_with("/user/")
        || path.starts_with("/c/")
        || path.starts_with("/@")
        || path == "/playlist"
        || url.query_pairs().any(|(key, _)| key == "list"))
}

fn choose_random_video_from_collection(input: &str) -> Result<String> {
    let stdout = run_yt_dlp(&[
        "--flat-playlist",
        "--dump-single-json",
        "--playlist-end",
        "50",
        "--skip-download",
        "--no-warnings",
        input,
    ])?;
    let listing: FlatPlaylistResponse =
        serde_json::from_str(&stdout).context("Failed to parse yt-dlp playlist JSON")?;

    let mut rng = rand::thread_rng();
    let entry = listing
        .entries
        .iter()
        .filter_map(|entry| {
            entry
                .webpage_url
                .clone()
                .or_else(|| entry.id.as_ref().map(|video_id| build_watch_url(video_id)))
        })
        .collect::<Vec<_>>()
        .choose(&mut rng)
        .cloned()
        .ok_or_else(|| anyhow!("yt-dlp returned no playable YouTube videos for {input}"))?;

    Ok(entry)
}

fn resolve_direct_video(input: &str) -> Result<YoutubeResolvedStream> {
    let metadata_stdout = run_yt_dlp(&[
        "--dump-single-json",
        "--no-playlist",
        "--skip-download",
        "--no-warnings",
        input,
    ])?;
    let metadata: VideoMetadata = serde_json::from_str(&metadata_stdout)
        .context("Failed to parse yt-dlp video metadata JSON")?;

    let stream_stdout = run_yt_dlp(&[
        "--get-url",
        "--no-playlist",
        "--no-warnings",
        "-f",
        YTDLP_PROGRESSIVE_FORMAT,
        input,
    ])?;
    let stream_url_raw = stream_stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .context("yt-dlp did not return a playable stream URL")?;

    let webpage_url = metadata.webpage_url.as_deref().unwrap_or(input);

    Ok(YoutubeResolvedStream {
        title: metadata
            .title
            .unwrap_or_else(|| "YouTube Video".to_string()),
        webpage_url: Url::parse(webpage_url)
            .with_context(|| format!("Invalid YouTube webpage URL from yt-dlp: {webpage_url}"))?,
        stream_url: Url::parse(stream_url_raw.trim())
            .with_context(|| format!("Invalid yt-dlp stream URL: {stream_url_raw}"))?,
    })
}

fn run_yt_dlp(args: &[&str]) -> Result<String> {
    let output = Command::new("yt-dlp")
        .args(args)
        .output()
        .context("Failed to launch yt-dlp. Install yt-dlp to resolve YouTube URLs.")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!(
            "yt-dlp failed while resolving YouTube input{}",
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        ));
    }

    String::from_utf8(output.stdout).context("yt-dlp output was not valid UTF-8")
}

fn build_watch_url(video_id: &str) -> String {
    format!("https://www.youtube.com/watch?v={video_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youtube_url_detection_handles_common_hosts() {
        assert!(is_youtube_url(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        ));
        assert!(is_youtube_url("https://youtu.be/dQw4w9WgXcQ"));
        assert!(!is_youtube_url("https://archive.org/details/example"));
    }

    #[test]
    fn collection_detection_handles_channels_and_playlists() {
        assert!(is_collection_url("https://www.youtube.com/@example").unwrap());
        assert!(is_collection_url("https://www.youtube.com/playlist?list=PL123").unwrap());
        assert!(!is_collection_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap());
    }

    #[test]
    fn build_watch_url_wraps_video_ids() {
        assert_eq!(
            build_watch_url("abc123"),
            "https://www.youtube.com/watch?v=abc123"
        );
    }
}
