use async_nats::jetstream::{self, consumer::PullConsumer};
use futures::{StreamExt, TryStreamExt};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;

static JETSTREAM_NAME: &str = "PIPELINE_ACTIONS";

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let max_connections = std::env::var("MAX_CONNECTIONS")
        .unwrap_or("10".to_string())
        .parse::<u32>()
        .expect("MAX_CONNECTIONS must be a number");

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set");
    let nats_client = async_nats::connect(nats_url)
        .await
        .expect("Failed to connect to NATS");

    let jetstream = jetstream::new(nats_client);

    let consumer = jetstream
        .create_stream(JETSTREAM_NAME)
        .await?
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("pipeline_executor".into()),
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            filter_subject: "pipeline.exec".into(),
            ..Default::default()
        })
        .await?;

    let mut messages = consumer.fetch().messages().await?;

    while let Some(message) = messages.next().await {
        match message {
            Ok(msg) => {
                let payload = msg.payload.clone();
                let payload = std::str::from_utf8(&payload)?;
                let payload: serde_json::Value = serde_json::from_str(payload)?;
                let _ = format!("{payload:?}");
            }
            Err(error) => {
                eprintln!("Error processing message: {error:?}");
            }
        }
    }

    Ok(())
}
