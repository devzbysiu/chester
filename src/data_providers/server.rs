use crate::entities::coverage::CoverageState;
use crate::entities::repo_root::RepoRoot;
use crate::entities::status::TestsState;
use crate::result::ServerErr;
use crate::use_cases::state::State;
use crate::use_cases::state::{StateReader, StateWriter};

use actix_web::web::{Data, Json};
use actix_web::{get, middleware, put, App, HttpResponse, HttpServer};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, instrument, trace};
use tracing_actix_web::TracingLogger;

type Result<T> = std::result::Result<T, ServerErr>;
type StateWriterData = Data<StateWriter>;
type StateReaderData = Data<StateReader>;

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
            .service(tests_status_endpt)
            .service(coverage_status_endpt)
            .service(change_root)
    })
    .bind_uds(socket_path)?
    .workers(1)
    .run()
    .await
}

#[instrument(level = "trace")]
#[get("/tests/status")]
async fn tests_status_endpt(state: StateReaderData) -> Result<Json<TestsStatusResp>> {
    let status = state
        .tests()
        .map_err(|_| server_err("Error while checking tests status."))?;
    trace!("responding with {status}");
    Ok(Json(TestsStatusResp::new(status)))
}

#[instrument(level = "trace")]
#[get("/coverage/status")]
async fn coverage_status_endpt(state: StateReaderData) -> Result<Json<CoverageStatusResp>> {
    let status = state
        .coverage()
        .map_err(|_| server_err("Error while checking coverage status."))?;
    trace!("responding with {status}");
    Ok(Json(CoverageStatusResp::new(status)))
}

fn server_err<S: Into<String>>(msg: S) -> ServerErr {
    ServerErr::Generic(anyhow!(msg.into()))
}

#[derive(Debug, Serialize)]
struct TestsStatusResp {
    tests_status: TestsState,
}

impl TestsStatusResp {
    fn new(tests_status: TestsState) -> Self {
        Self { tests_status }
    }
}

#[derive(Debug, Serialize)]
struct CoverageStatusResp {
    coverage_status: CoverageState,
}

impl CoverageStatusResp {
    fn new(coverage_status: CoverageState) -> Self {
        Self { coverage_status }
    }
}

#[instrument(skip(state))]
#[put("/repo/root")]
async fn change_root(state: StateWriterData, req: Json<ChangeRootReq>) -> Result<HttpResponse> {
    debug!("changing repo root to: {}", req.repo_root);
    state
        .repo_root(req.repo_root.clone())
        .map_err(|_| server_err("Error while changing repo root."))?;
    Ok(HttpResponse::NoContent().into()) // 204
}

#[derive(Debug, Deserialize)]
struct ChangeRootReq {
    repo_root: RepoRoot,
}
