#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::flamegraph::FlamegraphGenerator;
use crate::metrics::{BuildMetrics, MetricsStore};

pub struct ApiState {
    pub metrics_store: Mutex<MetricsStore>,
}

impl ApiState {
    pub fn new() -> Self {
        Self {
            metrics_store: Mutex::new(MetricsStore::new(100)),
        }
    }
}

impl Default for ApiState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

pub fn handle_api_request(
    state: &Arc<ApiState>,
    method: &str,
    path: &str,
    body: &[u8],
) -> (u16, &'static str, Vec<u8>) {
    match (method, path) {
        ("GET", "/api/health") => {
            let res = ApiResponse::success("OK".to_string());
            let json = serde_json::to_vec(&res).unwrap_or_default();
            (200, "application/json", json)
        }
        ("GET", "/api/builds") => {
            let store = state.metrics_store.lock().unwrap();
            let builds = store.get_all_builds();
            let res = ApiResponse::success(builds);
            let json = serde_json::to_vec(&res).unwrap_or_default();
            (200, "application/json", json)
        }
        ("POST", "/api/builds") => {
            if let Ok(metrics) = serde_json::from_slice::<BuildMetrics>(body) {
                let mut store = state.metrics_store.lock().unwrap();
                store.add_build(metrics);
                let res = ApiResponse::success("Build metrics stored".to_string());
                let json = serde_json::to_vec(&res).unwrap_or_default();
                (200, "application/json", json)
            } else {
                let res = ApiResponse::<()>::error("Invalid JSON body".to_string());
                let json = serde_json::to_vec(&res).unwrap_or_default();
                (400, "application/json", json)
            }
        }
        ("GET", p) if p.starts_with("/api/builds/") && p.ends_with("/flamegraph") => {
            let id = &p["/api/builds/".len()..p.len() - "/flamegraph".len()];
            let store = state.metrics_store.lock().unwrap();
            match store.get_build(id) {
                Some(build) => {
                    let fg = FlamegraphGenerator::from_build_metrics(build);
                    let res = ApiResponse::success(fg);
                    let json = serde_json::to_vec(&res).unwrap_or_default();
                    (200, "application/json", json)
                }
                None => {
                    let res = ApiResponse::<()>::error("Build not found".to_string());
                    let json = serde_json::to_vec(&res).unwrap_or_default();
                    (404, "application/json", json)
                }
            }
        }
        ("GET", p) if p.starts_with("/api/builds/") => {
            let id = &p["/api/builds/".len()..];
            let store = state.metrics_store.lock().unwrap();
            match store.get_build(id) {
                Some(build) => {
                    let res = ApiResponse::success(build);
                    let json = serde_json::to_vec(&res).unwrap_or_default();
                    (200, "application/json", json)
                }
                None => {
                    let res = ApiResponse::<()>::error("Build not found".to_string());
                    let json = serde_json::to_vec(&res).unwrap_or_default();
                    (404, "application/json", json)
                }
            }
        }
        _ => {
            let res = ApiResponse::<()>::error("Not found".to_string());
            let json = serde_json::to_vec(&res).unwrap_or_default();
            (404, "application/json", json)
        }
    }
}
