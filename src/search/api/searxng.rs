//! SearxNG JSON search API client and parser.

use crate::Result;
use crate::error::TarziError;
use crate::fetcher::WebFetcher;
use crate::search::types::SearchResult;
use serde_json::Value;

/// Normalize a SearxNG host into a search endpoint URL.
///
/// Accepts bare hosts (`searx.example.com`), origins (`https://searx.example.com`),
/// or full search paths (`https://searx.example.com/search`).
pub fn normalize_searx_endpoint(host: &str) -> String {
    let trimmed = host.trim().trim_end_matches('/');
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };

    if with_scheme.ends_with("/search") {
        with_scheme
    } else {
        format!("{with_scheme}/search")
    }
}

/// Perform a SearxNG JSON search against the configured host.
pub async fn search_searxng_api(
    fetcher: &WebFetcher,
    query: &str,
    limit: usize,
    host: &str,
) -> Result<Vec<SearchResult>> {
    let endpoint = normalize_searx_endpoint(host);
    let url = format!(
        "{endpoint}?q={}&format=json&language=en",
        urlencoding::encode(query)
    );

    let body = fetcher.fetch_get_with_headers(&url, &[]).await?;
    parse_searxng_api_response(&body, limit)
}

/// Parse SearxNG JSON search response into search results.
pub fn parse_searxng_api_response(json: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| TarziError::Search(format!("Failed to parse SearxNG JSON: {e}")))?;

    let results = value
        .get("results")
        .and_then(|r| r.as_array())
        .ok_or_else(|| TarziError::Search("SearxNG response missing results".to_string()))?;

    let mut out = Vec::new();
    for (i, item) in results.iter().take(limit).enumerate() {
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = item
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let snippet = item
            .get("content")
            .or_else(|| item.get("snippet"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if title.is_empty() && url.is_empty() {
            continue;
        }

        out.push(SearchResult {
            title,
            url,
            snippet,
            rank: i + 1,
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_searx_endpoint() {
        assert_eq!(
            normalize_searx_endpoint("searx.example.com"),
            "https://searx.example.com/search"
        );
        assert_eq!(
            normalize_searx_endpoint("https://searx.example.com/"),
            "https://searx.example.com/search"
        );
        assert_eq!(
            normalize_searx_endpoint("http://localhost:8080"),
            "http://localhost:8080/search"
        );
        assert_eq!(
            normalize_searx_endpoint("https://searx.example.com/search"),
            "https://searx.example.com/search"
        );
    }

    #[test]
    fn test_parse_searxng_api_response() {
        let json = r#"{
            "results": [
                {
                    "title": "Rust",
                    "url": "https://www.rust-lang.org/",
                    "content": "A language empowering everyone"
                },
                {
                    "title": "Book",
                    "url": "https://doc.rust-lang.org/book/",
                    "content": "The Rust Programming Language"
                }
            ]
        }"#;

        let results = parse_searxng_api_response(json, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Rust");
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[1].url, "https://doc.rust-lang.org/book/");
    }

    #[test]
    fn test_parse_searxng_respects_limit() {
        let json = r#"{
            "results": [
                {"title": "A", "url": "https://a.example", "content": "a"},
                {"title": "B", "url": "https://b.example", "content": "b"},
                {"title": "C", "url": "https://c.example", "content": "c"}
            ]
        }"#;
        let results = parse_searxng_api_response(json, 2).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_parse_searxng_missing_results() {
        let err = parse_searxng_api_response(r#"{"query": "x"}"#, 5);
        assert!(err.is_err());
    }
}
