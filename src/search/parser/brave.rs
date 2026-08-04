use super::base::{BaseParser, BaseParserImpl};
use crate::Result;
use crate::search::types::{SearchEngineType, SearchResult};
use regex;
use select::document::Document;
use select::predicate::{Class, Name};
use serde_json;

pub struct BraveParser {
    base: BaseParserImpl,
}

impl BraveParser {
    pub fn new() -> Self {
        Self {
            base: BaseParserImpl::new("BraveParser".to_string(), SearchEngineType::BraveSearch),
        }
    }
}

impl BaseParser for BraveParser {
    fn name(&self) -> &str {
        self.base.name()
    }
    fn engine_type(&self) -> SearchEngineType {
        self.base.engine_type()
    }

    fn parse(&self, html: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Brave Search now uses JavaScript-rendered content with JSON data embedded in the HTML
        // We need to extract the JSON data instead of parsing HTML elements

        let mut results = Vec::new();

        // Try to find the JSON data embedded in the HTML
        if let Some(json_results) = self.extract_json_results(html) {
            for (i, json_result) in json_results.iter().enumerate().take(limit) {
                if let Some(search_result) = self.parse_json_result(json_result) {
                    let mut result = search_result;
                    result.rank = i + 1;
                    results.push(result);
                }
            }
        }

        // If no JSON results found, fallback to HTML parsing for backward compatibility
        if results.is_empty() {
            results = self.parse_html_fallback(html, limit)?;
        }

        Ok(results)
    }
}

impl BraveParser {
    fn extract_result_from_node(&self, node: &select::node::Node) -> Option<SearchResult> {
        // Try multiple patterns to extract title and URL
        let mut title = String::new();
        let mut url = String::new();
        let mut snippet = String::new();

        // Look for title in various patterns
        if let Some(title_node) = node
            .find(Name("h3"))
            .next()
            .or_else(|| node.find(Name("h2")).next())
            .or_else(|| node.find(Name("h1")).next())
        {
            if let Some(link) = title_node.find(Name("a")).next() {
                title = link.text().trim().to_string();
                if let Some(href) = link.attr("href") {
                    url = self.normalize_url(href);
                }
            } else {
                title = title_node.text().trim().to_string();
            }
        }

        // If no title found in headers, look for any link
        if title.is_empty()
            && let Some(link) = node.find(Name("a")).next()
        {
            title = link.text().trim().to_string();
            if let Some(href) = link.attr("href") {
                url = self.normalize_url(href);
            }
        }

        // Look for snippet in various patterns
        let snippet_selectors = [
            "p",
            "result-snippet",
            "snippet",
            "description",
            "desc",
            "content",
        ];
        for &sel in &snippet_selectors {
            if sel == "p" {
                if let Some(snippet_node) = node.find(Name("p")).next() {
                    snippet = snippet_node.text().trim().to_string();
                    break;
                }
            } else if let Some(snippet_node) = node.find(Class(sel)).next() {
                snippet = snippet_node.text().trim().to_string();
                break;
            }
        }

        // Only return result if we have at least a title
        if !title.is_empty() && !url.is_empty() {
            Some(SearchResult {
                title,
                url,
                snippet,
                rank: 0, // Will be set later
            })
        } else {
            None
        }
    }

    fn normalize_url(&self, href: &str) -> String {
        if href.starts_with("http") {
            href.to_string()
        } else if href.starts_with("//") {
            format!("https:{href}")
        } else if href.starts_with("/") {
            format!("https://search.brave.com{href}")
        } else {
            href.to_string()
        }
    }

    fn extract_json_results(&self, html: &str) -> Option<Vec<serde_json::Value>> {
        // Prefer explicit injected JSON if available
        if let Some(start) =
            html.find("<script id=\"tarzi-brave-results\" type=\"application/json\">")
            && let Some(close) = html[start..].find("</script>")
        {
            let json_str = &html[start..start + close];
            if let Some(json_start) = json_str.find('>') {
                let payload = &json_str[json_start + 1..];
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload)
                    && let Some(arr) = v.get("results").and_then(|x| x.as_array())
                {
                    let filtered: Vec<_> = arr
                        .iter()
                        .filter(|&r| r.get("title").is_some() && r.get("url").is_some())
                        .cloned()
                        .collect();
                    if !filtered.is_empty() {
                        return Some(filtered);
                    }
                }
            }
        }

        // Look for JSON data embedded in script tags or data attributes
        // The search results are typically embedded in a JavaScript object

        // println!("DEBUG: Looking for JSON patterns in HTML ({} chars)", html.len());

        // Pattern: Look for individual result objects instead of arrays
        if let Some(start) = html.find("{title:") {
            // Find the end of this specific result object
            if let Some(end) = self.find_single_object_end(html, start) {
                let json_str = &html[start..end + 1]; // Include the closing brace

                // Convert JavaScript object notation to JSON
                let json_fixed = self.fix_js_object_to_json(json_str);

                // Try parsing as single object
                if let Ok(single_result) = serde_json::from_str::<serde_json::Value>(&json_fixed)
                    && single_result.get("title").is_some()
                    && single_result.get("url").is_some()
                {
                    return Some(vec![single_result]);
                }
            }
        }

        // Pattern 3: Generic search for result arrays in JSON
        let mut current_pos = 0;
        while let Some(pos) = html[current_pos..].find("\"title\":") {
            let absolute_pos = current_pos + pos;
            // Look backward for array start
            if let Some(array_start) = self.find_array_start(html, absolute_pos)
                && let Some(array_end) = self.find_json_end(html, array_start + 1)
            {
                let json_str = &html[array_start + 1..array_end];
                if let Ok(results) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                    // Filter results that look like search results
                    let filtered: Vec<_> = results
                        .into_iter()
                        .filter(|r| r.get("title").is_some() && r.get("url").is_some())
                        .collect();
                    if !filtered.is_empty() {
                        return Some(filtered);
                    }
                }
            }
            current_pos = absolute_pos + 1;
        }

        None
    }

    fn find_single_object_end(&self, html: &str, start: usize) -> Option<usize> {
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let chars: Vec<char> = html[start..].chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        return Some(start + i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_json_end(&self, html: &str, start: usize) -> Option<usize> {
        let mut bracket_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let chars: Vec<char> = html[start..].chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => escape_next = true,
                '"' => in_string = !in_string,
                '[' if !in_string => bracket_count += 1,
                ']' if !in_string => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        return Some(start + i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn find_array_start(&self, html: &str, from_pos: usize) -> Option<usize> {
        let search_range = from_pos.saturating_sub(1000);
        (search_range..from_pos)
            .rev()
            .find(|&i| html.chars().nth(i) == Some('['))
    }

    fn parse_json_result(&self, json_result: &serde_json::Value) -> Option<SearchResult> {
        let title = json_result.get("title")?.as_str()?.to_string();
        let url = json_result.get("url")?.as_str()?.to_string();

        // Extract description/snippet
        let snippet = json_result
            .get("description")
            .or_else(|| json_result.get("snippet"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Clean up HTML entities in the description
        let snippet = snippet
            .replace("\\u003Cstrong\\u003E", "")
            .replace("\\u003C/strong\\u003E", "")
            .replace("\\u003C", "<")
            .replace("\\u003E", ">")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
            .replace("&amp;", "&");

        if !title.is_empty() && !url.is_empty() {
            Some(SearchResult {
                title,
                url,
                snippet,
                rank: 0, // Will be set later
            })
        } else {
            None
        }
    }

    fn parse_html_fallback(&self, html: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let document = Document::from(html);
        let mut results = Vec::new();

        // Original HTML parsing logic as fallback
        let selectors = [
            "article",         // Simple article tag
            "result",          // Generic result class
            "web-result",      // Alternative class name
            "snippet-content", // Another common pattern
            "fdb-result",      // Feed DB result
            "result-row",      // Original selector
        ];

        for &class_name in &selectors {
            if class_name == "article" {
                // Use Name selector for article tags
                for node in document.find(Name("article")) {
                    if results.len() >= limit {
                        break;
                    }
                    if let Some(result) = self.extract_result_from_node(&node) {
                        results.push(result);
                    }
                }
            } else {
                // Use Class selector for class names
                for node in document.find(Class(class_name)) {
                    if results.len() >= limit {
                        break;
                    }
                    if let Some(result) = self.extract_result_from_node(&node) {
                        results.push(result);
                    }
                }
            }

            if !results.is_empty() {
                break; // Found results with this selector, stop trying others
            }
        }

        // Set proper ranking
        for (i, result) in results.iter_mut().enumerate() {
            result.rank = i + 1;
        }

        Ok(results)
    }

    fn fix_js_object_to_json(&self, js_str: &str) -> String {
        // Convert JavaScript object notation to valid JSON
        // This is a basic conversion for the patterns we expect
        let mut result = js_str.to_string();

        // Convert unquoted property names to quoted ones, but don't double-quote values
        // Look for pattern: word: but not "word":
        result = regex::Regex::new(r"(?:^|[,{\s])([a-zA-Z_][a-zA-Z0-9_]*)\s*:")
            .unwrap()
            .replace_all(&result, |caps: &regex::Captures| {
                let full_match = caps.get(0).unwrap().as_str();
                let prop_name = caps.get(1).unwrap().as_str();
                let prefix = &full_match[..full_match.len() - prop_name.len() - 1];
                format!("{}\"{}\":", prefix, prop_name)
            })
            .to_string();

        // Handle special values
        result = result.replace(":void 0", ":null");
        result = result.replace(":undefined", ":null");

        // Fix boolean values that might not be quoted properly
        result = regex::Regex::new(r":(\s*)(true|false)(\s*[,}\]])")
            .unwrap()
            .replace_all(&result, ":$1$2$3")
            .to_string();

        result
    }
}

impl Default for BraveParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::types::SearchEngineType;

    #[test]
    fn test_brave_parser_article_structure() {
        let parser = BraveParser::new();
        let html = r#"
        <html>
            <body>
                <article>
                    <h3><a href="https://example1.com">Brave Test Result 1</a></h3>
                    <p>This is a test snippet for Brave 1</p>
                </article>
                <article>
                    <h3><a href="https://example2.com">Brave Test Result 2</a></h3>
                    <p>This is a test snippet for Brave 2</p>
                </article>
                <article>
                    <h3><a href="https://example3.com">Brave Test Result 3</a></h3>
                    <p>This is a test snippet for Brave 3</p>
                </article>
            </body>
        </html>
        "#;
        let results = parser.parse(html, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(parser.name(), "BraveParser");
        assert!(parser.supports(&SearchEngineType::BraveSearch));
        assert!(!parser.supports(&SearchEngineType::Google));

        assert_eq!(results[0].title, "Brave Test Result 1");
        assert_eq!(results[0].url, "https://example1.com");
        assert_eq!(results[0].snippet, "This is a test snippet for Brave 1");
        assert_eq!(results[0].rank, 1);

        assert_eq!(results[1].title, "Brave Test Result 2");
        assert_eq!(results[1].url, "https://example2.com");
        assert_eq!(results[1].snippet, "This is a test snippet for Brave 2");
        assert_eq!(results[1].rank, 2);
    }

    #[test]
    fn test_brave_parser_result_class() {
        let parser = BraveParser::new();
        let html = r#"
        <html>
            <body>
                <div class="result">
                    <h2><a href="https://example1.com">Test Result 1</a></h2>
                    <div class="snippet">This is snippet 1</div>
                </div>
                <div class="result">
                    <h2><a href="https://example2.com">Test Result 2</a></h2>
                    <div class="description">This is snippet 2</div>
                </div>
            </body>
        </html>
        "#;
        let results = parser.parse(html, 3).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Test Result 1");
        assert_eq!(results[0].url, "https://example1.com");
        assert_eq!(results[0].snippet, "This is snippet 1");
        assert_eq!(results[0].rank, 1);

        assert_eq!(results[1].title, "Test Result 2");
        assert_eq!(results[1].url, "https://example2.com");
        assert_eq!(results[1].snippet, "This is snippet 2");
        assert_eq!(results[1].rank, 2);
    }

    #[test]
    fn test_brave_parser_original_structure() {
        let parser = BraveParser::new();
        let html = r#"
        <html>
            <body>
                <div class="result-row">
                    <a href="https://example1.com">Brave Test Result 1</a>
                    <div class="result-snippet">This is a test snippet for Brave 1</div>
                </div>
                <div class="result-row">
                    <a href="https://example2.com">Brave Test Result 2</a>
                    <div class="result-snippet">This is a test snippet for Brave 2</div>
                </div>
            </body>
        </html>
        "#;
        let results = parser.parse(html, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Brave Test Result 1");
        assert_eq!(results[0].url, "https://example1.com");
        assert_eq!(results[0].snippet, "This is a test snippet for Brave 1");
        assert_eq!(results[0].rank, 1);
    }

    #[test]
    fn test_brave_parser_empty_and_edge_cases() {
        let parser = BraveParser::new();

        // Test empty HTML
        let results = parser.parse("", 5).unwrap();
        assert!(results.is_empty());

        // Test zero limit
        let html = r#"<html><body><div class="result-row"><a href="https://example.com">Test</a></div></body></html>"#;
        let results = parser.parse(html, 0).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_brave_parser_url_normalization() {
        let parser = BraveParser::new();
        let html = r#"
        <html>
            <body>
                <div class="result-row">
                    <a href="/relative/path">Relative URL</a>
                    <div class="result-snippet">Relative snippet</div>
                </div>
                <div class="result-row">
                    <a href="//protocol-relative.com">Protocol-relative</a>
                    <div class="result-snippet">Protocol snippet</div>
                </div>
            </body>
        </html>
        "#;
        let results = parser.parse(html, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].url, "https://search.brave.com/relative/path");
        assert_eq!(results[1].url, "https://protocol-relative.com");
    }

    #[test]
    fn test_brave_parser_missing_elements() {
        let parser = BraveParser::new();
        let html = r#"
        <html>
            <body>
                <div class="result-row">
                    <a>No href attribute</a>
                </div>
                <div class="result-row">
                    <a href="">Empty href</a>
                </div>
                <div class="result-row">
                    <a href="https://good.com">Good result</a>
                    <div class="result-snippet">Good snippet</div>
                </div>
            </body>
        </html>
        "#;
        let results = parser.parse(html, 10).unwrap();
        assert_eq!(results.len(), 1); // Only the good result
        assert_eq!(results[0].url, "https://good.com");
        assert_eq!(results[0].title, "Good result");
        assert_eq!(results[0].snippet, "Good snippet");
    }

    #[test]
    fn test_brave_parser_limit_enforcement() {
        let parser = BraveParser::new();
        let html = r#"
        <html>
            <body>
                <div class="result-row">
                    <a href="https://example1.com">Result 1</a>
                </div>
                <div class="result-row">
                    <a href="https://example2.com">Result 2</a>
                </div>
                <div class="result-row">
                    <a href="https://example3.com">Result 3</a>
                </div>
            </body>
        </html>
        "#;
        let results = parser.parse(html, 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[1].rank, 2);
    }
}
