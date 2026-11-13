//! HTTP 받기. 업데이트 확인, 설치 파일·yt-dlp 받기, 사이트 아이콘에 쓴다.

use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::app_info::user_agent;

#[derive(Clone, Debug, PartialEq)]
pub enum HttpError {
    /// 200 이 아닌 응답.
    Status(u16),
    /// 연결하지 못했거나 받다가 끊겼다.
    Network(String),
    /// 정한 크기보다 큰 응답.
    TooLarge,
}

impl From<ureq::Error> for HttpError {
    fn from(e: ureq::Error) -> Self {
        match e {
            ureq::Error::Status(code, _) => HttpError::Status(code),
            ureq::Error::Transport(t) => HttpError::Network(t.to_string()),
        }
    }
}

/// 프록시를 설정한 사람은 앱의 요청도 프록시로 보낸다. `proxy` 가 비면 직접 붙는다.
pub fn agent(proxy: &str) -> ureq::Agent {
    let mut builder = ureq::AgentBuilder::new().user_agent(&user_agent()).timeout_connect(Duration::from_secs(20));
    if let Ok(tls) = native_tls::TlsConnector::new() {
        builder = builder.tls_connector(Arc::new(tls));
    }
    if !proxy.is_empty() {
        if let Ok(p) = ureq::Proxy::new(proxy) {
            builder = builder.proxy(p);
        }
    }
    builder.build()
}

/// 작은 응답을 통째로 받는다. `max` 바이트를 넘으면 버린다.
pub fn get_bytes(agent: &ureq::Agent, url: &str, accept: &str, max: usize, timeout: Duration) -> Result<Vec<u8>, HttpError> {
    let response = agent.get(url).set("Accept", accept).timeout(timeout).call()?;
    let mut body = Vec::new();
    response
        .into_reader()
        .take(max as u64 + 1)
        .read_to_end(&mut body)
        .map_err(|e| HttpError::Network(e.to_string()))?;
    if body.len() > max {
        return Err(HttpError::TooLarge);
    }
    Ok(body)
}

/// 큰 파일을 `part` 로 받는다. (받은 크기, SHA-256 소문자 hex)를 돌려준다. 실패하면 `part` 를 지운다.
pub fn download(agent: &ureq::Agent, url: &str, part: &Path, progress: &dyn Fn(u64, u64)) -> Result<(u64, String), HttpError> {
    let result = download_inner(agent, url, part, progress);
    if result.is_err() {
        let _ = std::fs::remove_file(part);
    }
    result
}

fn download_inner(agent: &ureq::Agent, url: &str, part: &Path, progress: &dyn Fn(u64, u64)) -> Result<(u64, String), HttpError> {
    let io = |e: std::io::Error| HttpError::Network(e.to_string());
    let response = agent.get(url).set("Accept", "application/octet-stream").call()?;
    let total: u64 = response.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(0);
    if let Some(dir) = part.parent() {
        std::fs::create_dir_all(dir).map_err(io)?;
    }
    let mut file = std::fs::File::create(part).map_err(io)?;
    let mut reader = response.into_reader();
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(io)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(io)?;
        hasher.update(&buf[..n]);
        done += n as u64;
        progress(done, total);
    }
    if total > 0 && done != total {
        return Err(HttpError::Network("다운로드 중단".into()));
    }
    let sha: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    Ok((done, sha))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::path::PathBuf;

    /// 요청 하나에 정해진 응답을 주는 서버. 받은 요청 글을 돌려준다.
    fn serve(response: Vec<u8>) -> (String, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/file", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap();
            stream.write_all(&response).unwrap();
            String::from_utf8_lossy(&buf[..n]).into_owned()
        });
        (url, handle)
    }

    fn ok(body: &[u8]) -> Vec<u8> {
        let mut r = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).into_bytes();
        r.extend_from_slice(body);
        r
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("qm-http-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn get_bytes_sends_agent_and_accept() {
        let (url, server) = serve(ok(b"hello"));
        let body = get_bytes(&agent(""), &url, "application/json", 1024, Duration::from_secs(5)).unwrap();
        assert_eq!(body, b"hello");
        let request = server.join().unwrap().to_lowercase();
        assert!(request.contains(&format!("user-agent: quickmedia/{}", crate::app_info::VERSION)), "{request}");
        assert!(request.contains("accept: application/json"));
    }

    #[test]
    fn get_bytes_refuses_large_bodies() {
        let (url, _server) = serve(ok(&[7u8; 2000]));
        assert_eq!(get_bytes(&agent(""), &url, "*/*", 1000, Duration::from_secs(5)), Err(HttpError::TooLarge));
    }

    #[test]
    fn status_is_reported() {
        let (url, _server) = serve(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec());
        assert_eq!(get_bytes(&agent(""), &url, "*/*", 1000, Duration::from_secs(5)), Err(HttpError::Status(404)));
    }

    #[test]
    fn unreachable_is_a_network_error() {
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{port}/x");
        assert!(matches!(get_bytes(&agent(""), &url, "*/*", 10, Duration::from_secs(2)), Err(HttpError::Network(_))));
    }

    #[test]
    fn download_writes_file_and_hashes() {
        let (url, _server) = serve(ok(b"abc"));
        let part = temp("dl").join("file.part");
        let seen = std::cell::RefCell::new(Vec::new());
        let (size, sha) = download(&agent(""), &url, &part, &|done, total| seen.borrow_mut().push((done, total))).unwrap();
        assert_eq!(size, 3);
        assert_eq!(sha, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(std::fs::read(&part).unwrap(), b"abc");
        assert_eq!(seen.borrow().last(), Some(&(3, 3)));
    }

    #[test]
    fn failed_download_leaves_no_part_file() {
        // 본문이 약속한 길이보다 짧게 끊긴다
        let (url, _server) = serve(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nshort".to_vec());
        let part = temp("cut").join("file.part");
        assert!(download(&agent(""), &url, &part, &|_, _| {}).is_err());
        assert!(!part.exists());
        let (url, _server) = serve(b"HTTP/1.1 500 Oops\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec());
        assert_eq!(download(&agent(""), &url, &part, &|_, _| {}), Err(HttpError::Status(500)));
        assert!(!part.exists());
    }
}
