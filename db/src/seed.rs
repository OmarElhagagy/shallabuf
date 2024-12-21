use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::collections::HashMap;
use uuid::Uuid;

use crate::dtos::{
    KeyProviderType, NodeConfig, NodeConfigV0, NodeContainerType, NodeInput, NodeInputType,
    NodeOutputType, PipelineTriggerConfig, PipelineTriggerConfigV0, SelectInput,
};
use sqlx::PgPool;

/// Seeds the database with initial data.
///
/// # Panics
///
/// This function panics if the database connection fails
/// or if a json config serialization fails.
#[allow(clippy::too_many_lines)]
pub async fn seed_database(db: &PgPool) -> anyhow::Result<()> {
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
        INSERT INTO pipeline_triggers (pipeline_id, config)
        VALUES ($1, $2)
        RETURNING id
        "#,
        pipeline.id,
        pipeline_trigger_config
    )
    .fetch_one(db)
    .await?;

    let echo_node_config = serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
        inputs: vec![NodeInput {
            name: "message".to_string(),
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
        }],
        outputs: vec![NodeOutputType::Text],
    }))
    .unwrap();

    let echo_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, identifier_name, description, publisher_name, container_type, config)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        "Echo",
        "echo",
        Some("A simple node that echoes the message it receives.".to_string()),
        "shallabuf",
        NodeContainerType::Wasm as NodeContainerType,
        echo_node_config
    )
    .fetch_one(db)
    .await?;

    let transformer_node_config = serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
        inputs: vec![
            NodeInput {
                name: "message".to_string(),
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
                name: "transformer".to_string(),
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
        outputs: vec![NodeOutputType::Text],
    }))
    .unwrap();

    let transformer_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, identifier_name, description, publisher_name, container_type, config)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        "Transformer",
        "transformer",
        Some("A simple node that transforms the message it receives.".to_string()),
        "shallabuf",
        NodeContainerType::Wasm as NodeContainerType,
        transformer_node_config
    )
    .fetch_one(db)
    .await?;

    let image_generator_node_config = serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
        inputs: vec![
            NodeInput {
                name: "width".to_string(),
                input: NodeInputType::Text {
                    default: Some("800".to_string()),
                },
                label: {
                    let mut map = HashMap::new();
                    map.insert("en".to_string(), "Width".to_string());
                    Some(map)
                },
                description: None,
                required: false,
            },
            NodeInput {
                name: "height".to_string(),
                input: NodeInputType::Text {
                    default: Some("600".to_string()),
                },
                label: {
                    let mut map = HashMap::new();
                    map.insert("en".to_string(), "Height".to_string());
                    Some(map)
                },
                description: None,
                required: false,
            },
        ],
        outputs: vec![NodeOutputType::Binary],
    }))
    .unwrap();

    let image_generator_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, identifier_name, description, publisher_name, container_type, config)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        "Image Generator",
        "image_generator",
        Some("A simple node that generates an image.".to_string()),
        "shallabuf",
        NodeContainerType::Wasm as NodeContainerType,
        image_generator_node_config
    )
    .fetch_one(db)
    .await?;

    let post_to_fb_node_config = serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
        inputs: vec![
            NodeInput {
                name: "message".to_string(),
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
                name: "media".to_string(),
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
        outputs: vec![NodeOutputType::Status],
    }))
    .unwrap();

    let post_to_fb_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, identifier_name, description, publisher_name, container_type, config)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        "Post to Facebook",
        "post_to_fb",
        Some("A simple node that posts to Facebook.".to_string()),
        "shallabuf",
        NodeContainerType::Wasm as NodeContainerType,
        post_to_fb_node_config
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

    let _echo_to_transformer_pipeline_connection = sqlx::query!(
        r#"
        INSERT INTO pipeline_nodes_connections (from_node_id, to_node_id)
        VALUES ($1, $2)
        "#,
        echo_pipeline_node.id,
        transformer_pipeline_node.id
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

    let _transformer_to_post_to_fb_pipeline_connection = sqlx::query!(
        r#"
        INSERT INTO pipeline_nodes_connections (from_node_id, to_node_id)
        VALUES ($1, $2)
        "#,
        transformer_pipeline_node.id,
        post_to_fb_pipeline_node.id
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

    let _image_generator_to_post_to_fb_pipeline_connection = sqlx::query!(
        r#"
        INSERT INTO pipeline_nodes_connections (from_node_id, to_node_id)
        VALUES ($1, $2)
        "#,
        image_generator_pipeline_node.id,
        post_to_fb_pipeline_node.id
    )
    .execute(db)
    .await?;

    Ok(())
}
