use axum::response::Sse;
use axum::{extract::Path, response::sse::Event};
use db::dtos::{ExecStatus, PipelineExec};
use futures::stream::Stream;
use futures::StreamExt;
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

            match serde_json::from_str::<PipelineExec>(&message_str) {
                Ok(exec) => {
                    if exec.id == id {
                        match serde_json::to_string::<PipelineExec>(&exec) {
                            Ok(exec_str) => {
                                info!("Received message for pipeline exec {id}: {exec_str}");
                                yield Ok(Event::default().data(exec_str));

                                if exec.status == ExecStatus::Completed {
                                    debug!("Pipeline exec {id} completed, closing stream");
                                    break;
                                }
                            }
                            Err(error) => {
                                debug!("Failed to parse message: {error}");
                            }
                        }
                    } else {
                        debug!("Received message for different pipeline exec, {exec:?}");
                    }
                }
                Err(error) => {
                    debug!("Failed to parse message: {error}");
                }
            }
        }
    };

    Ok(Sse::new(stream))
}
