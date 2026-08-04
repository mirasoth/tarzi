//! Serper (Google) Search API client and JSON parser.

use crate::Result;
use crate::constants::SERPER_API_URL;
use crate::error::TarziError;
use crate::fetcher::WebFetcher;
use crate::search::types::SearchResult;
use serde_json::{Value, json};

const SERPER_API_KEY_HEADER: &str = "X-API-KEY";

/// Perform a Serper web search and parse organic results.
pub async fn search_serper_api(
    fetcher: &WebFetcher,
    query: &str,
    limit: usize,
    api_key: &str,
) -> Result<Vec<SearchResult>> {
    let body = json!({
        "q": query,
        "num": limit,
    });

    let response = fetcher
        .fetch_post_json_with_headers(
            SERPER_API_URL,
            &[
                (SERPER_API_KEY_HEADER, api_key),
                ("Content-Type", "application/json"),
            ],
            &body,
        )
        .await?;

    parse_serper_api_response(&response, limit)
}

/// Parse Serper API JSON into search results (organic results).
pub fn parse_serper_api_response(json: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| TarziError::Search(format!("Failed to parse Serper API JSON: {e}")))?;

    let results = value
        .get("organic")
        .and_then(|r| r.as_array())
        .ok_or_else(|| {
            TarziError::Search("Serper API response missing organic results".to_string())
        })?;

    let mut out = Vec::new();
    for (i, item) in results.iter().take(limit).enumerate() {
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = item
            .get("link")
            .or_else(|| item.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let snippet = item
            .get("snippet")
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
    fn test_parse_serper_api_response() {
        let json = r#"{
            "organic": [
                {
                    "title": "Python.org",
                    "link": "https://www.python.org/",
                    "snippet": "The official Python website"
                },
                {
                    "title": "Python Docs",
                    "link": "https://docs.python.org/",
                    "snippet": "Documentation"
                }
            ]
        }"#;

        let results = parse_serper_api_response(json, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Python.org");
        assert_eq!(results[0].url, "https://www.python.org/");
        assert_eq!(results[0].snippet, "The official Python website");
        assert_eq!(results[0].rank, 1);
    }

    #[test]
    fn test_parse_serper_respects_limit() {
        let json = r#"{
            "organic": [
                {"title": "A", "link": "https://a.example", "snippet": "a"},
                {"title": "B", "link": "https://b.example", "snippet": "b"},
                {"title": "C", "link": "https://c.example", "snippet": "c"}
            ]
        }"#;
        let results = parse_serper_api_response(json, 1).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_parse_serper_missing_organic() {
        let err = parse_serper_api_response(r#"{"searchParameters": {}}"#, 5);
        assert!(err.is_err());
    }
}
