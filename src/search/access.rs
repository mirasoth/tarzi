//! Search access method resolution (API → plain HTTP → browser).

use super::types::{AccessMethod, SearchEngineType, SearchMode};
use crate::constants::{ENV_BRAVE_API_KEY, ENV_SERPER_API_KEY};
use crate::error::TarziError;

/// Resolve API key for the active search engine.
/// Engine-specific env vars (`BRAVE_API_KEY`, `SERPER_API_KEY`) take precedence over
/// programmatic `search.api_key`.
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
                        "google_serper requires SERPER_API_KEY".to_string(),
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

    fn all_engines() -> Vec<SearchEngineType> {
        vec![
            SearchEngineType::Bing,
            SearchEngineType::DuckDuckGo,
            SearchEngineType::Google,
            SearchEngineType::GoogleSerper,
            SearchEngineType::BraveSearch,
            SearchEngineType::Baidu,
            SearchEngineType::SougouWeixin,
        ]
    }

    fn web_only_engines() -> Vec<SearchEngineType> {
        all_engines()
            .into_iter()
            .filter(|e| e.supports_web() && !e.supports_api())
            .collect()
    }

    #[test]
    fn test_resolve_access_matrix_all_engines_all_modes() {
        let modes = [SearchMode::Auto, SearchMode::ApiQuery, SearchMode::WebQuery];
        let web_cascade = vec![AccessMethod::PlainHttp, AccessMethod::Browser];
        let full_cascade = vec![
            AccessMethod::Api,
            AccessMethod::PlainHttp,
            AccessMethod::Browser,
        ];

        for engine in all_engines() {
            for mode in modes {
                for has_key in [true, false] {
                    let result = resolve_access(engine, mode, has_key);
                    let label = format!("{engine:?} mode={mode:?} has_key={has_key}");

                    match (engine, mode, has_key) {
                        // API-only: google_serper
                        (SearchEngineType::GoogleSerper, SearchMode::WebQuery, _) => {
                            assert!(result.is_err(), "{label} should reject webquery");
                        }
                        (
                            SearchEngineType::GoogleSerper,
                            SearchMode::Auto | SearchMode::ApiQuery,
                            true,
                        ) => {
                            assert_eq!(result.unwrap(), vec![AccessMethod::Api], "{label}");
                        }
                        (
                            SearchEngineType::GoogleSerper,
                            SearchMode::Auto | SearchMode::ApiQuery,
                            false,
                        ) => {
                            assert!(result.is_err(), "{label} should require API key");
                        }

                        // Brave: API + web
                        (SearchEngineType::BraveSearch, SearchMode::Auto, true) => {
                            assert_eq!(result.unwrap(), full_cascade, "{label}");
                        }
                        (SearchEngineType::BraveSearch, SearchMode::Auto, false) => {
                            assert_eq!(result.unwrap(), web_cascade, "{label}");
                        }
                        (SearchEngineType::BraveSearch, SearchMode::WebQuery, _) => {
                            assert_eq!(result.unwrap(), web_cascade, "{label}");
                        }
                        (SearchEngineType::BraveSearch, SearchMode::ApiQuery, true) => {
                            assert_eq!(result.unwrap(), vec![AccessMethod::Api], "{label}");
                        }
                        (SearchEngineType::BraveSearch, SearchMode::ApiQuery, false) => {
                            assert!(result.is_err(), "{label} should require API key");
                        }

                        // Web-only engines
                        (e, SearchMode::ApiQuery, _) if !e.supports_api() => {
                            assert!(result.is_err(), "{label} should reject apiquery");
                        }
                        (e, SearchMode::Auto | SearchMode::WebQuery, _) if e.supports_web() => {
                            assert_eq!(result.unwrap(), web_cascade, "{label}");
                        }
                        _ => panic!("Unhandled matrix cell: {label}"),
                    }
                }
            }
        }
    }

    #[test]
    fn test_web_only_engines_never_get_api_in_auto() {
        for engine in web_only_engines() {
            let methods = resolve_access(engine, SearchMode::Auto, true).unwrap();
            assert!(
                !methods.contains(&AccessMethod::Api),
                "{engine:?} must not use API in auto even with a key"
            );
        }
    }

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

    #[test]
    fn test_resolve_api_key_ignored_for_web_only_engines() {
        for engine in web_only_engines() {
            let key = resolve_api_key(engine, &Some("cfg-key".to_string()));
            // Env-only engines map to None for env name; config key is still returned
            // for non-API engines as a generic fallback — only Brave/Serper read env names.
            // Web-only engines have no dedicated env; config key is still passed through.
            assert_eq!(
                key.as_deref(),
                Some("cfg-key"),
                "{engine:?} should still surface config api_key when present"
            );
        }
    }
}
