use db::dtos;
use futures::StreamExt;
use tracing::error;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), async_nats::Error> {
    let filter_layer = EnvFilter::from_default_env();
    let fmt_layer = fmt::layer().with_target(false).with_line_number(true);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set");
    let nats_client = async_nats::connect(nats_url)
        .await
        .expect("Failed to connect to NATS");

    let mut pipeline_nodes_exec_subscriber = nats_client.subscribe("pipeline.node.exec").await?;

    tokio::spawn(async move {
        while let Some(message) = pipeline_nodes_exec_subscriber.next().await {
            let payload =
                match serde_json::from_slice::<dtos::PipelineNodeExecPayload>(&message.payload) {
                    Ok(payload) => payload,
                    Err(error) => {
                        error!("Failed to deserialize message payload: {error:?}");
                        continue;
                    }
                };
        }
    });

    Ok(())
}
