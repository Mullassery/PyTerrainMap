//! Python-callable entry point for the real HTTP/HTTPS API server.
//!
//! `pyterrain_map.start_server()` binds a real TCP listener (synchronously,
//! before returning to Python -- no readiness races), spawns the async
//! server from `crate::server` on a background OS thread with its own tokio
//! runtime, and returns a `ServerHandle` that can `.stop()` it.

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::JoinHandle;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use tokio::sync::oneshot;

use crate::server::{
    dev_or_file_tls_acceptor, run_http_from_listener, run_https_from_listener, ServerState,
};

/// Handle to a running PyTerrainMap API server started via `start_server()`.
///
/// The server keeps running on a background thread until you call `.stop()`
/// (or the process exits). Dropping the handle without calling `.stop()`
/// leaves the server running -- call `.stop()` explicitly when you're done,
/// e.g. in a `finally` block or a context manager in higher-level Python code.
#[pyclass]
pub struct PyServerHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    join_handle: Option<JoinHandle<()>>,
    #[pyo3(get)]
    host: String,
    #[pyo3(get)]
    port: u16,
    #[pyo3(get)]
    tls: bool,
}

#[pymethods]
impl PyServerHandle {
    /// Stop the server and block until its background thread has exited.
    pub fn stop(&mut self) -> PyResult<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.join_handle.take() {
            handle
                .join()
                .map_err(|_| PyRuntimeError::new_err("Server thread panicked"))?;
        }
        Ok(())
    }

    /// Whether the background server thread is still running.
    pub fn is_running(&self) -> bool {
        match &self.join_handle {
            Some(h) => !h.is_finished(),
            None => false,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ServerHandle(host={:?}, port={}, tls={}, running={})",
            self.host,
            self.port,
            self.tls,
            self.is_running()
        )
    }

    fn __enter__(slf: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        Ok(slf)
    }

    fn __exit__(
        &mut self,
        _exc_type: PyObject,
        _exc_value: PyObject,
        _traceback: PyObject,
    ) -> PyResult<()> {
        self.stop()
    }
}

/// Start a real PyTerrainMap HTTP(S) API server in a background thread.
///
/// The listening socket is bound synchronously before this function returns,
/// so by the time you get a `ServerHandle` back the server is genuinely
/// accepting connections -- no sleep-and-hope required.
///
/// Endpoints: `GET /health`, `GET /version`, `GET /stats`,
/// `POST /observations` (submit), `POST /query/spatial` (radius query).
/// Backed by a real, in-process `ObservationStore` + spatial/temporal index
/// (see `src/server.rs`) -- not mocked responses.
///
/// TLS: pass `tls=True` to serve HTTPS. Unless `cert_path`/`key_path` are
/// both given, a fresh **self-signed** certificate is generated for
/// `localhost` -- fine for local development and tests, but real clients
/// won't trust it without extra configuration. For production, pass real
/// certificate/key file paths.
#[pyfunction]
#[pyo3(signature = (host="127.0.0.1".to_string(), port=8080, tls=false, cert_path=None, key_path=None))]
pub fn start_server(
    host: String,
    port: u16,
    tls: bool,
    cert_path: Option<String>,
    key_path: Option<String>,
) -> PyResult<PyServerHandle> {
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| PyValueError::new_err(format!("Invalid host/port {host}:{port}: {e}")))?;

    // Bind synchronously on the calling thread. If this succeeds, the OS has
    // reserved the port for us -- the caller can rely on the server being
    // reachable as soon as start_server() returns (once accept() is being
    // polled on the background thread, which happens immediately).
    let listener = std::net::TcpListener::bind(addr)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to bind {addr}: {e}")))?;

    let tls_acceptor = if tls {
        Some(
            dev_or_file_tls_acceptor(cert_path.as_deref(), key_path.as_deref())
                .map_err(PyRuntimeError::new_err)?,
        )
    } else {
        None
    };

    let state = Arc::new(ServerState::new());
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let join_handle = std::thread::Builder::new()
        .name("pyterrainmap-server".to_string())
        .spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("pyterrainmap: failed to start server tokio runtime: {e}");
                    return;
                }
            };
            let result: std::io::Result<()> = rt.block_on(async move {
                match tls_acceptor {
                    Some(acceptor) => {
                        run_https_from_listener(listener, state, acceptor, shutdown_rx).await
                    }
                    None => run_http_from_listener(listener, state, shutdown_rx)
                        .await
                        .map_err(std::io::Error::other),
                }
            });
            if let Err(e) = result {
                eprintln!("pyterrainmap: server error: {e}");
            }
        })
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to spawn server thread: {e}")))?;

    Ok(PyServerHandle {
        shutdown_tx: Some(shutdown_tx),
        join_handle: Some(join_handle),
        host,
        port,
        tls,
    })
}
