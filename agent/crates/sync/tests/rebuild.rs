use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::Duration;

use agent_sync::{SendError, StateClient, SyncConfig};

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

#[test]
fn rebuild_posts_to_the_contract_path_with_the_token() {
    let (url, requests) = serve_once(200, r#"{"status":"ok","rebuilt":[],"warnings":[]}"#);

    client(&url)
        .request_rebuild()
        .expect("200 должен приняться");

    let request = requests.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(
        request
            .request_line
            .starts_with("POST /api/bootstrap/rebuild "),
        "{}",
        request.request_line
    );
    assert_eq!(
        request.authorization.as_deref(),
        Some("Bearer device-token-123")
    );
    assert_eq!(request.body, "");
}

#[test]
fn empty_rebuilt_list_is_success_not_error() {
    let (url, _requests) = serve_once(200, r#"{"status":"ok","rebuilt":[],"warnings":[]}"#);

    let accepted = client(&url).request_rebuild().unwrap();
    assert!(accepted.rebuilt.is_empty());
    assert!(accepted.warnings.is_empty());
}

#[test]
fn rebuilt_regions_are_parsed() {
    let (url, _requests) = serve_once(
        200,
        r#"{"status":"ok","rebuilt":[{"region":"imperial","regionId":"imperial-ngalinn",
            "revision":7,"systems":173,"stations":1186}],"warnings":["imperial: 2 station(s)"]}"#,
    );

    let accepted = client(&url).request_rebuild().unwrap();
    assert_eq!(accepted.rebuilt.len(), 1);
    assert_eq!(accepted.rebuilt[0].region, "imperial");
    assert_eq!(accepted.rebuilt[0].revision, 7);
    assert_eq!(accepted.rebuilt[0].systems, 173);
    assert_eq!(accepted.rebuilt[0].stations, 1186);
    assert_eq!(accepted.warnings.len(), 1);
}

#[test]
fn unconfigured_server_answers_503_and_is_retryable() {
    let (url, _requests) = serve_once(
        503,
        r#"{"status":"error","message":"Bootstrap rebuild is not configured on this server."}"#,
    );

    let error = client(&url).request_rebuild().unwrap_err();
    assert!(error.is_retryable(), "{error:?}");
    assert!(error.to_string().contains("not configured"), "{error}");
}

#[test]
fn read_token_is_rejected_without_retry() {
    let (url, _requests) = serve_once(
        403,
        r#"{"status":"error","message":"This token cannot rebuild the bootstrap."}"#,
    );

    let error = client(&url).request_rebuild().unwrap_err();
    assert!(!error.is_retryable(), "{error:?}");
}

#[test]
fn rate_limited_rebuild_carries_the_delay() {
    let (url, _requests) = serve_once(
        429,
        r#"{"status":"error","message":"Too many requests.","errors":{"retryAfterSeconds":"12"}}"#,
    );

    let error = client(&url).request_rebuild().unwrap_err();
    let SendError::Retryable { retry_after, .. } = &error else {
        panic!("ожидался Retryable, получен {error:?}");
    };
    assert_eq!(*retry_after, Some(Duration::from_secs(12)));
}

#[test]
fn unreachable_server_does_not_panic() {
    let error = client("http://127.0.0.1:1").request_rebuild().unwrap_err();
    assert!(matches!(error, SendError::Transport(_)), "{error:?}");
}
