//! Search access method resolution (API → plain HTTP → browser).

use super::types::{AccessMethod, SearchEngineType};
use crate::constants::{
    ENV_BRAVE_API_KEY, ENV_GEMINI_API_KEY, ENV_SEARX_HOST, ENV_SERPER_API_KEY, ENV_TAVILY_API_KEY,
};
use crate::error::TarziError;
use crate::search::api::searxng::normalize_searx_endpoint;

/// Resolve API key for the active search engine.
/// Engine-specific env vars take precedence over programmatic `search.api_key`.
pub fn resolve_api_key(
    engine: SearchEngineType,
    config_api_key: &Option<String>,
) -> Option<String> {
    let env_name = match engine {
        SearchEngineType::BraveSearch => Some(ENV_BRAVE_API_KEY),
        SearchEngineType::GoogleSerper => Some(ENV_SERPER_API_KEY),
        SearchEngineType::Tavily => Some(ENV_TAVILY_API_KEY),
        SearchEngineType::GoogleAi => Some(ENV_GEMINI_API_KEY),
        _ => None,
    };

    if let Some(name) = env_name
        && let Ok(value) = std::env::var(name)
        && !value.is_empty()
    {
        return Some(value);
    }

    if engine.requires_api_key() {
        return config_api_key.as_ref().filter(|k| !k.is_empty()).cloned();
    }

    // Non-API engines still surface a config api_key when present (generic fallback).
    config_api_key.as_ref().filter(|k| !k.is_empty()).cloned()
}

/// Resolve base URL / host for engines that need one (SearxNG).
/// `SEARX_HOST` wins over programmatic `search.base_url`.
pub fn resolve_base_url(
    engine: SearchEngineType,
    config_base_url: &Option<String>,
) -> Option<String> {
    if !engine.requires_base_url() {
        return None;
    }

    if let Ok(value) = std::env::var(ENV_SEARX_HOST)
        && !value.is_empty()
    {
        return Some(normalize_searx_endpoint(&value));
    }

    config_base_url
        .as_ref()
        .filter(|u| !u.is_empty())
        .map(|u| normalize_searx_endpoint(u))
}

/// Whether credentials required for API access are present.
pub fn has_api_credentials(
    engine: SearchEngineType,
    api_key: &Option<String>,
    base_url: &Option<String>,
) -> bool {
    if engine.requires_base_url() {
        base_url.as_ref().is_some_and(|u| !u.is_empty())
    } else if engine.requires_api_key() || engine.is_api_only() {
        api_key.as_ref().is_some_and(|k| !k.is_empty())
    } else {
        api_key.as_ref().is_some_and(|k| !k.is_empty())
    }
}

/// Build ordered access attempts for an engine.
///
/// Cascade: API (if supported and credentials present) → plain HTTP → browser (if enabled).
/// API-only engines without credentials return an error (caller should skip or fail).
pub fn resolve_access(
    engine: SearchEngineType,
    has_credentials: bool,
    browser_enabled: bool,
) -> Result<Vec<AccessMethod>, TarziError> {
    let mut methods = Vec::new();

    if engine.supports_api() && has_credentials {
        methods.push(AccessMethod::Api);
    }

    if engine.supports_web() {
        methods.push(AccessMethod::PlainHttp);
        if browser_enabled {
            methods.push(AccessMethod::Browser);
        }
    }

    if methods.is_empty() {
        if engine.is_api_only() && !has_credentials {
            return Err(TarziError::Search(engine.missing_credentials_message()));
        }
        return Err(TarziError::Search(format!(
            "No access methods available for engine {engine:?}"
        )));
    }

    Ok(methods)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_engines() -> Vec<SearchEngineType> {
        vec![
            SearchEngineType::Bing,
            SearchEngineType::DuckDuckGo,
            SearchEngineType::Google,
            SearchEngineType::GoogleSerper,
            SearchEngineType::BraveSearch,
            SearchEngineType::Baidu,
            SearchEngineType::SougouWeixin,
            SearchEngineType::Tavily,
            SearchEngineType::GoogleAi,
            SearchEngineType::SearxNG,
        ]
    }

    fn web_only_engines() -> Vec<SearchEngineType> {
        all_engines()
            .into_iter()
            .filter(|e| e.supports_web() && !e.supports_api())
            .collect()
    }

    fn api_only_engines() -> Vec<SearchEngineType> {
        all_engines()
            .into_iter()
            .filter(|e| e.is_api_only())
            .collect()
    }

    #[test]
    fn test_resolve_access_matrix() {
        let web_cascade = vec![AccessMethod::PlainHttp, AccessMethod::Browser];
        let web_no_browser = vec![AccessMethod::PlainHttp];
        let full_cascade = vec![
            AccessMethod::Api,
            AccessMethod::PlainHttp,
            AccessMethod::Browser,
        ];

        for engine in all_engines() {
            for has_key in [true, false] {
                for browser in [true, false] {
                    let result = resolve_access(engine, has_key, browser);
                    let label = format!("{engine:?} has_key={has_key} browser={browser}");

                    match (engine, has_key, browser) {
                        (e, true, true) if e.is_api_only() => {
                            assert_eq!(result.unwrap(), vec![AccessMethod::Api], "{label}");
                        }
                        (e, true, false) if e.is_api_only() => {
                            assert_eq!(result.unwrap(), vec![AccessMethod::Api], "{label}");
                        }
                        (e, false, _) if e.is_api_only() => {
                            assert!(result.is_err(), "{label} should require credentials");
                        }

                        (SearchEngineType::BraveSearch, true, true) => {
                            assert_eq!(result.unwrap(), full_cascade, "{label}");
                        }
                        (SearchEngineType::BraveSearch, true, false) => {
                            assert_eq!(
                                result.unwrap(),
                                vec![AccessMethod::Api, AccessMethod::PlainHttp],
                                "{label}"
                            );
                        }
                        (SearchEngineType::BraveSearch, false, true) => {
                            assert_eq!(result.unwrap(), web_cascade, "{label}");
                        }
                        (SearchEngineType::BraveSearch, false, false) => {
                            assert_eq!(result.unwrap(), web_no_browser, "{label}");
                        }

                        (e, _, true) if e.supports_web() && !e.supports_api() => {
                            assert_eq!(result.unwrap(), web_cascade, "{label}");
                        }
                        (e, _, false) if e.supports_web() && !e.supports_api() => {
                            assert_eq!(result.unwrap(), web_no_browser, "{label}");
                        }
                        _ => panic!("Unhandled matrix cell: {label}"),
                    }
                }
            }
        }
    }

    #[test]
    fn test_web_only_engines_never_get_api() {
        for engine in web_only_engines() {
            let methods = resolve_access(engine, true, true).unwrap();
            assert!(
                !methods.contains(&AccessMethod::Api),
                "{engine:?} must not use API even with a key"
            );
        }
    }

    #[test]
    fn test_api_only_engines_list() {
        let engines = api_only_engines();
        assert!(engines.contains(&SearchEngineType::Tavily));
        assert!(engines.contains(&SearchEngineType::GoogleAi));
        assert!(engines.contains(&SearchEngineType::SearxNG));
        assert!(engines.contains(&SearchEngineType::GoogleSerper));
    }

    #[test]
    fn test_brave_with_and_without_key() {
        let with_key = resolve_access(SearchEngineType::BraveSearch, true, true).unwrap();
        assert_eq!(
            with_key,
            vec![
                AccessMethod::Api,
                AccessMethod::PlainHttp,
                AccessMethod::Browser
            ]
        );
        let without = resolve_access(SearchEngineType::BraveSearch, false, true).unwrap();
        assert_eq!(
            without,
            vec![AccessMethod::PlainHttp, AccessMethod::Browser]
        );
    }

    #[test]
    fn test_bing_browser_toggle() {
        let on = resolve_access(SearchEngineType::Bing, false, true).unwrap();
        assert_eq!(on, vec![AccessMethod::PlainHttp, AccessMethod::Browser]);
        let off = resolve_access(SearchEngineType::Bing, false, false).unwrap();
        assert_eq!(off, vec![AccessMethod::PlainHttp]);
    }

    #[test]
    fn test_google_serper_requires_key() {
        let err = resolve_access(SearchEngineType::GoogleSerper, false, true);
        assert!(err.is_err());

        let methods = resolve_access(SearchEngineType::GoogleSerper, true, true).unwrap();
        assert_eq!(methods, vec![AccessMethod::Api]);
    }

    #[test]
    fn test_tavily_requires_key() {
        let err = resolve_access(SearchEngineType::Tavily, false, true);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("TAVILY_API_KEY"));

        let methods = resolve_access(SearchEngineType::Tavily, true, false).unwrap();
        assert_eq!(methods, vec![AccessMethod::Api]);
    }

    #[test]
    fn test_searxng_requires_host() {
        let err = resolve_access(SearchEngineType::SearxNG, false, true);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("SEARX_HOST"));
    }

    #[test]
    fn test_resolve_api_key_from_config() {
        let key = resolve_api_key(SearchEngineType::BraveSearch, &Some("cfg-key".to_string()));
        if std::env::var(ENV_BRAVE_API_KEY)
            .ok()
            .filter(|v| !v.is_empty())
            .is_none()
        {
            assert_eq!(key.as_deref(), Some("cfg-key"));
        }
    }

    #[test]
    fn test_resolve_api_key_tavily() {
        let key = resolve_api_key(SearchEngineType::Tavily, &Some("tvly-test".to_string()));
        if std::env::var(ENV_TAVILY_API_KEY)
            .ok()
            .filter(|v| !v.is_empty())
            .is_none()
        {
            assert_eq!(key.as_deref(), Some("tvly-test"));
        }
    }

    #[test]
    fn test_resolve_base_url_searxng() {
        if std::env::var(ENV_SEARX_HOST)
            .ok()
            .filter(|v| !v.is_empty())
            .is_none()
        {
            let url = resolve_base_url(
                SearchEngineType::SearxNG,
                &Some("http://localhost:8080".to_string()),
            );
            assert_eq!(url.as_deref(), Some("http://localhost:8080/search"));
        }
        assert!(resolve_base_url(SearchEngineType::Tavily, &Some("x".to_string())).is_none());
    }

    #[test]
    fn test_has_api_credentials() {
        assert!(has_api_credentials(
            SearchEngineType::Tavily,
            &Some("k".to_string()),
            &None
        ));
        assert!(!has_api_credentials(SearchEngineType::Tavily, &None, &None));
        assert!(has_api_credentials(
            SearchEngineType::SearxNG,
            &None,
            &Some("http://localhost:8080/search".to_string())
        ));
        assert!(!has_api_credentials(
            SearchEngineType::SearxNG,
            &Some("k".to_string()),
            &None
        ));
    }

    #[test]
    fn test_resolve_api_key_ignored_for_web_only_engines() {
        for engine in web_only_engines() {
            let key = resolve_api_key(engine, &Some("cfg-key".to_string()));
            assert_eq!(
                key.as_deref(),
                Some("cfg-key"),
                "{engine:?} should still surface config api_key when present"
            );
        }
    }
}
