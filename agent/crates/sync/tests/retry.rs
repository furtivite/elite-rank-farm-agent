use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use agent_core::{CommanderState, SystemPosition, AGENT_VERSION, SCHEMA_VERSION};
use agent_sync::{send_with_retry_waiting, RetryPolicy, StateClient, SyncConfig};
use chrono::{TimeZone, Utc};

fn serve_sequence(responses: Vec<(u16, String)>) -> (String, Arc<Mutex<usize>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("занять порт");
    let port = listener.local_addr().unwrap().port();
    let handled = Arc::new(Mutex::new(0_usize));
    let counter = Arc::clone(&handled);

    std::thread::spawn(move || {
        for (status, body) in responses {
            let Ok((stream, _)) = listener.accept() else {
                return;
            };
            let mut reader = BufReader::new(&stream);
            let mut content_length = 0_usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                let trimmed = line.trim_end();
                if trimmed.is_empty() {
                    break;
                }
                if let Some(value) = trimmed.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = value.trim().parse().unwrap_or(0);
                }
            }
            let mut discard = vec![0_u8; content_length];
            let _ = reader.read_exact(&mut discard);
            *counter.lock().unwrap() += 1;

            let mut stream = stream;
            let response = format!(
                "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });

    (format!("http://127.0.0.1:{port}"), handled)
}

fn state() -> CommanderState {
    CommanderState {
        schema_version: SCHEMA_VERSION,
        agent_version: AGENT_VERSION.to_owned(),
        fined_factions: Vec::new(),
        commander_name: "Jameson".to_owned(),
        current_system: SystemPosition {
            name: "Mainani".to_owned(),
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        current_station: None,
        current_body: None,
        ship: "diamondbackxl".to_owned(),
        required_pad_size: None,
        jump_range_ly: 43.5,
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

fn client(url: &str) -> StateClient {
    StateClient::new(&SyncConfig::new(url, "token").unwrap())
}

const OK_BODY: &str = r#"{"status":"ok","updatedAt":"2026-09-02 09:00:00"}"#;
const RATE_LIMITED: &str =
    r#"{"status":"error","message":"Too many requests.","errors":{"retryAfterSeconds":"2"}}"#;

#[test]
fn retries_until_the_server_accepts() {
    let (url, handled) = serve_sequence(vec![
        (503, r#"{"status":"error","message":"nope"}"#.to_owned()),
        (200, OK_BODY.to_owned()),
    ]);
    let mut slept = Vec::new();
    let mut waited = 0;

    let accepted = send_with_retry_waiting(
        &client(&url),
        &state(),
        &RetryPolicy::default(),
        &mut |_, delay| slept.push(delay),
        &mut |_| waited += 1,
    )
    .expect("вторая попытка должна пройти");

    assert_eq!(accepted.updated_at, "2026-09-02 09:00:00");
    assert_eq!(*handled.lock().unwrap(), 2);
    assert_eq!(slept.len(), 1);
    assert_eq!(waited, 1);
}

#[test]
fn honours_retry_after_from_the_server() {
    let (url, _handled) = serve_sequence(vec![
        (429, RATE_LIMITED.to_owned()),
        (200, OK_BODY.to_owned()),
    ]);
    let mut slept = Vec::new();
    let mut waited = 0;

    send_with_retry_waiting(
        &client(&url),
        &state(),
        &RetryPolicy::default(),
        &mut |_, delay| slept.push(delay),
        &mut |_| waited += 1,
    )
    .unwrap();

    assert_eq!(slept, vec![Duration::from_secs(2)]);
    assert_eq!(waited, 1, "названную сервером паузу обязаны выждать");
}

#[test]
fn gives_up_after_max_attempts() {
    let responses = vec![(503, r#"{"status":"error","message":"nope"}"#.to_owned()); 5];
    let (url, handled) = serve_sequence(responses);
    let policy = RetryPolicy {
        max_attempts: 3,
        base_delay: Duration::from_millis(1),
    };

    let mut waited = 0;
    let error = send_with_retry_waiting(
        &client(&url),
        &state(),
        &policy,
        &mut |_, _| {},
        &mut |_| waited += 1,
    )
    .unwrap_err();

    assert!(error.is_retryable());
    assert_eq!(*handled.lock().unwrap(), 3);
    assert_eq!(waited, 2, "между тремя попытками — две паузы");
}

#[test]
fn does_not_retry_a_rejected_request() {
    let (url, handled) = serve_sequence(vec![
        (
            401,
            r#"{"status":"error","message":"Unknown device token."}"#.to_owned(),
        ),
        (200, OK_BODY.to_owned()),
    ]);

    let error = send_with_retry_waiting(
        &client(&url),
        &state(),
        &RetryPolicy::default(),
        &mut |_, _| panic!("паузы быть не должно"),
        &mut |_| panic!("ждать нечего"),
    )
    .unwrap_err();

    assert!(!error.is_retryable());
    assert_eq!(*handled.lock().unwrap(), 1);
}

#[test]
fn backoff_grows_and_is_capped() {
    let policy = RetryPolicy {
        max_attempts: 10,
        base_delay: Duration::from_secs(5),
    };
    assert_eq!(policy.delay_before(2, None), Duration::from_secs(5));
    assert_eq!(policy.delay_before(3, None), Duration::from_secs(10));
    assert_eq!(policy.delay_before(4, None), Duration::from_secs(20));
    assert_eq!(policy.delay_before(9, None), Duration::from_secs(60));
}
