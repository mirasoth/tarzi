//! Tavily Search API client and JSON parser.

use crate::Result;
use crate::constants::TAVILY_API_URL;
use crate::error::TarziError;
use crate::fetcher::WebFetcher;
use crate::search::types::SearchResult;
use serde_json::{Value, json};

const TAVILY_AUTH_HEADER: &str = "Authorization";

/// Perform a Tavily Search API query and parse results.
pub async fn search_tavily_api(
    fetcher: &WebFetcher,
    query: &str,
    limit: usize,
    api_key: &str,
) -> Result<Vec<SearchResult>> {
    let body = json!({
        "query": query,
        "max_results": limit,
        "include_answer": true,
        "search_depth": "basic",
    });

    let auth = format!("Bearer {api_key}");
    let response = fetcher
        .fetch_post_json_with_headers(
            TAVILY_API_URL,
            &[
                (TAVILY_AUTH_HEADER, auth.as_str()),
                ("Content-Type", "application/json"),
            ],
            &body,
        )
        .await?;

    parse_tavily_api_response(&response, limit)
}

/// Parse Tavily Search API JSON into search results.
///
/// When an `answer` field is present, it is emitted as the first result with an
/// empty URL so callers can surface the synthesized answer without a schema change.
pub fn parse_tavily_api_response(json: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| TarziError::Search(format!("Failed to parse Tavily API JSON: {e}")))?;

    let mut out = Vec::new();

    if let Some(answer) = value.get("answer").and_then(|v| v.as_str())
        && !answer.is_empty()
    {
        out.push(SearchResult {
            title: "Answer".to_string(),
            url: String::new(),
            snippet: answer.to_string(),
            rank: 1,
        });
    }

    let results = value
        .get("results")
        .and_then(|r| r.as_array())
        .ok_or_else(|| TarziError::Search("Tavily API response missing results".to_string()))?;

    let remaining = limit.saturating_sub(out.len());
    for item in results.iter().take(remaining) {
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
            rank: out.len() + 1,
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tavily_api_response() {
        let json = r#"{
            "answer": "Rust is a systems language.",
            "results": [
                {
                    "title": "Rust Lang",
                    "url": "https://www.rust-lang.org/",
                    "content": "A language empowering everyone"
                },
                {
                    "title": "Rust Book",
                    "url": "https://doc.rust-lang.org/book/",
                    "content": "The Rust Programming Language"
                }
            ]
        }"#;

        let results = parse_tavily_api_response(json, 10).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].title, "Answer");
        assert_eq!(results[0].snippet, "Rust is a systems language.");
        assert_eq!(results[0].url, "");
        assert_eq!(results[1].title, "Rust Lang");
        assert_eq!(results[1].url, "https://www.rust-lang.org/");
        assert_eq!(results[2].title, "Rust Book");
    }

    #[test]
    fn test_parse_tavily_respects_limit_including_answer() {
        let json = r#"{
            "answer": "yes",
            "results": [
                {"title": "A", "url": "https://a.example", "content": "a"},
                {"title": "B", "url": "https://b.example", "content": "b"},
                {"title": "C", "url": "https://c.example", "content": "c"}
            ]
        }"#;
        let results = parse_tavily_api_response(json, 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Answer");
        assert_eq!(results[1].title, "A");
    }

    #[test]
    fn test_parse_tavily_without_answer() {
        let json = r#"{
            "results": [
                {"title": "A", "url": "https://a.example", "content": "a"}
            ]
        }"#;
        let results = parse_tavily_api_response(json, 5).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rank, 1);
    }

    #[test]
    fn test_parse_tavily_missing_results() {
        let err = parse_tavily_api_response(r#"{"answer": "x"}"#, 5);
        assert!(err.is_err());
    }
}
