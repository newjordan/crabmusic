use anyhow::{anyhow, Context, Result};
use rand::{seq::SliceRandom, Rng};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone)]
pub struct ArchiveStream {
    pub title: String,
    pub identifier: String,
    pub stream_url: Url,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    response: SearchResultPage,
}

#[derive(Debug, Deserialize)]
struct SearchResultPage {
    #[serde(rename = "numFound")]
    num_found: usize,
    #[serde(default)]
    docs: Vec<SearchDoc>,
}

#[derive(Debug, Deserialize)]
struct SearchDoc {
    identifier: String,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataResponse {
    #[serde(default)]
    metadata: ArchiveMetadata,
    #[serde(default)]
    files: Vec<ArchiveFile>,
}

#[derive(Debug, Default, Deserialize)]
struct ArchiveMetadata {
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArchiveFile {
    name: String,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

#[allow(dead_code)]
pub fn get_random_video_stream(
    queries: &[String],
    rows_per_page: usize,
    max_pages: usize,
) -> Result<ArchiveStream> {
    get_random_video_stream_excluding(queries, rows_per_page, max_pages, None)
}

pub fn get_random_video_stream_excluding(
    queries: &[String],
    rows_per_page: usize,
    max_pages: usize,
    excluded_identifier: Option<&str>,
) -> Result<ArchiveStream> {
    if queries.is_empty() {
        return Err(anyhow!("No Internet Archive queries configured"));
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent(format!("crabmusic/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .context("Failed to build HTTP client")?;

    let mut rng = rand::thread_rng();
    let query = queries
        .choose(&mut rng)
        .context("No Internet Archive queries configured")?;

    let count_page = search_page(&client, query, 1, 1)?;
    let num_found = count_page.response.num_found;
    if num_found == 0 {
        return Err(anyhow!(
            "Internet Archive query returned no results: {query}"
        ));
    }

    let total_pages = num_found.div_ceil(rows_per_page).max(1);
    let page = rng.gen_range(1..=total_pages.min(max_pages));
    let mut page_results = search_page(&client, query, rows_per_page, page)?;

    if page_results.response.docs.is_empty() {
        return Err(anyhow!(
            "Internet Archive query returned an empty result page"
        ));
    }

    page_results.response.docs.shuffle(&mut rng);

    for doc in page_results.response.docs.into_iter() {
        if excluded_identifier == Some(doc.identifier.as_str()) {
            continue;
        }
        let metadata = fetch_metadata(&client, &doc.identifier)?;
        if let Some(file) = pick_streamable_file(&metadata.files) {
            let stream_url = build_download_url(&doc.identifier, &file.name)?;
            let title = metadata
                .metadata
                .title
                .or(doc.title)
                .unwrap_or_else(|| doc.identifier.clone());

            return Ok(ArchiveStream {
                title,
                identifier: doc.identifier,
                stream_url,
            });
        }
    }

    Err(anyhow!(
        "Couldn't find a streamable video on the sampled Internet Archive items"
    ))
}

fn search_page(client: &Client, query: &str, rows: usize, page: usize) -> Result<SearchResponse> {
    let response = client
        .get("https://archive.org/advancedsearch.php")
        .query(&[
            ("q", query),
            ("fl[]", "identifier"),
            ("fl[]", "title"),
            ("rows", &rows.to_string()),
            ("page", &page.to_string()),
            ("output", "json"),
        ])
        .send()
        .with_context(|| format!("Internet Archive search request failed for query: {query}"))?
        .error_for_status()
        .context("Internet Archive search returned an error status")?;

    let body = response
        .text()
        .context("Failed to read Internet Archive search response")?;
    serde_json::from_str(&body).context("Failed to parse Internet Archive search response")
}

fn fetch_metadata(client: &Client, identifier: &str) -> Result<MetadataResponse> {
    let response = client
        .get(format!("https://archive.org/metadata/{identifier}"))
        .send()
        .with_context(|| format!("Failed to fetch Internet Archive metadata for {identifier}"))?
        .error_for_status()
        .with_context(|| format!("Internet Archive metadata returned an error for {identifier}"))?;

    let body = response
        .text()
        .with_context(|| format!("Failed to read metadata body for {identifier}"))?;
    serde_json::from_str(&body)
        .with_context(|| format!("Failed to parse metadata JSON for {identifier}"))
}

fn pick_streamable_file(files: &[ArchiveFile]) -> Option<&ArchiveFile> {
    files
        .iter()
        .filter(|file| stream_priority(file).0 < 9)
        .min_by_key(|file| stream_priority(file))
}

fn stream_priority(file: &ArchiveFile) -> (u8, u8, String) {
    let name = file.name.to_ascii_lowercase();
    let format = file
        .format
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let source = file
        .source
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    let container_rank = if format.contains("h.264")
        || format.contains("mpeg4")
        || name.ends_with(".mp4")
        || name.ends_with(".m4v")
    {
        0
    } else if format.contains("ogg video") || name.ends_with(".ogv") || name.ends_with(".webm") {
        1
    } else if name.ends_with(".mov")
        || name.ends_with(".avi")
        || name.ends_with(".mkv")
        || name.ends_with(".mpg")
        || name.ends_with(".mpeg")
    {
        2
    } else {
        9
    };

    let source_rank = if source == "derivative" { 0 } else { 1 };
    (container_rank, source_rank, name)
}

fn build_download_url(identifier: &str, file_name: &str) -> Result<Url> {
    let base = Url::parse("https://archive.org/download/")?.join(&format!("{identifier}/"))?;
    base.join(file_name)
        .with_context(|| format!("Failed to build download URL for {identifier}/{file_name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_derivative_mp4_streams() {
        let files = vec![
            ArchiveFile {
                name: "clip.m4v".into(),
                format: Some("MPEG4".into()),
                source: Some("original".into()),
            },
            ArchiveFile {
                name: "clip.mp4".into(),
                format: Some("h.264".into()),
                source: Some("derivative".into()),
            },
        ];

        let picked = pick_streamable_file(&files).unwrap();
        assert_eq!(picked.name, "clip.mp4");
    }

    #[test]
    fn ignores_non_video_files() {
        let files = vec![
            ArchiveFile {
                name: "thumb.jpg".into(),
                format: Some("Thumbnail".into()),
                source: Some("derivative".into()),
            },
            ArchiveFile {
                name: "movie.ogv".into(),
                format: Some("Ogg Video".into()),
                source: Some("derivative".into()),
            },
        ];

        let picked = pick_streamable_file(&files).unwrap();
        assert_eq!(picked.name, "movie.ogv");
    }

    #[test]
    fn download_url_escapes_spaces() {
        let url = build_download_url("my_item", "folder/My Clip 01.mp4").unwrap();
        assert_eq!(
            url.as_str(),
            "https://archive.org/download/my_item/folder/My%20Clip%2001.mp4"
        );
    }

    #[test]
    fn exclusion_check_matches_identifier() {
        let excluded = Some("episode-2");
        assert!(excluded == Some("episode-2"));
        assert!(excluded != Some("episode-3"));
    }
}
