use crate::entities::check::CheckState;
use crate::entities::coverage::CoverageState;
use crate::entities::repo_root::RepoRoot;
use crate::entities::tests::TestsState;
use crate::result::ServerErr;
use crate::use_cases::state::State;
use crate::use_cases::state::{StateReader, StateWriter};

use actix_service::ServiceFactory;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
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
    HttpServer::new(move || app(&state))
        .bind_uds(socket_path)?
        .workers(1)
        .run()
        .await
}

// NOTE: Complex type taken from https://github.com/actix/actix-web/issues/1190
pub fn app(
    state: &State,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    App::new()
        .wrap(TracingLogger::default())
        .wrap(middleware::DefaultHeaders::new())
        .wrap(middleware::Compress::default())
        .wrap(middleware::Logger::default())
        .app_data(Data::new(state.reader()))
        .app_data(Data::new(state.writer()))
        .service(tests_status_endpt)
        .service(check_status_endpt)
        .service(coverage_status_endpt)
        .service(change_root)
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
#[get("/check/status")]
async fn check_status_endpt(state: StateReaderData) -> Result<Json<CheckStatusResp>> {
    let status = state
        .check()
        .map_err(|_| server_err("Error while checking tests status."))?;
    trace!("responding with {status}");
    Ok(Json(CheckStatusResp::new(status)))
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct TestsStatusResp {
    tests_status: TestsState,
}

impl TestsStatusResp {
    fn new(tests_status: TestsState) -> Self {
        Self { tests_status }
    }
}

#[derive(Debug, Serialize)]
struct CheckStatusResp {
    check_status: CheckState,
}

impl CheckStatusResp {
    fn new(check_status: CheckState) -> Self {
        Self { check_status }
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

#[cfg(test)]
mod test {
    use super::*;

    use crate::testingtools::state;

    use actix_web::body::to_bytes;
    use actix_web::test::{call_service, init_service, TestRequest};
    use anyhow::Result;
    use serde::de::DeserializeOwned;

    #[actix_web::test]
    async fn calling_tests_status_endpoint_returns_response_with_status() -> Result<()> {
        // given
        let svc = init_service(app(&state::working())).await;
        let req = TestRequest::default().uri("/tests/status").to_request();

        // when
        let resp = call_service(&svc, req).await;

        // then
        assert!(resp.status().is_success());
        let resp: TestsStatusResp = to_resp(resp).await;
        assert_eq!(resp.tests_status, TestsState::Success);

        Ok(())
    }

    async fn to_resp<T: DeserializeOwned>(resp: ServiceResponse<impl MessageBody>) -> T {
        let resp = resp.into_body();
        let Ok(resp) = to_bytes(resp).await else {
            panic!("failed to convert to bytes");
        };
        serde_json::from_slice(&resp).unwrap()
    }
}
