use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::dtos::{
    KeyProviderType, NodeConfig, NodeConfigV0, NodeContainerType, NodeInput, NodeInputType,
    NodeOutput, NodeOutputType, PipelineTriggerConfig, PipelineTriggerConfigV0, SelectInput,
};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::types::BucketVersioningStatus;
use sqlx::PgPool;

/// Seeds the database with initial data.
///
/// # Panics
///
/// This function panics if the database connection fails
/// or if a json config serialization fails.
#[allow(clippy::too_many_lines)]
pub async fn seed_database(db: &PgPool) -> anyhow::Result<()> {
    // Setup S3 client
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

    // Check if the bucket exists, and create it if it does not
    let bucket_name = "builtins";
    match s3_client.head_bucket().bucket(bucket_name).send().await {
        Ok(_) => {}
        Err(ref error) => {
            if matches!(error, SdkError::ServiceError(ref err) if err.err().is_not_found()) {
                s3_client.create_bucket().bucket(bucket_name).send().await?;

                // Enable versioning on the bucket
                s3_client
                    .put_bucket_versioning()
                    .bucket(bucket_name)
                    .versioning_configuration(
                        aws_sdk_s3::types::VersioningConfiguration::builder()
                            .status(BucketVersioningStatus::Enabled)
                            .build(),
                    )
                    .send()
                    .await?;
            } else {
                return Err(anyhow::anyhow!(error.to_string()));
            }
        }
    }

    let echo_wasm_path = "./builtins/echo.wasm";
    let echo_body = tokio::fs::read(echo_wasm_path).await?;

    let put_object_output = s3_client
        .put_object()
        .bucket(bucket_name)
        .key("echo:v1.wasm")
        .body(echo_body.into())
        .send()
        .await?;

    let echo_wasm_id = put_object_output.version_id().unwrap_or_default();

    let text_transformer_wasm_path = "./builtins/text-transformer.wasm";
    let text_transformer_body = tokio::fs::read(text_transformer_wasm_path).await?;

    let put_object_output = s3_client
        .put_object()
        .bucket(bucket_name)
        .key("text-transformer:v1.wasm")
        .body(text_transformer_body.into())
        .send()
        .await?;

    let text_transform_wasm_id = put_object_output.version_id().unwrap_or_default();

    let btc_price_wasm_path = "./builtins/btc-price.wasm";
    let btc_price_body = tokio::fs::read(btc_price_wasm_path).await?;

    let put_object_output = s3_client
        .put_object()
        .bucket(bucket_name)
        .key("btc-price:v1.wasm")
        .body(btc_price_body.into())
        .send()
        .await?;

    let btc_price_wasm_id = put_object_output.version_id().unwrap_or_default();

    // Define predetermined UUIDs
    let organization_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
    let team_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap();
    let pipeline_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174002").unwrap();
    let alex_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174003").unwrap();
    let bob_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174004").unwrap();

    let organization = sqlx::query!(
        r#"
        INSERT INTO organizations (id, name)
        VALUES ($1, $2)
        RETURNING id
        "#,
        organization_id,
        "Aurora Innovations"
    )
    .fetch_one(db)
    .await?;

    let team = sqlx::query!(
        r#"
        INSERT INTO teams (id, name, organization_id)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        team_id,
        "Stellar Team",
        organization.id
    )
    .fetch_one(db)
    .await?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hashed_password = argon2
        .hash_password(b"alexpass", &salt)
        .map_err(|error| anyhow::anyhow!(error))?
        .to_string();

    let user_alex = sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, password_hash, email_verified, organization_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        alex_id,
        "Alex",
        "alex@mail.com",
        hashed_password,
        true,
        organization_id
    )
    .fetch_one(db)
    .await?;

    let _user_alex_key = sqlx::query!(
        r#"
        INSERT INTO keys (user_id, provider, provider_key)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        user_alex.id,
        KeyProviderType::Password as KeyProviderType,
        "alex@mail.com"
    )
    .fetch_one(db)
    .await?;

    let hashed_password = argon2
        .hash_password(b"bobpass", &salt)
        .map_err(|error| anyhow::anyhow!(error))?
        .to_string();

    let user_bob = sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, password_hash, email_verified, organization_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        bob_id,
        "Bob",
        "bob@mail.com",
        hashed_password,
        true,
        organization_id
    )
    .fetch_one(db)
    .await?;

    let _user_bob_key = sqlx::query!(
        r#"
        INSERT INTO keys (user_id, provider, provider_key)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        user_bob.id,
        KeyProviderType::Password as KeyProviderType,
        "bob@mail.com"
    )
    .fetch_one(db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO user_teams (user_id, team_id)
        VALUES ($1, $2)
        "#,
        user_alex.id,
        team_id
    )
    .execute(db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO user_teams (user_id, team_id)
        VALUES ($1, $2)
        "#,
        user_bob.id,
        team_id
    )
    .execute(db)
    .await?;

    let pipeline = sqlx::query!(
        r#"
        INSERT INTO pipelines (id, name, description, team_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        pipeline_id,
        "Quantum Pipeline",
        Some("A cutting-edge pipeline for quantum data.".to_string()),
        team.id
    )
    .fetch_one(db)
    .await?;

    let pipeline_trigger_config =
        serde_json::to_value(PipelineTriggerConfig::V0(PipelineTriggerConfigV0 {
            allow_manual_execution: true,
        }))
        .unwrap();

    let pipeline_trigger = sqlx::query!(
        r#"
        INSERT INTO pipeline_triggers (pipeline_id, coords, config)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        pipeline.id,
        serde_json::json!({
            "x": -154,
            "y": -112,
        }),
        pipeline_trigger_config
    )
    .fetch_one(db)
    .await?;

    let echo_node_config = serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
        inputs: vec![NodeInput {
            key: "message".to_string(),
            input: NodeInputType::Text {
                default: Some("Hello World!".to_string()),
            },
            label: {
                let mut map = HashMap::new();
                map.insert("en".to_string(), "Message".to_string());
                Some(map)
            },
            description: None,
            required: false,
        }],
        outputs: vec![NodeOutput {
            key: "echoed".to_string(),
            output: NodeOutputType::Text,
            label: {
                let mut map = HashMap::new();
                map.insert("en".to_string(), "Echoed".to_string());
                Some(map)
            },
            description: None,
        }],
    }))
    .unwrap();

    let echo_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, identifier_name, description, publisher_name, container_type, config, version_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        "Echo",
        "echo",
        Some("A simple node that echoes the message it receives.".to_string()),
        "builtins",
        NodeContainerType::Wasm as NodeContainerType,
        echo_node_config,
        echo_wasm_id
    )
    .fetch_one(db)
    .await?;

    let transformer_node_config = serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
        inputs: vec![
            NodeInput {
                key: "message".to_string(),
                input: NodeInputType::Text {
                    default: Some(String::new()),
                },
                label: {
                    let mut map = HashMap::new();
                    map.insert("en".to_string(), "Message".to_string());
                    Some(map)
                },
                description: None,
                required: false,
            },
            NodeInput {
                key: "transformer".to_string(),
                input: NodeInputType::Select {
                    options: vec![
                        SelectInput {
                            value: "uppercase".to_string(),
                            label: {
                                let mut map = HashMap::new();
                                map.insert("en".to_string(), "Uppercase".to_string());
                                map
                            },
                        },
                        SelectInput {
                            value: "lowercase".to_string(),
                            label: {
                                let mut map = HashMap::new();
                                map.insert("en".to_string(), "Lowercase".to_string());
                                map
                            },
                        },
                    ],
                    default: Some("uppercase".to_string()),
                },
                label: {
                    let mut map = HashMap::new();
                    map.insert("en".to_string(), "Transformer".to_string());
                    Some(map)
                },
                description: None,
                required: false,
            },
        ],
        outputs: vec![NodeOutput {
            key: "transformed".to_string(),
            output: NodeOutputType::Text,
            label: {
                let mut map = HashMap::new();
                map.insert("en".to_string(), "Transformed".to_string());
                Some(map)
            },
            description: None,
        }],
    }))
    .unwrap();

    let transformer_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, identifier_name, description, publisher_name, container_type, config, version_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        "Text transformer",
        "text-transformer",
        Some("A simple node that transforms the message it receives.".to_string()),
        "builtins",
        NodeContainerType::Wasm as NodeContainerType,
        transformer_node_config,
        text_transform_wasm_id
    )
    .fetch_one(db)
    .await?;

    let image_generator_node_config = serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
        inputs: vec![NodeInput {
            key: "prompt".to_string(),
            input: NodeInputType::Text {
                default: Some("The quick brown fox jumps over the lazy dog".to_string()),
            },
            label: {
                let mut map = HashMap::new();
                map.insert("en".to_string(), "Prompt".to_string());
                Some(map)
            },
            description: None,
            required: false,
        }],
        outputs: vec![NodeOutput {
            key: "image".to_string(),
            output: NodeOutputType::Binary,
            label: {
                let mut map = HashMap::new();
                map.insert("en".to_string(), "Image".to_string());
                Some(map)
            },
            description: None,
        }],
    }))
    .unwrap();

    let image_generator_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, identifier_name, description, publisher_name, container_type, config, version_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        "Image Generator",
        "image_generator",
        Some("A simple node that generates an image.".to_string()),
        "builtins",
        NodeContainerType::Wasm as NodeContainerType,
        image_generator_node_config,
        "FIXME"
    )
    .fetch_one(db)
    .await?;

    let post_to_fb_node_config = serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
        inputs: vec![
            NodeInput {
                key: "message".to_string(),
                input: NodeInputType::Text {
                    default: Some(String::new()),
                },
                label: {
                    let mut map = HashMap::new();
                    map.insert("en".to_string(), "Message".to_string());
                    Some(map)
                },
                description: None,
                required: false,
            },
            NodeInput {
                key: "media".to_string(),
                input: NodeInputType::Binary,
                label: {
                    let mut map = HashMap::new();
                    map.insert("en".to_string(), "Media".to_string());
                    Some(map)
                },
                description: None,
                required: false,
            },
        ],
        outputs: vec![NodeOutput {
            key: "posted".to_string(),
            output: NodeOutputType::Status,
            label: {
                let mut map = HashMap::new();
                map.insert("en".to_string(), "Posted".to_string());
                Some(map)
            },
            description: None,
        }],
    }))
    .unwrap();

    let post_to_fb_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, identifier_name, description, publisher_name, container_type, config, version_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        "Post to Facebook",
        "post_to_fb",
        Some("A simple node that posts to Facebook.".to_string()),
        "builtins",
        NodeContainerType::Wasm as NodeContainerType,
        post_to_fb_node_config,
        "FIXME"
    )
    .fetch_one(db)
    .await?;

    let btc_price_node_config = serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
        inputs: vec![],
        outputs: vec![NodeOutput {
            key: "price".to_string(),
            output: NodeOutputType::Text,
            label: {
                let mut map = HashMap::new();
                map.insert("en".to_string(), "Price".to_string());
                Some(map)
            },
            description: None,
        }],
    }))
    .unwrap();

    let btc_price_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, identifier_name, description, publisher_name, container_type, config, version_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        "BTC Price",
        "btc-price",
        Some("A simple node that fetches the current BTC price.".to_string()),
        "builtins",
        NodeContainerType::Wasm as NodeContainerType,
        btc_price_node_config,
        btc_price_wasm_id
    )
    .fetch_one(db)
    .await?;

    let echo_pipeline_node = sqlx::query!(
        r#"
        INSERT INTO pipeline_nodes (pipeline_id, node_id, trigger_id, coords, node_version)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        pipeline.id,
        echo_node.id,
        Some(pipeline_trigger.id),
        serde_json::json!({
            "x": 2.0,
            "y": -92.0,
        }),
        "v1"
    )
    .fetch_one(db)
    .await?;

    let echo_pipeline_node_output_echoed = sqlx::query!(
        r#"
        SELECT id FROM pipeline_node_outputs
        WHERE pipeline_node_id = $1
        "#,
        echo_pipeline_node.id
    )
    .fetch_one(db)
    .await?;

    let transformer_pipeline_node = sqlx::query!(
        r#"
        INSERT INTO pipeline_nodes (pipeline_id, node_id, coords, node_version)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        pipeline.id,
        transformer_node.id,
        serde_json::json!({
            "x": 285.0,
            "y": -18.0,
        }),
        "v1"
    )
    .fetch_one(db)
    .await?;

    let transform_pipeline_node_input_message = sqlx::query!(
        r#"
        SELECT id FROM pipeline_node_inputs
        WHERE pipeline_node_id = $1
        "#,
        transformer_pipeline_node.id
    )
    .fetch_one(db)
    .await?;

    let transform_pipeline_node_output_transformed = sqlx::query!(
        r#"
        SELECT id FROM pipeline_node_outputs
        WHERE pipeline_node_id = $1
        "#,
        transformer_pipeline_node.id
    )
    .fetch_one(db)
    .await?;

    let _echo_to_transformer_pipeline_connection = sqlx::query!(
        r#"
        INSERT INTO pipeline_node_connections (from_pipeline_node_output_id, to_pipeline_node_input_id)
        VALUES ($1, $2)
        "#,
        echo_pipeline_node_output_echoed.id,
        transform_pipeline_node_input_message.id
    )
    .execute(db)
    .await?;

    let post_to_fb_pipeline_node = sqlx::query!(
        r#"
        INSERT INTO pipeline_nodes (pipeline_id, node_id, coords, node_version)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        pipeline.id,
        post_to_fb_node.id,
        serde_json::json!({
            "x": 620.0,
            "y": -109.0,
        }),
        "v1"
    )
    .fetch_one(db)
    .await?;

    let post_to_fb_pipeline_node_input_message = sqlx::query!(
        r#"
        SELECT id FROM pipeline_node_inputs
        WHERE pipeline_node_id = $1 AND key = 'message'
        "#,
        post_to_fb_pipeline_node.id
    )
    .fetch_one(db)
    .await?;

    let post_to_fb_pipeline_node_input_media = sqlx::query!(
        r#"
        SELECT id FROM pipeline_node_inputs
        WHERE pipeline_node_id = $1 AND key = 'media'
        "#,
        post_to_fb_pipeline_node.id
    )
    .fetch_one(db)
    .await?;

    let _transformer_to_post_to_fb_pipeline_connection = sqlx::query!(
        r#"
        INSERT INTO pipeline_node_connections (from_pipeline_node_output_id, to_pipeline_node_input_id)
        VALUES ($1, $2)
        "#,
        transform_pipeline_node_output_transformed.id,
        post_to_fb_pipeline_node_input_message.id
    )
    .execute(db)
    .await?;

    let image_generator_pipeline_node = sqlx::query!(
        r#"
            INSERT INTO pipeline_nodes (pipeline_id, node_id, coords, node_version)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        pipeline.id,
        image_generator_node.id,
        serde_json::json!({
            "x": 284.0,
            "y": -282.0,
        }),
        "v1"
    )
    .fetch_one(db)
    .await?;

    let image_generator_pipeline_node_output_image = sqlx::query!(
        r#"
        SELECT id FROM pipeline_node_outputs
        WHERE pipeline_node_id = $1
        "#,
        image_generator_pipeline_node.id
    )
    .fetch_one(db)
    .await?;

    let _image_generator_to_post_to_fb_pipeline_connection = sqlx::query!(
        r#"
        INSERT INTO pipeline_node_connections (from_pipeline_node_output_id, to_pipeline_node_input_id)
        VALUES ($1, $2)
        "#,
        image_generator_pipeline_node_output_image.id,
        post_to_fb_pipeline_node_input_media.id
    )
    .execute(db)
    .await?;

    Ok(())
}
