use std::convert::Infallible;

use axum::response::Sse;
use axum::{extract::Path, response::sse::Event};
use futures::stream::{self, Stream};
use futures::StreamExt;
use hyper::StatusCode;
use uuid::Uuid;

use crate::{app_state::JetStreamConsumer, utils::internal_error};

pub async fn subscribe(
    Path(id): Path<Uuid>,
    JetStreamConsumer(jetstream_consumer): JetStreamConsumer,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, hyper::StatusCode> {
    let messages = jetstream_consumer
        .messages()
        .await
        .map_err(internal_error)?;

    let stream = messages.map(|message_result| {
        let message = message_result.unwrap_or_else(|_| {
            panic!("Failed to receive message from JetStream");
        });

        let data = String::from_utf8(message.payload.to_vec())
            .unwrap_or_else(|_| String::from("Invalid UTF-8"));

        Ok(Event::default().data(data))
    });

    Ok(Sse::new(stream))
}
