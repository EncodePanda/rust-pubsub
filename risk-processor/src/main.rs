use anyhow::Context;
use futures_util::StreamExt;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::client::{Client, ClientConfig};
use rand::Rng;
use shared::{
    DecisionStatus, LoanApplication, LoanDecision,
    SUB_APPLICATIONS, TOPIC_DECISIONS,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ClientConfig::default();
    let client = Client::new(config)
        .await
        .context("failed to create pubsub client")?;

    println!("Connected to Pub/Sub.");

    let topic = client.topic(TOPIC_DECISIONS);
    let publisher = topic.new_publisher(None);

    let subscription = client.subscription(SUB_APPLICATIONS);
    let mut stream = subscription
        .subscribe(None)
        .await
        .context("failed to subscribe")?;

    println!(
        "Listening for loan applications on '{}'...",
        SUB_APPLICATIONS
    );

    while let Some(message) = stream.next().await {
        let data = &message.message.data;
        let app: LoanApplication = match serde_json::from_slice(data) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Failed to deserialize message: {e}");
                let _ = message.ack().await;
                continue;
            }
        };

        println!(
            "Processing application {} for user {} ({} {})",
            app.application_id,
            app.user_id,
            app.amount,
            app.currency,
        );

        let mut rng = rand::rng();
        let approved = rng.random_bool(0.7);

        let decision = if approved {
            let rate: f64 =
                (rng.random_range(3.0_f64..=12.0) * 100.0).round()
                    / 100.0;
            let term = rng.random_range(12..=60);
            LoanDecision {
                application_id: app.application_id.clone(),
                status: DecisionStatus::Approved,
                interest_rate: Some(rate),
                max_term_months: Some(term),
            }
        } else {
            LoanDecision {
                application_id: app.application_id.clone(),
                status: DecisionStatus::Rejected,
                interest_rate: None,
                max_term_months: None,
            }
        };

        println!(
            "Decision for {}: {:?}",
            decision.application_id, decision.status,
        );

        let payload = serde_json::to_vec(&decision)
            .context("failed to serialize decision")?;

        let awaiter = publisher
            .publish(PubsubMessage {
                data: payload.into(),
                ..Default::default()
            })
            .await;

        awaiter
            .get()
            .await
            .context("failed to publish decision")?;

        message.ack().await.context("failed to ack message")?;
    }

    Ok(())
}
