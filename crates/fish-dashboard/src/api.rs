#![forbid(unsafe_code)]

use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

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

pub async fn get_builds(state: web::Data<ApiState>) -> impl Responder {
    let store = state.metrics_store.lock().unwrap();
    let builds = store.get_all_builds();
    HttpResponse::Ok().json(ApiResponse::success(builds))
}

pub async fn get_build(state: web::Data<ApiState>, path: web::Path<String>) -> impl Responder {
    let build_id = path.into_inner();
    let store = state.metrics_store.lock().unwrap();

    match store.get_build(&build_id) {
        Some(build) => HttpResponse::Ok().json(ApiResponse::success(build)),
        None => {
            HttpResponse::NotFound().json(ApiResponse::<()>::error("Build not found".to_string()))
        }
    }
}

pub async fn get_flamegraph(state: web::Data<ApiState>, path: web::Path<String>) -> impl Responder {
    let build_id = path.into_inner();
    let store = state.metrics_store.lock().unwrap();

    match store.get_build(&build_id) {
        Some(build) => {
            let fg = FlamegraphGenerator::from_build_metrics(build);
            HttpResponse::Ok().json(ApiResponse::success(fg))
        }
        None => {
            HttpResponse::NotFound().json(ApiResponse::<()>::error("Build not found".to_string()))
        }
    }
}

pub async fn post_build(
    state: web::Data<ApiState>,
    metrics: web::Json<BuildMetrics>,
) -> impl Responder {
    let mut store = state.metrics_store.lock().unwrap();
    store.add_build(metrics.into_inner());
    HttpResponse::Ok().json(ApiResponse::success("Build metrics stored".to_string()))
}

pub async fn get_health() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success("OK".to_string()))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(get_health))
            .route("/builds", web::get().to(get_builds))
            .route("/builds", web::post().to(post_build))
            .route("/builds/{id}", web::get().to(get_build))
            .route("/builds/{id}/flamegraph", web::get().to(get_flamegraph)),
    );
}
