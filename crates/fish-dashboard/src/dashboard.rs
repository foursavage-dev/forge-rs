#![forbid(unsafe_code)]

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use std::sync::Arc;
use std::thread;

use crate::api::{ApiState, configure_routes};

const INDEX_HTML: &str = include_str!("../static/index.html");

async fn index_handler() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(INDEX_HTML)
}

pub struct DashboardServer {
    port: u16,
    state: Arc<ApiState>,
}

impl DashboardServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            state: Arc::new(ApiState::new()),
        }
    }

    pub fn state(&self) -> Arc<ApiState> {
        self.state.clone()
    }

    pub fn run_blocking(self) -> std::io::Result<()> {
        let state = self.state;
        let port = self.port;

        actix_web::rt::System::new().block_on(async move {
            HttpServer::new(move || {
                let cors = Cors::permissive();

                App::new()
                    .app_data(web::Data::new(state.clone()))
                    .wrap(cors)
                    .configure(configure_routes)
                    .route("/", web::get().to(index_handler))
                    .route("/index.html", web::get().to(index_handler))
            })
            .bind(("127.0.0.1", port))?
            .run()
            .await
        })
    }

    pub fn spawn(self) -> thread::JoinHandle<std::io::Result<()>> {
        thread::spawn(move || self.run_blocking())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let dashboard = DashboardServer::new(8080);
        assert_eq!(dashboard.port, 8080);
    }
}
