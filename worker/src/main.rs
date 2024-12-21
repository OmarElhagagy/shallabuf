use db::dtos;
use futures::StreamExt;
use tokio::signal::ctrl_c;
use tracing::{debug, error};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use wasmtime::{Caller, Engine, Linker, Module, Store};

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

    let mut pipeline_node_execs_subscriber = nats_client.subscribe("pipeline.node.exec").await?;

    tokio::spawn(async move {
        while let Some(message) = pipeline_node_execs_subscriber.next().await {
            let payload =
                match serde_json::from_slice::<dtos::PipelineNodeExecPayload>(&message.payload) {
                    Ok(payload) => payload,
                    Err(error) => {
                        error!("Failed to deserialize message payload: {error:?}");
                        continue;
                    }
                };

            let engine = Engine::default();
            let wat = r#"
                (module
                    (import "host" "host_func" (func $host_hello (param i32) (result i32)))

                    (func (export "hello") (result i32)
                        i32.const 3
                        call $host_hello)
                )
            "#;

            let module = match Module::new(&engine, wat) {
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

            let payload_bytes = match serde_json::to_string(&dtos::PipelineNodeExecResultPayload {
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
