use crate::entities::status::TestsStatus;
use crate::result::ServerErr;
use crate::use_cases::state::StateReader;

use actix_web::{get, middleware, web, App, HttpServer};
use anyhow::anyhow;
use serde::Serialize;
use std::path::PathBuf;
use tracing_actix_web::TracingLogger;

#[allow(clippy::unused_async)]
#[get("/tests/status")]
async fn status(state: web::Data<StateReader>) -> Result<web::Json<StatusResponse>, ServerErr> {
    let status = state
        .status()
        .map_err(|_| ServerErr::Generic(anyhow!("Error during exection.")))?;
    Ok(web::Json(StatusResponse::new(status)))
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    tests_status: TestsStatus,
}

impl StatusResponse {
    fn new(tests_status: TestsStatus) -> Self {
        Self { tests_status }
    }
}

pub async fn start_server(state: StateReader) -> std::io::Result<()> {
    let socket_path = dirs::runtime_dir().unwrap_or(PathBuf::from("/run"));
    let socket_path = socket_path.join("chester.sock");
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(middleware::DefaultHeaders::new())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(state.clone()))
            .service(status)
    })
    .bind_uds(socket_path)?
    .workers(1)
    .run()
    .await
}
