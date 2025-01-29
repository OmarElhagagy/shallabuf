use axum::response::Sse;
use axum::{extract::Path, response::sse::Event};
use db::dtos::{ExecStatus, PipelineExecEvent};
use futures::stream::Stream;
use futures::StreamExt;
use serde_json::json;
use std::error::Error;
use tracing::{debug, info};
use uuid::Uuid;

use crate::app_state::NatsSubscriber;

pub async fn subscribe(
    Path(id): Path<Uuid>,
    NatsSubscriber(subscriber): NatsSubscriber,
) -> Result<Sse<impl Stream<Item = Result<Event, Box<dyn Error + Send + Sync>>>>, hyper::StatusCode>
{
    debug!("Subscribing to JetStream for pipeline exec {id}");

    let stream = async_stream::stream! {
        let mut subscriber = subscriber.lock().await;

        while let Some(message) = subscriber.next().await {
            let message_str = String::from_utf8_lossy(&message.payload);
            debug!("Received message: {message_str}");

            if let Ok(raw_value) = serde_json::from_str::<serde_json::Value>(&message_str) {
                let wrapped_event = if raw_value.get("pipeline_node_id").is_some() {
                    json!({
                        "type": "node",
                        "data": raw_value
                    })
                } else {
                    json!({
                        "type": "pipeline",
                        "data": raw_value
                    })
                };

                if let Ok(exec) = serde_json::from_value::<PipelineExecEvent>(wrapped_event) {
                    let exec_id = match &exec {
                        PipelineExecEvent::Pipeline(pipeline_exec) => pipeline_exec.id,
                        PipelineExecEvent::Node(pipeline_node_exec) => pipeline_node_exec.pipeline_exec_id,
                    };

                    let pipeline_exec_finished = matches!(&exec,
                        PipelineExecEvent::Pipeline(pipeline_exec) if pipeline_exec.status == ExecStatus::Completed || pipeline_exec.status == ExecStatus::Failed
                    );

                    if exec_id == id {
                        if let Ok(exec_str) = serde_json::to_string(&exec) {
                            info!("Received message for pipeline exec {id}: {exec_str}");
                            yield Ok(Event::default().data(exec_str));

                            if pipeline_exec_finished {
                                debug!("Pipeline exec {id} completed, closing stream");
                                break;
                            }
                        }
                    } else {
                        debug!("Received message for different pipeline exec, {exec:?}");
                    }
                }
            } else {
                debug!("Failed to parse message as JSON: {message_str}");
            }
        }
    };

    Ok(Sse::new(stream))
}
