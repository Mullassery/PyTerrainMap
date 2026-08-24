//! Real HTTP/HTTPS server binding for the PyTerrainMap API.
//!
//! Prior to this module, `src/api/mod.rs` and `src/api_tls/mod.rs` defined a
//! fully-typed REST API (request/response structs, routes, TLS config types)
//! that was never bound to an actual running server -- no `TcpListener`, no
//! `hyper::Server`, nothing that could accept a real connection. This module
//! wires those types to a genuine `hyper` server backed by the real
//! `ObservationStore` + `SpatialIndex` + `TemporalIndex` core, with optional
//! TLS termination via `rustls`.
//!
//! Two ways to actually run the server:
//! - Rust: [`run_http_from_listener`] / [`run_https_from_listener`] (see
//!   `tests/server_integration.rs` for a full end-to-end example).
//! - Python: `pyterrain_map.start_server(host, port, tls=False)` (see
//!   `src/py_server.rs`), which spawns this exact server on a background
//!   thread and returns a handle you can `.stop()`.
//!
//! ## TLS
//!
//! `dev_tls_acceptor()` generates a fresh self-signed certificate (via
//! `rcgen`) at call time. This is intentionally **development/test only** --
//! self-signed certs are not trusted by real clients/browsers without extra
//! configuration. For production, use `tls_acceptor_from_pem_files` with a
//! real certificate (e.g. from Let's Encrypt) and the `cert_path`/`key_path`
//! already modeled by [`crate::api_tls::TlsConfig`].

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use hyper::server::conn::{AddrIncoming, AddrStream, Http};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, StatusCode};
use parking_lot::RwLock;
use tokio::sync::oneshot;

use crate::api::{
    ApiError, HealthResponse, QueryObservationResponse, SpatialQueryRequest,
    SubmitObservationRequest, SubmitObservationResponse,
};
use crate::query::Query;
use crate::spatial::SpatialIndex;
use crate::storage::backends::StorageBackend;
use crate::storage::persistence_bridge;
use crate::storage::ObservationStore;
use crate::temporal::{DecayFunction, TemporalIndex};

/// Shared, mutable backing state for the HTTP API.
///
/// Wraps the real core: an append-only [`ObservationStore`] plus the spatial
/// and temporal indices needed to answer queries. This is genuinely mutated
/// as observations are submitted -- it is not a mock or fixture.
///
/// `persistent_backend` is optional and `None` by default (`ServerState::new`
/// / `Default`): with no backend configured, this is exactly the previous
/// in-memory-only behavior. When set (via [`ServerState::with_backend`]),
/// every submitted observation is also written through to the backend
/// (e.g. `PostgresBackend`), and [`ServerState::restore_from_backend`] can
/// repopulate the in-memory store from it on startup -- closing the gap
/// where a server restart lost all terrain history even though a working
/// Postgres backend existed in this codebase.
pub struct ServerState {
    pub store: Arc<ObservationStore>,
    spatial: RwLock<SpatialIndex>,
    temporal: RwLock<TemporalIndex>,
    started_at: Instant,
    persistent_backend: Option<Arc<dyn StorageBackend>>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            store: Arc::new(ObservationStore::new()),
            spatial: RwLock::new(SpatialIndex::new()),
            temporal: RwLock::new(TemporalIndex::new(DecayFunction::Exponential {
                half_life_ms: 3_600_000,
            })),
            started_at: Instant::now(),
            persistent_backend: None,
        }
    }

    /// Same as `new()`, but every submitted observation is also written
    /// through to `backend`, and `restore_from_backend()` can repopulate
    /// the in-memory store from whatever `backend` already has.
    pub fn with_backend(backend: Arc<dyn StorageBackend>) -> Self {
        ServerState {
            persistent_backend: Some(backend),
            ..Self::new()
        }
    }

    pub fn has_persistent_backend(&self) -> bool {
        self.persistent_backend.is_some()
    }

    /// Repopulate the in-memory store (and its spatial/temporal indices)
    /// from the configured persistent backend. Call this once at startup,
    /// before serving traffic, to survive a restart. No-op (returns `Ok(0)`)
    /// if no backend is configured.
    ///
    /// Returns the number of observations actually restored. If the backend
    /// had more history than fits in one page, or some rows failed to
    /// convert back into a real `Observation`, that's logged via
    /// `tracing::warn!` rather than silently dropped or silently truncated.
    pub async fn restore_from_backend(&self) -> Result<usize, crate::storage::backends::BackendError> {
        let Some(backend) = &self.persistent_backend else {
            return Ok(0);
        };

        let outcome = persistence_bridge::fetch_all_observations(backend).await?;

        if outcome.truncated {
            tracing::warn!(
                "restore_from_backend: backend has more observations than fit in one \
                 restore page; only the first page was loaded into memory"
            );
        }
        for err in &outcome.conversion_errors {
            tracing::warn!(observation = %err, "restore_from_backend: failed to convert a stored observation, skipping it");
        }

        let mut restored = 0usize;
        for obs in outcome.observations {
            let index = match self.store.add(obs.clone()) {
                Ok(index) => index,
                Err(e) => {
                    tracing::warn!(error = %e, observation_id = %obs.id, "restore_from_backend: failed to re-add observation to in-memory store");
                    continue;
                }
            };
            if let Err(e) = self.spatial.write().insert(index, obs.location, obs.elevation_asl) {
                tracing::warn!(error = %e, observation_id = %obs.id, "restore_from_backend: failed to index observation spatially");
            }
            if let Err(e) = self.temporal.write().insert(obs.timestamp) {
                tracing::warn!(error = %e, observation_id = %obs.id, "restore_from_backend: failed to index observation temporally");
            }
            restored += 1;
        }

        tracing::info!(restored, "restore_from_backend: restored observations from persistent backend");
        Ok(restored)
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

fn json_response<T: serde::Serialize>(status: StatusCode, body: &T) -> Response<Body> {
    let bytes = serde_json::to_vec(body).unwrap_or_else(|_| b"{}".to_vec());
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(bytes))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn error_response(err: ApiError) -> Response<Body> {
    let status = StatusCode::from_u16(err.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    json_response(status, &err)
}

async fn read_body_json<T: serde::de::DeserializeOwned>(req: Request<Body>) -> Result<T, ApiError> {
    let bytes = hyper::body::to_bytes(req.into_body())
        .await
        .map_err(|e| ApiError::invalid_request(&format!("Failed to read request body: {e}")))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| ApiError::invalid_request(&format!("Invalid JSON body: {e}")))
}

/// Route + dispatch a single request. This is the single source of truth for
/// what the server does, shared by both the plain-HTTP and TLS code paths.
async fn handle(state: Arc<ServerState>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let result = match (&method, path.as_str()) {
        (&Method::GET, "/health") => Ok(json_response(
            StatusCode::OK,
            &HealthResponse {
                status: "ok".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_seconds: state.started_at.elapsed().as_secs(),
                observations_stored: state.store.len(),
            },
        )),
        (&Method::GET, "/version") => Ok(json_response(
            StatusCode::OK,
            &serde_json::json!({ "version": env!("CARGO_PKG_VERSION") }),
        )),
        (&Method::POST, "/observations") => handle_submit(state, req).await,
        (&Method::POST, "/query/spatial") => handle_spatial_query(state, req).await,
        (&Method::GET, "/stats") => Ok(json_response(
            StatusCode::OK,
            &serde_json::json!({
                "total_observations": state.store.len(),
                "by_sensor_type": state.store.count_by_sensor_type(),
            }),
        )),
        _ => Err(ApiError::not_found()),
    };

    Ok(result.unwrap_or_else(error_response))
}

async fn handle_submit(
    state: Arc<ServerState>,
    req: Request<Body>,
) -> Result<Response<Body>, ApiError> {
    let submit: SubmitObservationRequest = read_body_json(req).await?;
    let obs = submit.to_observation()?;

    let index = state
        .store
        .add(obs.clone())
        .map_err(|e| ApiError::internal_error(&e.to_string()))?;

    state
        .spatial
        .write()
        .insert(index, obs.location, obs.elevation_asl)
        .map_err(|e| ApiError::internal_error(&e.to_string()))?;
    state
        .temporal
        .write()
        .insert(obs.timestamp)
        .map_err(|e| ApiError::internal_error(&e.to_string()))?;

    // Write-through to the persistent backend, if one is configured. This is
    // awaited (not fire-and-forget): if a caller configured persistence,
    // failing to durably persist should surface as a real error rather than
    // silently returning 201 for data that only exists in memory and won't
    // survive a restart.
    if let Some(backend) = &state.persistent_backend {
        let storage_obs = persistence_bridge::observation_to_storage(&obs);
        backend
            .insert_observation(&storage_obs)
            .await
            .map_err(|e| ApiError::internal_error(&format!("failed to persist observation: {e}")))?;
    }

    Ok(json_response(
        StatusCode::CREATED,
        &SubmitObservationResponse {
            id: obs.id.to_string(),
            status: "stored".to_string(),
            timestamp: obs.timestamp,
        },
    ))
}

async fn handle_spatial_query(
    state: Arc<ServerState>,
    req: Request<Body>,
) -> Result<Response<Body>, ApiError> {
    let q: SpatialQueryRequest = read_body_json(req).await?;

    // Query::new takes ownership of the indices, so we snapshot (clone) the
    // current state under a read lock. The clone is cheap relative to a
    // network round trip and keeps the write path (handle_submit) from
    // blocking concurrent reads for longer than a HashMap/Vec clone takes.
    let spatial_snapshot = state.spatial.read().clone();
    let temporal_snapshot = state.temporal.read().clone();
    let now_us = chrono::Utc::now().timestamp_micros();

    let query = Query::new(
        spatial_snapshot,
        temporal_snapshot,
        state.store.clone(),
        now_us,
    );
    let results = query
        .radius(q.latitude, q.longitude, q.radius_m)
        .map_err(|e| ApiError::internal_error(&e.to_string()))?;

    let response: Vec<QueryObservationResponse> = results
        .into_iter()
        .map(|r| {
            let obs = &r.observation;
            QueryObservationResponse {
                id: obs.id.to_string(),
                robot_id: obs.robot_id.clone(),
                timestamp: obs.timestamp,
                latitude: obs.location.lat,
                longitude: obs.location.lon,
                elevation: obs.elevation_asl,
                sensor_type: obs.sensor_type.to_string(),
                confidence: obs.confidence,
                decayed_confidence: Some(r.decayed_confidence),
            }
        })
        .collect();

    Ok(json_response(StatusCode::OK, &response))
}

/// Run a plain-HTTP server on an already-bound listener until `shutdown`
/// fires (or fires with the sender dropped).
///
/// Binding synchronously (via `std::net::TcpListener::bind`) before calling
/// this function means the caller can know for certain the port is listening
/// as soon as bind succeeds -- no readiness race, no guessing sleep.
pub async fn run_http_from_listener(
    listener: std::net::TcpListener,
    state: Arc<ServerState>,
    shutdown: oneshot::Receiver<()>,
) -> hyper::Result<()> {
    listener
        .set_nonblocking(true)
        .expect("failed to set listener non-blocking");
    let incoming = AddrIncoming::from_listener(
        tokio::net::TcpListener::from_std(listener).expect("failed to adopt std listener"),
    )?;

    let make_svc = make_service_fn(move |_conn: &AddrStream| {
        let state = state.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(state.clone(), req))) }
    });

    hyper::Server::builder(incoming)
        .serve(make_svc)
        .with_graceful_shutdown(async {
            let _ = shutdown.await;
        })
        .await
}

/// Convenience wrapper: bind `addr` and serve plain HTTP until `shutdown`.
pub async fn run_http(
    addr: SocketAddr,
    state: Arc<ServerState>,
    shutdown: oneshot::Receiver<()>,
) -> std::io::Result<()> {
    let listener = std::net::TcpListener::bind(addr)?;
    run_http_from_listener(listener, state, shutdown)
        .await
        .map_err(std::io::Error::other)
}

/// Run an HTTPS server on an already-bound listener, terminating TLS with
/// `acceptor`, until `shutdown` fires.
pub async fn run_https_from_listener(
    listener: std::net::TcpListener,
    state: Arc<ServerState>,
    acceptor: tokio_rustls::TlsAcceptor,
    mut shutdown: oneshot::Receiver<()>,
) -> std::io::Result<()> {
    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;

    loop {
        tokio::select! {
            _ = &mut shutdown => return Ok(()),
            accepted = listener.accept() => {
                let (stream, _peer) = accepted?;
                let acceptor = acceptor.clone();
                let state = state.clone();
                tokio::spawn(async move {
                    match acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            let svc = service_fn(move |req| handle(state.clone(), req));
                            if let Err(e) = Http::new().serve_connection(tls_stream, svc).await {
                                tracing::warn!("HTTPS connection error: {e}");
                            }
                        }
                        Err(e) => tracing::warn!("TLS handshake failed: {e}"),
                    }
                });
            }
        }
    }
}

/// Convenience wrapper: bind `addr` and serve HTTPS until `shutdown`.
pub async fn run_https(
    addr: SocketAddr,
    state: Arc<ServerState>,
    acceptor: tokio_rustls::TlsAcceptor,
    shutdown: oneshot::Receiver<()>,
) -> std::io::Result<()> {
    let listener = std::net::TcpListener::bind(addr)?;
    run_https_from_listener(listener, state, acceptor, shutdown).await
}

/// Generate a fresh self-signed certificate for `subject_alt_names` (e.g.
/// `["localhost"]`). Returns `(cert_pem, key_pem)`.
///
/// **Development/test only.** Self-signed certs are not trusted by real
/// clients without explicitly adding them to a trust store, which is exactly
/// what `tests/server_integration.rs` does to prove the TLS handshake and
/// request/response cycle genuinely work end-to-end.
pub fn generate_dev_certificate(
    subject_alt_names: Vec<String>,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let cert = rcgen::generate_simple_self_signed(subject_alt_names)
        .map_err(|e| format!("Failed to generate self-signed certificate: {e}"))?;
    let cert_pem = cert
        .serialize_pem()
        .map_err(|e| format!("Failed to serialize certificate: {e}"))?;
    let key_pem = cert.serialize_private_key_pem();
    Ok((cert_pem.into_bytes(), key_pem.into_bytes()))
}

/// Build a `TlsAcceptor` from PEM-encoded certificate + private key bytes.
pub fn tls_acceptor_from_pem(
    cert_pem: &[u8],
    key_pem: &[u8],
) -> Result<tokio_rustls::TlsAcceptor, String> {
    let certs = rustls_pemfile::certs(&mut &*cert_pem)
        .map_err(|e| format!("Failed to parse certificate PEM: {e}"))?
        .into_iter()
        .map(rustls::Certificate)
        .collect::<Vec<_>>();
    if certs.is_empty() {
        return Err("No certificates found in PEM input".to_string());
    }

    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut &*key_pem)
        .map_err(|e| format!("Failed to parse PKCS8 private key PEM: {e}"))?;
    if keys.is_empty() {
        // rcgen emits PKCS8 keys; fall back to RSA (PKCS1) just in case a
        // user supplies a non-rcgen key in that format.
        keys = rustls_pemfile::rsa_private_keys(&mut &*key_pem)
            .map_err(|e| format!("Failed to parse RSA private key PEM: {e}"))?;
    }
    if keys.is_empty() {
        return Err("No private key found in PEM input".to_string());
    }
    let key = rustls::PrivateKey(keys.remove(0));

    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| format!("Invalid certificate/key pair: {e}"))?;

    Ok(tokio_rustls::TlsAcceptor::from(Arc::new(config)))
}

/// Load a `TlsAcceptor` from cert/key file paths (production use), or
/// generate a fresh self-signed dev certificate if either path is `None`.
pub fn dev_or_file_tls_acceptor(
    cert_path: Option<&str>,
    key_path: Option<&str>,
) -> Result<tokio_rustls::TlsAcceptor, String> {
    match (cert_path, key_path) {
        (Some(cert_path), Some(key_path)) => {
            let cert_pem = std::fs::read(cert_path)
                .map_err(|e| format!("Failed to read cert file {cert_path}: {e}"))?;
            let key_pem = std::fs::read(key_path)
                .map_err(|e| format!("Failed to read key file {key_path}: {e}"))?;
            tls_acceptor_from_pem(&cert_pem, &key_pem)
        }
        _ => {
            let (cert_pem, key_pem) = generate_dev_certificate(vec!["localhost".to_string()])?;
            tls_acceptor_from_pem(&cert_pem, &key_pem)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_state_starts_empty() {
        let state = ServerState::new();
        assert_eq!(state.store.len(), 0);
    }

    #[test]
    fn test_server_state_new_has_no_persistent_backend() {
        let state = ServerState::new();
        assert!(!state.has_persistent_backend());
    }

    fn sample_submit(robot_id: &str, lat: f64, lon: f64, timestamp: i64) -> SubmitObservationRequest {
        serde_json::from_value(serde_json::json!({
            "robot_id": robot_id,
            "timestamp": timestamp,
            "latitude": lat,
            "longitude": lon,
            "sensor_type": "thermal",
            "sensor_value": { "celsius": 21.0 },
            "confidence": 0.9
        }))
        .unwrap()
    }

    #[tokio::test]
    async fn submitted_observations_are_written_through_to_the_configured_backend() {
        use crate::storage::backends::InMemoryBackend;

        let backend: Arc<dyn StorageBackend> = Arc::new(InMemoryBackend::new());
        let state = Arc::new(ServerState::with_backend(backend.clone()));
        assert!(state.has_persistent_backend());

        let req = sample_submit("robot-1", 37.7749, -122.4194, 1_700_000_000_000_000);
        let body = Body::from(serde_json::to_vec(&req).unwrap());
        let http_req = Request::builder()
            .method(Method::POST)
            .uri("/observations")
            .body(body)
            .unwrap();

        let response = handle_submit(state.clone(), http_req).await;
        assert!(response.is_ok(), "submit should succeed when a backend is configured");

        // Verify it actually landed in the backend, not just in memory.
        let count = persistence_bridge::count_observations(&backend).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(state.store.len(), 1, "should also still be in the fast in-memory path");
    }

    #[tokio::test]
    async fn restore_from_backend_repopulates_the_in_memory_store() {
        use crate::storage::backends::InMemoryBackend;

        let backend: Arc<dyn StorageBackend> = Arc::new(InMemoryBackend::new());

        // Simulate a previous server instance's history already sitting in
        // the backend before this one starts.
        let old_obs = crate::types::Observation::with_clock_source(
            "robot-2".to_string(),
            1_700_000_000_000_000,
            crate::types::GeoPoint::new(10.0, 20.0),
            None,
            crate::types::SensorType::Thermal,
            crate::types::SensorValue::Temperature { celsius: 15.0 },
            0.7,
            crate::types::ClockSource::UTC,
        );
        backend
            .insert_observation(&persistence_bridge::observation_to_storage(&old_obs))
            .await
            .unwrap();

        let state = ServerState::with_backend(backend);
        assert_eq!(state.store.len(), 0, "restore hasn't run yet");

        let restored = state.restore_from_backend().await.unwrap();

        assert_eq!(restored, 1);
        assert_eq!(state.store.len(), 1, "in-memory store should now have the restored observation");
    }

    #[tokio::test]
    async fn restore_from_backend_is_a_noop_without_a_configured_backend() {
        let state = ServerState::new();
        let restored = state.restore_from_backend().await.unwrap();
        assert_eq!(restored, 0);
    }

    #[test]
    fn test_generate_dev_certificate() {
        let (cert_pem, key_pem) = generate_dev_certificate(vec!["localhost".to_string()]).unwrap();
        assert!(String::from_utf8_lossy(&cert_pem).contains("BEGIN CERTIFICATE"));
        assert!(!key_pem.is_empty());
    }

    #[test]
    fn test_tls_acceptor_from_generated_cert() {
        let (cert_pem, key_pem) = generate_dev_certificate(vec!["localhost".to_string()]).unwrap();
        let acceptor = tls_acceptor_from_pem(&cert_pem, &key_pem);
        assert!(acceptor.is_ok());
    }

    #[test]
    fn test_tls_acceptor_rejects_garbage() {
        let result = tls_acceptor_from_pem(b"not a cert", b"not a key");
        assert!(result.is_err());
    }
}
