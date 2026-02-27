use axum::{
    Json, Router,
    extract::{Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use rand::Rng;
use serde::{Deserialize, Serialize};

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
    Json(req): Json<SubmitRequest>,
) -> Result<(StatusCode, Json<SubmitResponse>), AppError> {
    let id = {
        let n: u32 = rand::rng().random_range(10000..=99999);
        format!("loan-{n}")
    };

    println!("Received application {:?}", req);

    Ok((
        StatusCode::CREATED,
        Json(SubmitResponse {
            application_id: id,
        }),
    ))
}

async fn get_application(
    Path(_id): Path<String>,
) -> Result<Json<()>, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let app = Router::new()
        .route(
            "/applications",
            post(submit_application),
        )
        .route(
            "/applications/{id}",
            get(get_application),
        );

    let listener =
        tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await?;
    println!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
