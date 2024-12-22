use dotenvy::dotenv;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::signal::ctrl_c;
use tracing::{debug, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use uuid::Uuid;
use wasmtime::{Caller, Engine, Linker, Module, Store};

#[derive(Clone, Serialize, Deserialize)]
pub struct PipelineNodeExecPayload {
    pub pipeline_execs_id: Uuid,
    pub pipeline_node_exec_id: Uuid,
    pub container_type: NodeContainerType,
    pub path: String,
    pub params: serde_json::Value,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "node_container_type", rename_all = "snake_case")]
pub enum NodeContainerType {
    Docker,
    Wasm,
}

impl From<&std::string::String> for NodeContainerType {
    fn from(s: &std::string::String) -> Self {
        match s.as_str() {
            "docker" => NodeContainerType::Docker,
            "wasm" => NodeContainerType::Wasm,
            _ => panic!("Invalid node container type"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PipelineNodeExecResultPayload {
    pub pipeline_exec_id: Uuid,
    pub pipeline_node_exec_id: Uuid,
    pub result: serde_json::Value,
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), async_nats::Error> {
    dotenv().ok();

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

    let mut pipeline_node_execs_subscriber = nats_client.subscribe("pipeline.node.exec").await?;

    let minio_endpoint = std::env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set");
    let minio_access_key = std::env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set");
    let minio_secret_key = std::env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set");

    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(minio_endpoint)
        .force_path_style(true)
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            minio_access_key,
            minio_secret_key,
            None,
            None,
            "",
        ))
        .region(aws_sdk_s3::config::Region::new("us-east-1"))
        .behavior_version_latest()
        .build();

    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

    tokio::spawn(async move {
        while let Some(message) = pipeline_node_execs_subscriber.next().await {
            let payload = match serde_json::from_slice::<PipelineNodeExecPayload>(&message.payload)
            {
                Ok(payload) => payload,
                Err(error) => {
                    error!("Failed to deserialize message payload: {error:?}");
                    continue;
                }
            };

            let engine = Engine::default();

            let s3_object = match s3_client
                .get_object()
                .bucket("builtins")
                .key("builtins.wasm")
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(error) => {
                    error!("Failed to download module from S3: {error}");
                    continue;
                }
            };

            let module_bytes =
                match aws_sdk_s3::primitives::ByteStream::collect(s3_object.body).await {
                    Ok(bytes) => bytes.into_bytes(),
                    Err(error) => {
                        error!("Failed to read module bytes: {error}");
                        continue;
                    }
                };

            let module = match Module::new(&engine, module_bytes) {
                Ok(module) => module,
                Err(error) => {
                    error!("Failed to compile module: {error}");
                    continue;
                }
            };

            let mut linker = Linker::new(&engine);

            match linker.func_wrap(
                "host",
                "host_func",
                |caller: Caller<'_, u32>, param: i32| {
                    debug!("Got {param} from WebAssembly");
                    debug!("my host state is: {}", caller.data());
                    Ok(param)
                },
            ) {
                Ok(_) => {}
                Err(error) => {
                    error!("Failed to link host function: {error}");
                    continue;
                }
            }

            let mut store = Store::new(&engine, 4);

            let instance = match linker.instantiate(&mut store, &module) {
                Ok(instance) => instance,
                Err(error) => {
                    error!("Failed to instantiate module: {error}");
                    continue;
                }
            };

            let hello = match instance.get_typed_func::<(), i32>(&mut store, "hello") {
                Ok(hello) => hello,
                Err(error) => {
                    error!("Failed to get typed function: {error}");
                    continue;
                }
            };

            let result = match match hello.call(&mut store, ()) {
                Ok(wasm_response) => serde_json::to_value(wasm_response),
                Err(error) => serde_json::to_value(error.to_string()),
            } {
                Ok(result) => result,
                Err(error) => {
                    error!("Failed to serialize result: {error}");
                    continue;
                }
            };

            let payload_bytes = match serde_json::to_string(&PipelineNodeExecResultPayload {
                pipeline_exec_id: payload.pipeline_execs_id,
                pipeline_node_exec_id: payload.pipeline_node_exec_id,
                result,
            }) {
                Ok(payload) => payload.into(),
                Err(error) => {
                    error!("Failed to serialize payload: {error}");
                    continue;
                }
            };

            if let Err(error) = nats_client
                .publish("pipeline.node.result", payload_bytes)
                .await
            {
                error!("Failed to publish message to JetStream: {error:?}");
            } else {
                debug!(
                    "Published message to JetStream for pipeline_node_exec_id {}",
                    payload.pipeline_node_exec_id
                );
            }
        }
    });

    ctrl_c().await?;

    Ok(())
}
