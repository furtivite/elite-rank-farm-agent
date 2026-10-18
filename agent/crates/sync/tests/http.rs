use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::Duration;

use agent_core::{CommanderState, SystemPosition, AGENT_VERSION, SCHEMA_VERSION};
use agent_sync::{SendError, StateClient, SyncConfig};
use chrono::{TimeZone, Utc};

struct RecordedRequest {
    request_line: String,
    authorization: Option<String>,
    body: String,
}

fn serve_once(status: u16, body: &'static str) -> (String, mpsc::Receiver<RecordedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("занять порт");
    let port = listener.local_addr().unwrap().port();
    let (sender, receiver) = mpsc::channel();

    std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("принять соединение");
        let recorded = read_request(&stream);
        let _ = sender.send(recorded);
        respond(stream, status, body);
    });

    (format!("http://127.0.0.1:{port}"), receiver)
}

fn read_request(stream: &TcpStream) -> RecordedRequest {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).expect("request line");

    let mut authorization = None;
    let mut content_length = 0_usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("заголовок");
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        let lowered = trimmed.to_ascii_lowercase();
        if let Some(value) = lowered.strip_prefix("authorization:") {
            authorization = Some(trimmed[trimmed.len() - value.trim().len()..].to_owned());
        }
        if let Some(value) = lowered.strip_prefix("content-length:") {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body).expect("тело запроса");

    RecordedRequest {
        request_line: request_line.trim_end().to_owned(),
        authorization,
        body: String::from_utf8_lossy(&body).into_owned(),
    }
}

fn respond(mut stream: TcpStream, status: u16, body: &str) {
    let response = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn client(base_url: &str) -> StateClient {
    StateClient::new(&SyncConfig::new(base_url, "device-token-123").unwrap())
}

fn state() -> CommanderState {
    CommanderState {
        schema_version: SCHEMA_VERSION,
        agent_version: AGENT_VERSION.to_owned(),
        fined_factions: Vec::new(),
        commander_name: "Jameson".to_owned(),
        current_system: SystemPosition {
            name: "Mainani".to_owned(),
            x: 10.40625,
            y: -219.8125,
            z: -64.21875,
        },
        current_station: Some("Mies van der Rohe's Claim".to_owned()),
        current_body: None,
        ship: "diamondbackxl".to_owned(),
        required_pad_size: None,
        jump_range_ly: 43.487682,
        imperial_rank: 7,
        imperial_progress: 71.0,
        imperial_progress_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
        federal_rank: 2,
        federal_progress: 100.0,
        federal_progress_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
        observed_at: Utc.with_ymd_and_hms(2026, 9, 1, 22, 16, 9).unwrap(),
        presence: None,
        loadout: None,
        recent_systems: None,
        recent_supercruise: None,
    }
}

#[test]
fn successful_put_sends_the_expected_request() {
    let (url, requests) = serve_once(200, r#"{"status":"ok","updatedAt":"2026-09-02 09:00:00"}"#);

    let accepted = client(&url)
        .put_state(&state())
        .expect("200 должен приняться");
    assert_eq!(accepted.updated_at, "2026-09-02 09:00:00");

    let request = requests.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(
        request.request_line.starts_with("PUT /api/elite/state "),
        "{}",
        request.request_line
    );
    assert_eq!(
        request.authorization.as_deref(),
        Some("Bearer device-token-123")
    );

    let sent: serde_json::Value = serde_json::from_str(&request.body).expect("тело — JSON");
    assert_eq!(sent["schemaVersion"], 4);
    assert_eq!(sent["imperialProgressAt"], "2026-09-01T18:00:00Z");
    assert_eq!(sent["currentSystem"]["name"], "Mainani");
    assert_eq!(sent["currentBody"], serde_json::Value::Null);
}

#[test]
fn invalid_state_is_not_sent_at_all() {
    let mut broken = state();
    broken.imperial_rank = 42;

    let error = client("http://127.0.0.1:1").put_state(&broken).unwrap_err();
    assert!(matches!(error, SendError::Contract(_)), "{error:?}");
    assert!(!error.is_retryable());
}

#[test]
fn rate_limit_is_retryable_and_carries_the_delay() {
    let (url, _requests) = serve_once(
        429,
        r#"{"status":"error","message":"Too many requests.","errors":{"retryAfterSeconds":"3"}}"#,
    );

    let error = client(&url).put_state(&state()).unwrap_err();
    let SendError::Retryable { retry_after, .. } = &error else {
        panic!("ожидался Retryable, получен {error:?}");
    };
    assert_eq!(*retry_after, Some(Duration::from_secs(3)));
    assert!(error.is_retryable());
}

#[test]
fn server_error_is_retryable() {
    let (url, _requests) = serve_once(503, r#"{"status":"error","message":"nope"}"#);

    let error = client(&url).put_state(&state()).unwrap_err();
    assert!(error.is_retryable(), "{error:?}");
}

#[test]
fn bad_token_is_not_retried() {
    let (url, _requests) = serve_once(
        401,
        r#"{"status":"error","message":"Device token has been revoked."}"#,
    );

    let error = client(&url).put_state(&state()).unwrap_err();
    assert!(!error.is_retryable(), "{error:?}");
    assert!(error.to_string().contains("revoked"), "{error}");
    let hint = error.hint().expect("401 без подсказки");
    assert!(hint.contains("Reissue"), "{hint}");
    assert!(error.is_token_rejected(), "{error:?}");
}

#[test]
fn forbidden_is_also_a_token_problem() {
    let (url, _requests) = serve_once(
        403,
        r#"{"status":"error","message":"This token cannot write commander state."}"#,
    );

    let error = client(&url).put_state(&state()).unwrap_err();
    assert!(error.is_token_rejected(), "{error:?}");
}

#[test]
fn contract_error_is_not_a_token_problem() {
    let (url, _requests) = serve_once(400, r#"{"status":"error","message":"Validation failed."}"#);

    let error = client(&url).put_state(&state()).unwrap_err();
    assert!(!error.is_token_rejected(), "{error:?}");
    assert!(!error.is_retryable(), "{error:?}");
}

#[test]
fn validation_error_from_server_is_not_retried() {
    let (url, _requests) = serve_once(
        400,
        r#"{"status":"error","message":"Validation failed.","errors":["imperialRank"]}"#,
    );

    let error = client(&url).put_state(&state()).unwrap_err();
    assert!(!error.is_retryable(), "{error:?}");
    assert!(error.to_string().contains("imperialRank"), "{error}");
}

#[test]
fn unreachable_server_is_transport_error_and_retryable() {
    let error = client("http://127.0.0.1:1")
        .put_state(&state())
        .unwrap_err();
    assert!(matches!(error, SendError::Transport(_)), "{error:?}");
    assert!(error.is_retryable());
}

#[test]
fn response_carries_the_released_agent_version() {
    let (url, _requests) = serve_once(
        200,
        r#"{"status":"ok","updatedAt":"2026-09-06 09:00:00","agentUpdate":{"currentVersion":"0.3.0"}}"#,
    );

    let accepted = client(&url).put_state(&state()).expect("ответ разобран");

    assert_eq!(
        accepted
            .agent_update
            .and_then(|update| update.current_version),
        Some("0.3.0".to_owned()),
    );
}

#[test]
fn response_without_the_field_is_still_valid() {
    let (url, _requests) = serve_once(200, r#"{"status":"ok","updatedAt":"2026-09-06 09:00:00"}"#);

    let accepted = client(&url).put_state(&state()).expect("ответ разобран");

    assert!(accepted.agent_update.is_none());
}

#[test]
fn forbidden_token_also_gets_a_hint() {
    let (url, _requests) = serve_once(403, r#"{"status":"error","message":"Forbidden."}"#);

    let error = client(&url).put_state(&state()).unwrap_err();
    assert!(!error.is_retryable(), "{error:?}");
    assert!(error.hint().is_some(), "{error}");
}

#[test]
fn retryable_error_has_no_hint() {
    let (url, _requests) = serve_once(503, r#"{"status":"error","message":"nope"}"#);

    let error = client(&url).put_state(&state()).unwrap_err();
    assert!(error.hint().is_none(), "{error}");
}
