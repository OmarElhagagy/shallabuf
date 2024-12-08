use std::collections::HashMap;

use crate::dtos::{
    NodeConfig, NodeConfigV0, NodeInput, NodeInputType, NodeOutputType, PipelineTriggerConfig,
    PipelineTriggerConfigV0, SelectInput,
};
use sea_orm::DatabaseConnection;
use sea_orm::{ActiveValue::Set, EntityTrait};

use crate::entities::{
    nodes, organizations, pipeline_nodes, pipeline_nodes_connections, pipeline_triggers, pipelines,
    sea_orm_active_enums::NodeContainerType, teams,
};

/// Seeds the database with initial data.
///
/// # Panics
///
/// This function panics if the database connection fails
/// or if a json config serialization fails.
#[allow(clippy::too_many_lines)]
pub async fn seed_database(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let organization = organizations::Entity::insert(organizations::ActiveModel {
        name: Set("Organization 1".to_string()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let team = teams::Entity::insert(teams::ActiveModel {
        name: Set("Team 1".to_string()),
        organization_id: Set(organization.last_insert_id),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let pipeline = pipelines::Entity::insert(pipelines::ActiveModel {
        name: Set("Pipeline 1".to_string()),
        description: Set(Some("Description 1".to_string())),
        team_id: Set(team.last_insert_id),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let pipeline_trigger = pipeline_triggers::Entity::insert(pipeline_triggers::ActiveModel {
        pipeline_id: Set(pipeline.last_insert_id),
        config: Set(
            serde_json::to_value(PipelineTriggerConfig::V0(PipelineTriggerConfigV0 {
                allow_manual_execution: true,
            }))
            .unwrap(),
        ),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let echo_node = nodes::Entity::insert(nodes::ActiveModel {
        name: Set("Echo".to_string()),
        description: Set(Some(
            "A simple node that echoes the message it receives.".to_string(),
        )),
        publisher_name: Set("shallabuf".to_string()),
        container_type: Set(NodeContainerType::Wasm),
        config: Set(serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
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
        .unwrap()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let transformer_node = nodes::Entity::insert(nodes::ActiveModel {
        name: Set("Transformer".to_string()),
        description: Set(Some(
            "A simple node that transforms the message it receives.".to_string(),
        )),
        publisher_name: Set("shallabuf".to_string()),
        container_type: Set(NodeContainerType::Wasm),
        config: Set(serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
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
        .unwrap()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let image_generator_node = nodes::Entity::insert(nodes::ActiveModel {
        name: Set("Image Generator".to_string()),
        description: Set(Some("A simple node that generates an image.".to_string())),
        publisher_name: Set("shallabuf".to_string()),
        container_type: Set(NodeContainerType::Wasm),
        config: Set(serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
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
        .unwrap()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let post_to_fb_node = nodes::Entity::insert(nodes::ActiveModel {
        name: Set("Post to Facebook".to_string()),
        description: Set(Some("A simple node that posts to Facebook.".to_string())),
        container_type: Set(NodeContainerType::Wasm),
        publisher_name: Set("shallabuf".to_string()),
        config: Set(serde_json::to_value(NodeConfig::V0(NodeConfigV0 {
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
        .unwrap()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let echo_pipeline_node = pipeline_nodes::Entity::insert(pipeline_nodes::ActiveModel {
        pipeline_id: Set(pipeline.last_insert_id),
        node_id: Set(echo_node.last_insert_id),
        trigger_id: Set(Some(pipeline_trigger.last_insert_id)),
        node_version: Set("latest".to_string()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let transformer_pipeline_node = pipeline_nodes::Entity::insert(pipeline_nodes::ActiveModel {
        pipeline_id: Set(pipeline.last_insert_id),
        node_id: Set(transformer_node.last_insert_id),
        node_version: Set("latest".to_string()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let _echo_to_transformer_pipeline_connection =
        pipeline_nodes_connections::Entity::insert(pipeline_nodes_connections::ActiveModel {
            from_node_id: Set(echo_pipeline_node.last_insert_id),
            to_node_id: Set(transformer_pipeline_node.last_insert_id),
            ..Default::default()
        })
        .exec(db)
        .await?;

    let image_generator_pipeline_node =
        pipeline_nodes::Entity::insert(pipeline_nodes::ActiveModel {
            pipeline_id: Set(pipeline.last_insert_id),
            node_id: Set(image_generator_node.last_insert_id),
            node_version: Set("latest".to_string()),
            ..Default::default()
        })
        .exec(db)
        .await?;

    let _transformer_to_image_generator_pipeline_connection =
        pipeline_nodes_connections::Entity::insert(pipeline_nodes_connections::ActiveModel {
            from_node_id: Set(transformer_pipeline_node.last_insert_id),
            to_node_id: Set(image_generator_pipeline_node.last_insert_id),
            ..Default::default()
        })
        .exec(db)
        .await?;

    let post_to_fb_pipeline_node = pipeline_nodes::Entity::insert(pipeline_nodes::ActiveModel {
        pipeline_id: Set(pipeline.last_insert_id),
        node_id: Set(post_to_fb_node.last_insert_id),
        node_version: Set("latest".to_string()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    let _image_generator_to_post_to_fb_pipeline_connection =
        pipeline_nodes_connections::Entity::insert(pipeline_nodes_connections::ActiveModel {
            from_node_id: Set(image_generator_pipeline_node.last_insert_id),
            to_node_id: Set(post_to_fb_pipeline_node.last_insert_id),
            ..Default::default()
        })
        .exec(db)
        .await?;

    Ok(())
}
