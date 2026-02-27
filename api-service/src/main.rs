use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use shared::ApplicationStatus;

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        eprintln!("Error: {:?}", self.0);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}

type ApplicationStore = Arc<Mutex<HashMap<String, ApplicationStatus>>>;

#[derive(Clone)]
struct AppState {
    store: ApplicationStore,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SubmitRequest {
    user_id: String,
    amount: u64,
    currency: String,
}

#[derive(Debug, Serialize)]
struct SubmitResponse {
    application_id: String,
}

async fn submit_application(
    State(state): State<AppState>,
    Json(req): Json<SubmitRequest>,
) -> Result<(StatusCode, Json<SubmitResponse>), AppError> {
    let id = {
        let n: u32 = rand::rng().random_range(10000..=99999);
        format!("loan-{n}")
    };

    println!("Received application {:?}", req);
    {
        let mut store = state.store.lock().await;
        store.insert(
            id.clone(),
            ApplicationStatus {
                application_id: id.clone(),
                status: "PENDING".into(),
                interest_rate: None,
                max_term_months: None,
            },
        );
    }

    Ok((
        StatusCode::CREATED,
        Json(SubmitResponse {
            application_id: id,
        }),
    ))
}

async fn get_application(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApplicationStatus>, StatusCode> {
    let store = state.store.lock().await;
    store
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store: ApplicationStore =
        Arc::new(Mutex::new(HashMap::new()));

    let state = AppState { store };

    let app = Router::new()
        .route(
            "/applications",
            post(submit_application),
        )
        .route(
            "/applications/{id}",
            get(get_application),
        )
        .with_state(state);

    let listener =
        tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await?;
    println!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
