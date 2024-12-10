use async_nats::{self, jetstream};
use db::dtos::{self, PipelineExecPayloadParams};
use db::{entities::pipeline_exec, seed::seed_database, MigratorTrait};
use pool::Db;
use rocket::{fairing, serde::Serialize, Build, Rocket};
use rocket::{fairing::AdHoc, serde::json::Json};
use rocket::{http::Status, State};
use sea_orm::{ActiveValue::Set, EntityTrait};
use sea_orm_rocket::{Connection, Database};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};
use uuid::Uuid;

mod pool;

#[macro_use]
extern crate rocket;

static JETSTREAM_NAME: &str = "PIPELINE_ACTIONS";

#[derive(Serialize)]
struct PipelineTriggerResponse {
    pipeline_exec_id: Uuid,
}

#[post("/trigger/pipelines/<id>", format = "json", data = "<params>")]
async fn trigger_pipeline(
    id: &str,
    conn: Connection<'_, Db>,
    stream: &State<jetstream::Context>,
    params: Json<PipelineExecPayloadParams>,
) -> Result<Json<PipelineTriggerResponse>, Status> {
    info!("Received request to trigger pipeline with id: {id}");
    let db = conn.into_inner();

    let pipeline_id = Uuid::parse_str(id).map_err(|_| {
        warn!("Invalid UUID format for pipeline id: {id}");
        Status::BadRequest
    })?;

    let pipeline_exec = pipeline_exec::Entity::insert(pipeline_exec::ActiveModel {
        pipeline_id: Set(pipeline_id),
        ..Default::default()
    })
    .exec(db)
    .await
    .map_err(|error| {
        error!("Database error: {error:?}");
        Status::InternalServerError
    })?;

    info!(
        "Pipeline execution record created with id: {}",
        pipeline_exec.last_insert_id
    );

    let payload_bytes = serde_json::to_string(&dtos::PipelineExecPayload {
        pipeline_id,
        pipeline_exec_id: pipeline_exec.last_insert_id,
        params: params.into_inner(),
    })
    .map_err(|error| {
        error!("Failed to serialize payload: {error:?}");
        Status::InternalServerError
    })?
    .into();

    if let Err(error) = stream.publish("pipeline.exec", payload_bytes).await {
        error!("Failed to publish message to JetStream: {error:?}");
    } else {
        debug!(
            "Published message to JetStream for pipeline_exec_id: {}",
            pipeline_exec.last_insert_id
        );
    }

    Ok(Json(PipelineTriggerResponse {
        pipeline_exec_id: pipeline_exec.last_insert_id,
    }))
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    let Some(db) = Db::fetch(&rocket) else {
        return Err(rocket);
    };

    let conn = &db.conn;

    let rust_env = std::env::var("RUST_ENV").unwrap_or("dev".to_string());

    if rust_env == "dev" {
        match db::Migrator::fresh(conn).await {
            Ok(()) => info!("Database freshed successfully"),
            Err(error) => error!("Failed to fresh database: {error:?}"),
        };

        match seed_database(conn).await {
            Ok(()) => info!("Database seeded successfully"),
            Err(error) => error!("Failed to seed database: {error:?}"),
        };
    } else {
        match db::Migrator::up(conn, None).await {
            Ok(()) => info!("Database migrated successfully"),
            Err(error) => error!("Failed to migrate database: {error:?}"),
        };
    }

    Ok(rocket)
}

#[launch]
async fn rocket() -> _ {
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

    let jetstream = jetstream::new(nats_client);

    jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: JETSTREAM_NAME.to_string(),
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            subjects: vec!["pipeline.>".to_string()],
            ..Default::default()
        })
        .await
        .expect("Failed to get or create JetStream");

    rocket::build()
        .manage(jetstream)
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .mount("/", routes![trigger_pipeline])
}
