//! Brave Search API client and JSON parser.

use crate::Result;
use crate::constants::BRAVE_API_QUERY_PATTERN;
use crate::error::TarziError;
use crate::fetcher::WebFetcher;
use crate::search::types::SearchResult;
use serde_json::Value;

const BRAVE_TOKEN_HEADER: &str = "X-Subscription-Token";

/// Perform a Brave Search API query and parse results.
pub async fn search_brave_api(
    fetcher: &WebFetcher,
    query: &str,
    limit: usize,
    api_key: &str,
) -> Result<Vec<SearchResult>> {
    let url = BRAVE_API_QUERY_PATTERN
        .replace("{query}", &urlencoding::encode(query))
        .replace("{limit}", &limit.to_string());

    let body = fetcher
        .fetch_get_with_headers(&url, &[(BRAVE_TOKEN_HEADER, api_key)])
        .await?;

    parse_brave_api_response(&body, limit)
}

/// Parse Brave Search API JSON into search results.
pub fn parse_brave_api_response(json: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| TarziError::Search(format!("Failed to parse Brave API JSON: {e}")))?;

    let results = value
        .get("web")
        .and_then(|w| w.get("results"))
        .and_then(|r| r.as_array())
        .ok_or_else(|| TarziError::Search("Brave API response missing web.results".to_string()))?;

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
            .get("description")
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
    fn test_parse_brave_api_response() {
        let json = r#"{
            "web": {
                "results": [
                    {
                        "title": "Rust Lang",
                        "url": "https://www.rust-lang.org/",
                        "description": "A language empowering everyone"
                    },
                    {
                        "title": "Rust Book",
                        "url": "https://doc.rust-lang.org/book/",
                        "description": "The Rust Programming Language"
                    }
                ]
            }
        }"#;

        let results = parse_brave_api_response(json, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Rust Lang");
        assert_eq!(results[0].url, "https://www.rust-lang.org/");
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[1].title, "Rust Book");
    }

    #[test]
    fn test_parse_brave_api_respects_limit() {
        let json = r#"{
            "web": {
                "results": [
                    {"title": "A", "url": "https://a.example", "description": "a"},
                    {"title": "B", "url": "https://b.example", "description": "b"},
                    {"title": "C", "url": "https://c.example", "description": "c"}
                ]
            }
        }"#;
        let results = parse_brave_api_response(json, 2).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_parse_brave_api_missing_results() {
        let err = parse_brave_api_response(r#"{"web": {}}"#, 5);
        assert!(err.is_err());
    }
}
