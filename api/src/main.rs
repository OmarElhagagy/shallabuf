use async_nats::{self, jetstream};
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{http::Status, State};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;

#[macro_use]
extern crate rocket;

static JETSTREAM_NAME: &str = "PIPELINE_ACTIONS";

#[derive(Serialize, sqlx::FromRow)]
struct PipelineTriggerResponse {
    id: Uuid,
}

#[post("/trigger/pipeline/<id>", format = "json", data = "<params>")]
async fn trigger_pipeline(
    id: &str,
    db: &State<Pool<Postgres>>,
    stream: &State<jetstream::Context>,
    params: Json<serde_json::Value>,
) -> Result<Json<PipelineTriggerResponse>, Status> {
    let pipeline_id = Uuid::parse_str(id).map_err(|_| Status::BadRequest)?;

    let result = sqlx::query_as::<Postgres, PipelineTriggerResponse>(
        "INSERT INTO pipeline_exec (pipeline_id, status) VALUES (?, ?) RETURNING id",
    )
    .bind(pipeline_id)
    .bind("Pending")
    .fetch_one(db.inner())
    .await
    .map(Json)
    .map_err(|_| Status::InternalServerError);

    if let Ok(response) = &result {
        let payload = serde_json::json!({
            "pipeline_exec_id": response.id.to_string(),
            "pipeline_id": pipeline_id.to_string(),
            "params": params.into_inner(),
        })
        .to_string()
        .into_bytes();

        stream
            .publish("pipeline.exec", payload.into())
            .await
            .expect("Failed to publish message to JetStream");
    }

    result
}

#[launch]
async fn rocket() -> _ {
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

    let _ = jetstream
        .create_stream(jetstream::stream::Config {
            name: JETSTREAM_NAME.to_string(),
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            subjects: vec!["pipeline.>".to_string()],
            ..Default::default()
        })
        .await
        .expect("Failed to create JetStream stream");

    rocket::build()
        .manage(pool)
        .manage(jetstream)
        .mount("/", routes![trigger_pipeline])
}
