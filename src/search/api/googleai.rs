//! Gemini (Google AI) grounded search via the Google Search tool.

use crate::Result;
use crate::constants::{DEFAULT_GOOGLEAI_MODEL, GOOGLEAI_API_URL_PATTERN};
use crate::error::TarziError;
use crate::fetcher::WebFetcher;
use crate::search::types::SearchResult;
use serde_json::{Value, json};

/// Perform a Gemini generateContent request with the Google Search tool enabled.
pub async fn search_googleai_api(
    fetcher: &WebFetcher,
    query: &str,
    limit: usize,
    api_key: &str,
) -> Result<Vec<SearchResult>> {
    search_googleai_api_with_model(fetcher, query, limit, api_key, DEFAULT_GOOGLEAI_MODEL).await
}

/// Same as [`search_googleai_api`] with an explicit model id.
pub async fn search_googleai_api_with_model(
    fetcher: &WebFetcher,
    query: &str,
    limit: usize,
    api_key: &str,
    model: &str,
) -> Result<Vec<SearchResult>> {
    let url = format!(
        "{}?key={}",
        GOOGLEAI_API_URL_PATTERN.replace("{model}", model),
        urlencoding::encode(api_key)
    );

    let prompt = format!(
        "Search the web for current, credible information about the following query. \
         Summarize key findings briefly.\n\nQuery: {query}"
    );

    let body = json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }],
        "tools": [{
            "google_search": {}
        }],
        "generationConfig": {
            "temperature": 0.0
        }
    });

    let response = fetcher
        .fetch_post_json_with_headers(&url, &[("Content-Type", "application/json")], &body)
        .await?;

    parse_googleai_api_response(&response, limit)
}

/// Parse Gemini generateContent JSON into search results.
///
/// Grounding chunks become ranked URL results. When present, the model text is
/// prepended as an "Answer" row (empty URL), matching the Tavily convention.
pub fn parse_googleai_api_response(json: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| TarziError::Search(format!("Failed to parse Google AI API JSON: {e}")))?;

    let candidate = value
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| TarziError::Search("Google AI response missing candidates".to_string()))?;

    let answer_text = extract_answer_text(candidate);
    let chunks = candidate
        .get("groundingMetadata")
        .and_then(|m| m.get("groundingChunks"))
        .and_then(|c| c.as_array());

    let mut out = Vec::new();

    if let Some(answer) = answer_text.as_deref()
        && !answer.is_empty()
    {
        out.push(SearchResult {
            title: "Answer".to_string(),
            url: String::new(),
            snippet: answer.to_string(),
            rank: 1,
        });
    }

    if let Some(chunks) = chunks {
        let remaining = limit.saturating_sub(out.len());
        let mut seen = std::collections::HashSet::new();
        for item in chunks.iter().take(remaining.saturating_mul(2)) {
            if out.len() >= limit {
                break;
            }
            let web = item.get("web").unwrap_or(item);
            let title = web
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let url = web
                .get("uri")
                .or_else(|| web.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if url.is_empty() || !seen.insert(url.clone()) {
                continue;
            }
            out.push(SearchResult {
                title: if title.is_empty() { url.clone() } else { title },
                url,
                snippet: String::new(),
                rank: out.len() + 1,
            });
        }
    }

    if out.is_empty() {
        return Err(TarziError::Search(
            "Google AI response contained no answer or grounding chunks".to_string(),
        ));
    }

    // Normalize ranks
    for (i, item) in out.iter_mut().enumerate() {
        item.rank = i + 1;
    }

    Ok(out.into_iter().take(limit).collect())
}

fn extract_answer_text(candidate: &Value) -> Option<String> {
    let parts = candidate
        .get("content")
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())?;
    let mut texts = Vec::new();
    for part in parts {
        if let Some(text) = part.get("text").and_then(|t| t.as_str())
            && !text.is_empty()
        {
            texts.push(text);
        }
    }
    if texts.is_empty() {
        None
    } else {
        Some(texts.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_googleai_with_answer_and_chunks() {
        let json = r#"{
            "candidates": [{
                "content": {
                    "parts": [{"text": "Rust is a systems programming language."}]
                },
                "groundingMetadata": {
                    "groundingChunks": [
                        {"web": {"uri": "https://www.rust-lang.org/", "title": "Rust"}},
                        {"web": {"uri": "https://doc.rust-lang.org/book/", "title": "The Book"}}
                    ]
                }
            }]
        }"#;

        let results = parse_googleai_api_response(json, 10).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].title, "Answer");
        assert!(results[0].snippet.contains("systems programming"));
        assert_eq!(results[1].url, "https://www.rust-lang.org/");
        assert_eq!(results[2].title, "The Book");
    }

    #[test]
    fn test_parse_googleai_dedupes_urls() {
        let json = r#"{
            "candidates": [{
                "content": {"parts": [{"text": "ok"}]},
                "groundingMetadata": {
                    "groundingChunks": [
                        {"web": {"uri": "https://example.com", "title": "A"}},
                        {"web": {"uri": "https://example.com", "title": "B"}}
                    ]
                }
            }]
        }"#;
        let results = parse_googleai_api_response(json, 10).unwrap();
        assert_eq!(results.len(), 2); // answer + one unique url
        assert_eq!(results[1].url, "https://example.com");
    }

    #[test]
    fn test_parse_googleai_answer_only() {
        let json = r#"{
            "candidates": [{
                "content": {"parts": [{"text": "Only an answer"}]},
                "groundingMetadata": {"groundingChunks": []}
            }]
        }"#;
        let results = parse_googleai_api_response(json, 5).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].snippet, "Only an answer");
    }

    #[test]
    fn test_parse_googleai_missing_candidates() {
        let err = parse_googleai_api_response(r#"{"candidates": []}"#, 5);
        assert!(err.is_err());
    }

    #[test]
    fn test_parse_googleai_respects_limit() {
        let json = r#"{
            "candidates": [{
                "content": {"parts": [{"text": "ans"}]},
                "groundingMetadata": {
                    "groundingChunks": [
                        {"web": {"uri": "https://a.example", "title": "A"}},
                        {"web": {"uri": "https://b.example", "title": "B"}},
                        {"web": {"uri": "https://c.example", "title": "C"}}
                    ]
                }
            }]
        }"#;
        let results = parse_googleai_api_response(json, 2).unwrap();
        assert_eq!(results.len(), 2);
    }
}
