use std::collections::HashMap;

use crate::dtos::{
    NodeConfig, NodeConfigV0, NodeContainerType, NodeInput, NodeInputType, NodeOutputType,
    PipelineTriggerConfig, PipelineTriggerConfigV0, SelectInput,
};
use sqlx::PgPool;

/// Seeds the database with initial data.
///
/// # Panics
///
/// This function panics if the database connection fails
/// or if a json config serialization fails.
#[allow(clippy::too_many_lines)]
pub async fn seed_database(db: &PgPool) -> Result<(), sqlx::Error> {
    // Insert into organizations
    let organization = sqlx::query!(
        r#"
        INSERT INTO organizations (name)
        VALUES ($1)
        RETURNING id
        "#,
        "Organization 1"
    )
    .fetch_one(db)
    .await?;

    // Insert into teams
    let team = sqlx::query!(
        r#"
        INSERT INTO teams (name, organization_id)
        VALUES ($1, $2)
        RETURNING id
        "#,
        "Team 1",
        organization.id
    )
    .fetch_one(db)
    .await?;

    // Insert into pipelines
    let pipeline = sqlx::query!(
        r#"
        INSERT INTO pipelines (name, description, team_id)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        "Pipeline 1",
        Some("Description 1".to_string()),
        team.id
    )
    .fetch_one(db)
    .await?;

    // Insert into pipeline_triggers
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

    // Insert into nodes (echo_node)
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
        INSERT INTO nodes (name, description, publisher_name, container_type, config)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        "Echo",
        Some("A simple node that echoes the message it receives.".to_string()),
        "shallabuf",
        NodeContainerType::Wasm as NodeContainerType,
        echo_node_config
    )
    .fetch_one(db)
    .await?;

    // Insert into nodes (transformer_node)
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
        INSERT INTO nodes (name, description, publisher_name, container_type, config)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        "Transformer",
        Some("A simple node that transforms the message it receives.".to_string()),
        "shallabuf",
        NodeContainerType::Wasm as NodeContainerType,
        transformer_node_config
    )
    .fetch_one(db)
    .await?;

    // Insert into nodes (image_generator_node)
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
        outputs: vec![NodeOutputType::Text],
    }))
    .unwrap();

    let image_generator_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, description, publisher_name, container_type, config)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        "Image Generator",
        Some("A simple node that generates an image.".to_string()),
        "shallabuf",
        NodeContainerType::Wasm as NodeContainerType,
        image_generator_node_config
    )
    .fetch_one(db)
    .await?;

    // Insert into nodes (post_to_fb_node)
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
        outputs: vec![NodeOutputType::Text],
    }))
    .unwrap();

    let post_to_fb_node = sqlx::query!(
        r#"
        INSERT INTO nodes (name, description, publisher_name, container_type, config)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        "Post to Facebook",
        Some("A simple node that posts to Facebook.".to_string()),
        "shallabuf",
        NodeContainerType::Wasm as NodeContainerType,
        post_to_fb_node_config
    )
    .fetch_one(db)
    .await?;

    // Insert into pipeline_nodes (echo_pipeline_node)
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
            "x": 100,
            "y": 100
        }),
        "latest"
    )
    .fetch_one(db)
    .await?;

    // Insert into pipeline_nodes (transformer_pipeline_node)
    let transformer_pipeline_node = sqlx::query!(
        r#"
        INSERT INTO pipeline_nodes (pipeline_id, node_id, coords, node_version)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        pipeline.id,
        transformer_node.id,
        serde_json::json!({
            "x": 300,
            "y": 100
        }),
        "latest"
    )
    .fetch_one(db)
    .await?;

    // Insert into pipeline_nodes_connections (echo_to_transformer_pipeline_connection)
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

    // Insert into pipeline_nodes (post_to_fb_pipeline_node)
    let post_to_fb_pipeline_node = sqlx::query!(
        r#"
        INSERT INTO pipeline_nodes (pipeline_id, node_id, coords, node_version)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        pipeline.id,
        post_to_fb_node.id,
        serde_json::json!({
            "x": 500,
            "y": 100
        }),
        "latest"
    )
    .fetch_one(db)
    .await?;

    // Insert into pipeline_nodes_connections (transformer_to_post_to_fb_pipeline_connection)
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

    // Insert into pipeline_nodes (image_generator_pipeline_node)
    let image_generator_pipeline_node = sqlx::query!(
        r#"
            INSERT INTO pipeline_nodes (pipeline_id, node_id, coords, node_version)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        pipeline.id,
        image_generator_node.id,
        serde_json::json!({
            "x": 700,
            "y": 100
        }),
        "latest"
    )
    .fetch_one(db)
    .await?;

    // Insert into pipeline_nodes_connections (image_generator_to_post_to_fb_pipeline_connection)
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
