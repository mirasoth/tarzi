//! Search access method resolution (API → plain HTTP → browser).

use super::types::{AccessMethod, SearchEngineType, SearchMode};
use crate::constants::{ENV_BRAVE_API_KEY, ENV_SERPER_API_KEY};
use crate::error::TarziError;

/// Resolve API key for the active search engine.
/// Environment variables take precedence over `search.api_key` in config.
pub fn resolve_api_key(
    engine: SearchEngineType,
    config_api_key: &Option<String>,
) -> Option<String> {
    let env_name = match engine {
        SearchEngineType::BraveSearch => Some(ENV_BRAVE_API_KEY),
        SearchEngineType::GoogleSerper => Some(ENV_SERPER_API_KEY),
        _ => None,
    };

    if let Some(name) = env_name
        && let Ok(value) = std::env::var(name)
        && !value.is_empty()
    {
        return Some(value);
    }

    config_api_key.as_ref().filter(|k| !k.is_empty()).cloned()
}

/// Build ordered access attempts for the given engine, mode, and key availability.
///
/// For API-only engines (`google_serper`) without a key, returns an error.
pub fn resolve_access(
    engine: SearchEngineType,
    mode: SearchMode,
    has_api_key: bool,
) -> Result<Vec<AccessMethod>, TarziError> {
    if engine.is_api_only() {
        return match mode {
            SearchMode::WebQuery => Err(TarziError::Search(format!(
                "Engine {engine:?} only supports apiquery; webquery is not available"
            ))),
            SearchMode::Auto | SearchMode::ApiQuery => {
                if has_api_key {
                    Ok(vec![AccessMethod::Api])
                } else {
                    Err(TarziError::Search(
                        "google_serper requires SERPER_API_KEY (or search.api_key)".to_string(),
                    ))
                }
            }
        };
    }

    match mode {
        SearchMode::ApiQuery => {
            if !engine.supports_api() {
                return Err(TarziError::Search(format!(
                    "Engine {engine:?} does not support apiquery"
                )));
            }
            if !has_api_key {
                return Err(TarziError::Search(format!(
                    "apiquery requested for {engine:?} but no API key is configured"
                )));
            }
            Ok(vec![AccessMethod::Api])
        }
        SearchMode::WebQuery => Ok(vec![AccessMethod::PlainHttp, AccessMethod::Browser]),
        SearchMode::Auto => {
            let mut methods = Vec::new();
            if engine.supports_api() && has_api_key {
                methods.push(AccessMethod::Api);
            }
            if engine.supports_web() {
                methods.push(AccessMethod::PlainHttp);
                methods.push(AccessMethod::Browser);
            }
            if methods.is_empty() {
                return Err(TarziError::Search(format!(
                    "No access methods available for engine {engine:?}"
                )));
            }
            Ok(methods)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_brave_with_key() {
        let methods =
            resolve_access(SearchEngineType::BraveSearch, SearchMode::Auto, true).unwrap();
        assert_eq!(
            methods,
            vec![
                AccessMethod::Api,
                AccessMethod::PlainHttp,
                AccessMethod::Browser
            ]
        );
    }

    #[test]
    fn test_auto_brave_without_key() {
        let methods =
            resolve_access(SearchEngineType::BraveSearch, SearchMode::Auto, false).unwrap();
        assert_eq!(
            methods,
            vec![AccessMethod::PlainHttp, AccessMethod::Browser]
        );
    }

    #[test]
    fn test_auto_bing() {
        let methods = resolve_access(SearchEngineType::Bing, SearchMode::Auto, false).unwrap();
        assert_eq!(
            methods,
            vec![AccessMethod::PlainHttp, AccessMethod::Browser]
        );
    }

    #[test]
    fn test_webquery_ignores_key() {
        let methods =
            resolve_access(SearchEngineType::BraveSearch, SearchMode::WebQuery, true).unwrap();
        assert_eq!(
            methods,
            vec![AccessMethod::PlainHttp, AccessMethod::Browser]
        );
    }

    #[test]
    fn test_apiquery_brave_requires_key() {
        let err = resolve_access(SearchEngineType::BraveSearch, SearchMode::ApiQuery, false);
        assert!(err.is_err());
    }

    #[test]
    fn test_apiquery_bing_unsupported() {
        let err = resolve_access(SearchEngineType::Bing, SearchMode::ApiQuery, true);
        assert!(err.is_err());
    }

    #[test]
    fn test_google_serper_requires_key() {
        let err = resolve_access(SearchEngineType::GoogleSerper, SearchMode::Auto, false);
        assert!(err.is_err());

        let methods =
            resolve_access(SearchEngineType::GoogleSerper, SearchMode::Auto, true).unwrap();
        assert_eq!(methods, vec![AccessMethod::Api]);
    }

    #[test]
    fn test_google_serper_webquery_rejected() {
        let err = resolve_access(SearchEngineType::GoogleSerper, SearchMode::WebQuery, true);
        assert!(err.is_err());
    }

    #[test]
    fn test_resolve_api_key_from_config() {
        let key = resolve_api_key(SearchEngineType::BraveSearch, &Some("cfg-key".to_string()));
        // May be overridden by env in CI; at least config path works when env unset
        if std::env::var(ENV_BRAVE_API_KEY)
            .ok()
            .filter(|v| !v.is_empty())
            .is_none()
        {
            assert_eq!(key.as_deref(), Some("cfg-key"));
        }
    }
}
