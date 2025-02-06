use async_nats::jetstream;
use event_bridge::JETSTREAM_NAME;
use sqlx::postgres::PgListener;
use std::{env, io, process};
use tokio::signal::ctrl_c;
use tracing::{error, info};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

use dotenvy::dotenv;

static PG_EVENT_CHANNEL: &str = "pipeline_execs_events";

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let filter_layer = EnvFilter::from_default_env();
    let fmt_layer = fmt::layer().with_target(false).with_line_number(true);

    let (loki_layer, loki_task) = tracing_loki::builder()
        .label("host", "mine")
        .expect("Failed to create Loki layer")
        .extra_field("pid", format!("{}", process::id()))
        .expect("Failed to add extra field to Loki layer")
        .build_url(
            env::var("LOKI_URL")
                .expect("LOKI_URL must be set")
                .parse()
                .expect("Failed to parse Loki URL"),
        )
        .expect("Failed to build Loki layer");

    tokio::spawn(loki_task);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(loki_layer)
        .init();

    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set");
    let nats_client = async_nats::connect(nats_url)
        .await
        .expect("Failed to connect to NATS");

    let jetstream_events = jetstream::new(nats_client.clone());

    jetstream_events
        .get_or_create_stream(jetstream::stream::Config {
            name: JETSTREAM_NAME.to_string(),
            subjects: vec!["exec.events".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            ..Default::default()
        })
        .await
        .expect("Failed to get or create JetStream");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jetstream_events_clone = jetstream_events.clone();

    tokio::spawn(async move {
        let jetstream_events = jetstream_events_clone;

        let mut listener = match PgListener::connect(&database_url).await {
            Ok(listener) => listener,
            Err(error) => {
                error!("Failed to connect to Postgres: {error:?}");
                return;
            }
        };

        match listener.listen(PG_EVENT_CHANNEL).await {
            Ok(()) => {
                info!("Listening for notifications from Postgres");
            }
            Err(error) => {
                error!("Failed to listen for notifications from Postgres: {error:?}");
                return;
            }
        };

        while let Ok(Some(notification)) = listener.try_recv().await {
            let payload = notification.payload().to_string();

            if let Err(error) = jetstream_events
                .publish("exec.events", payload.into())
                .await
            {
                error!("Failed to publish message to JetStream: {error:?}");
            } else {
                info!("Published message to JetStream for notification: {notification:?}");
            }
        }
    });

    ctrl_c().await?;

    Ok(())
}
