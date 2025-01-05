use dotenvy::dotenv;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, CString};
use tokio::signal::ctrl_c;
use tracing::{debug, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use uuid::Uuid;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{preview1::WasiP1Ctx, WasiCtxBuilder};

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

            let mut config = Config::new();
            config.async_support(true);

            let engine = match Engine::new(&config) {
                Ok(engine) => engine,
                Err(error) => {
                    error!("Failed to create engine: {error}");
                    continue;
                }
            };

            let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
            match wasmtime_wasi::preview1::add_to_linker_async(&mut linker, |t| t) {
                Ok(()) => {}
                Err(error) => {
                    error!("Failed to add WASI to linker: {error}");
                    continue;
                }
            };

            let wasi = WasiCtxBuilder::new().inherit_stdio().build_p1();
            let mut store = Store::new(&engine, wasi);

            let s3_object = match s3_client
                .get_object()
                .bucket("builtins")
                .key("echo.wasm")
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

            let instance = match linker.instantiate_async(&mut store, &module).await {
                Ok(instance) => instance,
                Err(error) => {
                    error!("Failed to instantiate module asynchronously: {error}");
                    continue;
                }
            };

            let Some(memory) = instance.get_memory(&mut store, "memory") else {
                error!("Failed to get memory");
                continue;
            };

            let message = match CString::new(
                serde_json::json!({
                    "message": "Hello, World!"
                })
                .to_string(),
            ) {
                Ok(message) => message,
                Err(error) => {
                    error!("Failed to create message: {error}");
                    continue;
                }
            };

            let required_size = message.as_bytes_with_nul().len() as u64;
            let current_size = memory.data_size(&store) as u64;
            let new_size = current_size + required_size;

            let page_size = memory.page_size(&store);
            let required_pages = (new_size + page_size - 1) / page_size;
            let current_pages = memory.size(&store);

            if required_pages > current_pages {
                if let Err(error) = memory.grow(&mut store, required_pages - current_pages) {
                    error!("Failed to grow memory: {error}");
                    continue;
                }
            }

            let memory_ptr = current_size as usize;

            match memory.write(&mut store, memory_ptr, message.as_bytes_with_nul()) {
                Ok(()) => {}
                Err(error) => {
                    error!("Failed to write message to memory: {error}");
                    continue;
                }
            };

            let Ok(run_fn) = instance.get_typed_func::<u32, u32>(&mut store, "run") else {
                error!("Failed to get run function");
                continue;
            };

            let result_ptr = match run_fn.call_async(&mut store, memory_ptr as u32).await {
                Ok(result_ptr) => result_ptr as *const c_char,
                Err(error) => {
                    error!("Failed to call run_fn: {error}");
                    continue;
                }
            };

            if result_ptr.is_null() {
                error!("Received invalid pointer from run_fn");
                continue;
            }

            let mut buffer = Vec::new();

            let offset = result_ptr as usize;
            for i in offset..memory.data_size(&store) {
                let byte = memory.data(&store)[i];
                if byte == 0 {
                    break;
                }
                buffer.push(byte);
            }

            let result = match std::str::from_utf8(&buffer) {
                Ok(s) => s.to_string(),
                Err(error) => {
                    error!("Failed to convert result to string: {error}");
                    continue;
                }
            };

            let result = match serde_json::from_str(&result) {
                Ok(result) => result,
                Err(error) => {
                    error!("Failed to deserialize result: {error}");
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
