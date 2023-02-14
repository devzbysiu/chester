use crate::entities::repo_root::RepoRoot;
use crate::entities::status::TestsStatus;
use crate::result::ServerErr;
use crate::use_cases::state::State;
use crate::use_cases::state::{StateReader, StateWriter};

use actix_web::web::{Data, Json};
use actix_web::{get, middleware, post, App, HttpServer};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, instrument};
use tracing_actix_web::TracingLogger;

type Result<T> = std::result::Result<T, ServerErr>;
type StateWriterData = Data<StateWriter>;
type StateReaderData = Data<StateReader>;

#[instrument(skip(state))]
#[get("/tests/status")]
async fn status(state: StateReaderData) -> Result<Json<StatusResp>> {
    let status = state
        .status()
        .map_err(|_| ServerErr::Generic(anyhow!("Error while checking status.")))?;
    debug!("responding with {status}");
    Ok(Json(StatusResp::new(status)))
}

#[derive(Debug, Serialize)]
struct StatusResp {
    tests_status: TestsStatus,
}

impl StatusResp {
    fn new(tests_status: TestsStatus) -> Self {
        Self { tests_status }
    }
}

#[instrument(skip(state))]
#[post("/repo/root")]
async fn change_root(state: StateWriterData, req: Json<ChangeRootReq>) -> Result<String> {
    debug!("changing repo root");
    state
        .repo_root(req.repo_root.clone())
        .map_err(|_| ServerErr::Generic(anyhow!("Error while changing repo root.")))?;
    Ok("ok".into())
}

#[derive(Debug, Deserialize)]
struct ChangeRootReq {
    repo_root: RepoRoot,
}

#[instrument(skip(state))]
pub async fn start_server(state: State) -> std::io::Result<()> {
    let socket_path = dirs::runtime_dir().unwrap_or(PathBuf::from("/run"));
    let socket_path = socket_path.join("chester.sock");
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(middleware::DefaultHeaders::new())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .app_data(Data::new(state.reader()))
            .app_data(Data::new(state.writer()))
            .service(status)
            .service(change_root)
    })
    .bind_uds(socket_path)?
    .workers(1)
    .run()
    .await
}
