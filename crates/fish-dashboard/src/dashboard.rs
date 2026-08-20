#![forbid(unsafe_code)]

use actix_web::{web, App, HttpServer, Responder};
use actix_cors::Cors;
use actix_files::Files;
use std::sync::Arc;

use crate::api::{ApiState, configure_routes};

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
    
    pub async fn start(self) -> std::io::Result<()> {
        let state = self.state;
        
        HttpServer::new(move || {
            let cors = Cors::permissive();
            
            App::new()
                .app_data(web::Data::new(state.clone()))
                .wrap(cors)
                .configure(configure_routes)
                .service(Files::new("/", "./static").index_file("index.html"))
        })
        .bind(("127.0.0.1", self.port))?
        .run()
        .await
    }
    
    pub fn spawn(self) -> tokio::task::JoinHandle<std::io::Result<()>> {
        tokio::spawn(async move {
            self.start().await
        })
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
