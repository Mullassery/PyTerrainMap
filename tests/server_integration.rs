//! End-to-end integration tests for the real HTTP/HTTPS API server.
//!
//! These tests start an actual `pyterrain_map::server` instance on a real
//! OS-assigned TCP port and make real requests against it over a real
//! socket (raw HTTP/1.1 for plain, a real rustls TLS client for HTTPS).
//! This is what proves the previously type-only `api`/`api_tls` layer is
//! now genuinely bound to a running server, not just types that compile.

use std::sync::Arc;

use pyterrain_map::server::{
    generate_dev_certificate, run_http_from_listener, run_https_from_listener,
    tls_acceptor_from_pem, ServerState,
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::oneshot;

/// Bind to an OS-assigned ephemeral port and return the bound listener plus
/// the address it's listening on -- avoids hardcoding/racing on a fixed port.
fn bind_ephemeral() -> (std::net::TcpListener, std::net::SocketAddr) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let addr = listener.local_addr().expect("failed to read local_addr");
    (listener, addr)
}

async fn send_raw_http<S>(stream: &mut S, request: &str) -> String
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.unwrap();
    String::from_utf8_lossy(&buf).to_string()
}

#[tokio::test]
async fn test_http_health_and_submit_and_query_end_to_end() {
    let (listener, addr) = bind_ephemeral();
    let state = Arc::new(ServerState::new());
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let server = tokio::spawn(run_http_from_listener(listener, state, shutdown_rx));

    // --- GET /health ---
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let response = send_raw_http(
        &mut stream,
        "GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert!(
        response.starts_with("HTTP/1.1 200"),
        "response was: {response}"
    );
    assert!(response.contains("\"status\":\"ok\""));

    // --- POST /observations ---
    let body = serde_json::json!({
        "robot_id": "bot-1",
        "timestamp": 1_700_000_000_000_000i64,
        "latitude": 40.7128,
        "longitude": -74.0060,
        "elevation": 10.0,
        "sensor_type": "thermal",
        "sensor_value": {"celsius": 22.5},
        "confidence": 0.9,
        "metadata": {}
    })
    .to_string();
    let request = format!(
        "POST /observations HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let response = send_raw_http(&mut stream, &request).await;
    assert!(
        response.starts_with("HTTP/1.1 201"),
        "response was: {response}"
    );
    assert!(response.contains("\"status\":\"stored\""));

    // --- POST /query/spatial should now find the observation we just submitted ---
    let query_body = serde_json::json!({
        "latitude": 40.7128,
        "longitude": -74.0060,
        "radius_m": 1000.0,
        "elevation_min": null,
        "elevation_max": null
    })
    .to_string();
    let request = format!(
        "POST /query/spatial HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        query_body.len(),
        query_body
    );
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let response = send_raw_http(&mut stream, &request).await;
    assert!(
        response.starts_with("HTTP/1.1 200"),
        "response was: {response}"
    );
    assert!(
        response.contains("\"robot_id\":\"bot-1\""),
        "response was: {response}"
    );

    // --- GET /stats reflects the real store state ---
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let response = send_raw_http(
        &mut stream,
        "GET /stats HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert!(
        response.contains("\"total_observations\":1"),
        "response was: {response}"
    );

    // --- Unknown route returns 404 via the real ApiError type ---
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let response = send_raw_http(
        &mut stream,
        "GET /nope HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert!(
        response.starts_with("HTTP/1.1 404"),
        "response was: {response}"
    );

    let _ = shutdown_tx.send(());
    let _ = server.await;
}

#[tokio::test]
async fn test_https_real_tls_handshake_and_request_end_to_end() {
    // Generate a real self-signed dev certificate and trust it explicitly on
    // the client side (this is genuine certificate validation, not a
    // danger-accept-invalid-certs bypass).
    let (cert_pem, key_pem) = generate_dev_certificate(vec!["localhost".to_string()]).unwrap();
    let acceptor = tls_acceptor_from_pem(&cert_pem, &key_pem).unwrap();

    let (listener, addr) = bind_ephemeral();
    let state = Arc::new(ServerState::new());
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let server = tokio::spawn(run_https_from_listener(
        listener,
        state,
        acceptor,
        shutdown_rx,
    ));

    // Build a rustls client that trusts exactly the cert we just generated.
    let mut root_store = rustls::RootCertStore::empty();
    root_store
        .add(&rustls::Certificate(
            rustls_pemfile::certs(&mut &*cert_pem).unwrap().remove(0),
        ))
        .unwrap();
    let client_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
    let server_name = rustls::ServerName::try_from("localhost").unwrap();

    let tcp = TcpStream::connect(addr).await.unwrap();
    let mut tls_stream = connector.connect(server_name, tcp).await.unwrap();

    let response = send_raw_http(
        &mut tls_stream,
        "GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert!(
        response.starts_with("HTTP/1.1 200"),
        "response was: {response}"
    );
    assert!(response.contains("\"status\":\"ok\""));

    let _ = shutdown_tx.send(());
    let _ = server.await;
}
