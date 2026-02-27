use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::client::{Client, ClientConfig};
use google_cloud_pubsub::publisher::Publisher;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use shared::{ApplicationStatus, LoanDecision, SUB_DECISIONS};
use shared::{LoanApplication, TOPIC_APPLICATIONS};
use futures_util::StreamExt;

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
    publisher: Publisher,
    store: ApplicationStore,
}

#[derive(Debug, Deserialize)]
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

    let app = LoanApplication {
        application_id: id.clone(),
        user_id: req.user_id,
        amount: req.amount,
        currency: req.currency,
        submitted_at: chrono::Utc::now().to_rfc3339(),
    };

    println!(
        "Submitting application {} for user {} ({} {})",
        app.application_id,
        app.user_id,
        app.amount,
        app.currency,
    );

    let payload = serde_json::to_vec(&app)
        .context("failed to serialize application")?;

    state
        .publisher
        .publish(PubsubMessage {
            data: payload.into(),
            ..Default::default()
        })
        .await
        .get()
        .await
        .context("failed to publish application")?;

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

async fn decision_listener(
    client: Client,
    store: ApplicationStore,
) {
    let subscription = client.subscription(SUB_DECISIONS);

    let mut stream = match subscription.subscribe(None).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Failed to subscribe to decisions: {e}"
            );
            return;
        }
    };

    println!(
        "Background listener subscribed to '{}'",
        SUB_DECISIONS,
    );

    while let Some(message) = stream.next().await {
        let data = &message.message.data;
        match serde_json::from_slice::<LoanDecision>(data) {
            Ok(decision) => {
                println!(
                    "Received decision for {}: {:?}",
                    decision.application_id,
                    decision.status,
                );
                let status_str =
                    serde_json::to_value(&decision.status)
                        .ok()
                        .and_then(|v| {
                            v.as_str().map(String::from)
                        })
                        .unwrap_or("UNKNOWN".into());

                let mut store = store.lock().await;
                if let Some(entry) =
                    store.get_mut(&decision.application_id)
                {
                    entry.status = status_str;
                    entry.interest_rate =
                        decision.interest_rate;
                    entry.max_term_months =
                        decision.max_term_months;
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to deserialize decision: {e}"
                );
            }
        }
        let _ = message.ack().await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ClientConfig::default();

    // auto-detects PUBSUB_EMULATOR_HOST from the environment
    let client = Client::new(config)
        .await
        .context("failed to create pubsub client")?;

    println!("Connected to Pub/Sub.");

    let topic = client.topic(TOPIC_APPLICATIONS);
    let publisher = topic.new_publisher(None);

    let store: ApplicationStore =
        Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(decision_listener(
        client.clone(),
        store.clone(),
    ));

    let state = AppState { publisher, store };

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
