use axum::response::Sse;
use axum::{extract::Path, response::sse::Event};
use db::dtos::{ExecStatus, PipelineExecEvent};
use futures::stream::Stream;
use futures::stream::StreamExt;
use serde_json::json;
use std::error::Error;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::app_state::ExecEventsConsumer;

pub async fn subscribe(
    Path(id): Path<Uuid>,
    ExecEventsConsumer(exec_events_consumer): ExecEventsConsumer,
) -> Result<Sse<impl Stream<Item = Result<Event, Box<dyn Error + Send + Sync>>>>, hyper::StatusCode>
{
    debug!("Subscribing directly to NATS for pipeline exec {id}");

    let stream = async_stream::stream! {
        'outer: loop {
            match exec_events_consumer.clone().messages().await {
                Ok(mut messages) => {
                    while let Some(Ok(message)) = messages.next().await {
                        if let Err(err) = message.ack().await {
                            error!("Failed to ack message: {err}");
                        }

                        let message_str = String::from_utf8_lossy(&message.payload);

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

                                    }
                                } else {
                                    debug!("Received message for different pipeline exec, {exec:?}");
                                }

                                if pipeline_exec_finished {
                                    debug!("Pipeline exec {id} completed, closing stream");
                                    break 'outer;
                                }
                            } else {
                                error!("Failed to parse message: {message_str}");
                            }
                        }
                    }
                }
                Err(err) => {
                    error!("Error getting messages: {err}");
                    break 'outer;
                }
            }
        }
    };

    Ok(Sse::new(stream))
}
