use agent_sync::config::{ENV_BASE_URL, ENV_DEVICE_TOKEN};
use agent_sync::{SyncConfig, DEFAULT_BASE_URL, STATE_PATH};

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn https_base_url_is_accepted_and_normalised() {
    let config = SyncConfig::new("https://example.com/", "token").unwrap();
    assert_eq!(config.base_url(), "https://example.com");
    assert_eq!(
        config.state_url(),
        format!("https://example.com{STATE_PATH}")
    );
}

#[test]
fn plain_http_is_rejected_except_localhost() {
    assert!(SyncConfig::new("http://example.com", "token").is_err());
    assert!(SyncConfig::new("http://127.0.0.1:8080", "token").is_ok());
    assert!(SyncConfig::new("http://localhost:8080", "token").is_ok());
    assert!(SyncConfig::new("ftp://example.com", "token").is_err());
    assert!(SyncConfig::new("example.com", "token").is_err());
}

#[test]
fn empty_token_is_rejected() {
    assert!(SyncConfig::new("https://example.com", "   ").is_err());
}

#[test]
fn debug_output_does_not_leak_the_token() {
    let config = SyncConfig::new("https://example.com", "s3cret-device-token").unwrap();
    let rendered = format!("{config:?}");
    assert!(!rendered.contains("s3cret-device-token"), "{rendered}");
    assert!(rendered.contains("https://example.com"));
}

#[test]
fn config_file_is_read_and_unknown_fields_rejected() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let dir = std::env::temp_dir().join("agent-sync-config");
    std::fs::create_dir_all(&dir).unwrap();

    let good = dir.join("good.json");
    std::fs::write(
        &good,
        r#"{"baseUrl":"https://elite.example.com","deviceToken":"abc"}"#,
    )
    .unwrap();
    let config = SyncConfig::from_file(&good).unwrap();
    assert_eq!(config.device_token(), "abc");
    assert_eq!(config.base_url(), DEFAULT_BASE_URL);

    let bad = dir.join("bad.json");
    std::fs::write(
        &bad,
        r#"{"baseUrl":"https://elite.example.com","deviceToken":"abc","token":"abc"}"#,
    )
    .unwrap();
    assert!(SyncConfig::from_file(&bad).is_err());
}

#[test]
fn environment_overrides_the_file() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    std::env::set_var(ENV_BASE_URL, "https://from-env.example.com");
    std::env::set_var(ENV_DEVICE_TOKEN, "env-token");

    let config = SyncConfig::load().unwrap();
    assert_eq!(config.base_url(), "https://from-env.example.com");
    assert_eq!(config.device_token(), "env-token");

    std::env::remove_var(ENV_BASE_URL);
    std::env::remove_var(ENV_DEVICE_TOKEN);
}
