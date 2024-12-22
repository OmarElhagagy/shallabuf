-- Drop indexes
DROP INDEX IF EXISTS idx_pipeline_nodes_pipeline_id;
DROP INDEX IF EXISTS idx_pipeline_node_connections_to_pipeline_node_input_id;
DROP INDEX IF EXISTS idx_pipeline_node_connections_from_pipeline_node_output_id;
DROP INDEX IF EXISTS idx_pipeline_node_connections_from_to;
DROP INDEX IF EXISTS idx_nodes_name;
DROP INDEX IF EXISTS idx_pipeline_node_inputs_pipeline_node_id;
DROP INDEX IF EXISTS idx_pipeline_node_outputs_pipeline_node_id;
DROP INDEX IF EXISTS idx_pipeline_execs_pipeline_id;
DROP INDEX IF EXISTS idx_pipeline_node_execs_pipeline_node_id;

-- Drop triggers
DROP TRIGGER IF EXISTS set_updated_at_templates ON templates;
DROP TRIGGER IF EXISTS set_updated_at_pipelines ON pipelines;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_triggers ON pipeline_triggers;
DROP TRIGGER IF EXISTS set_updated_at_nodes ON nodes;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_nodes ON pipeline_nodes;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_node_connections ON pipeline_node_connections;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_node_outputs ON pipeline_node_outputs;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_node_inputs ON pipeline_node_inputs;

-- Drop tables
DROP TABLE IF EXISTS pipeline_node_connections CASCADE;
DROP TABLE IF EXISTS pipeline_node_execs CASCADE;
DROP TABLE IF EXISTS pipeline_nodes CASCADE;
DROP TABLE IF EXISTS pipeline_execs CASCADE;
DROP TABLE IF EXISTS nodes CASCADE;
DROP TABLE IF EXISTS pipeline_triggers CASCADE;
DROP TABLE IF EXISTS pipelines CASCADE;
DROP TABLE IF EXISTS templates CASCADE;
DROP TABLE IF EXISTS pipeline_node_outputs CASCADE;
DROP TABLE IF EXISTS pipeline_node_inputs CASCADE;

-- Drop enums
DROP TYPE IF EXISTS node_container_type;
DROP TYPE IF EXISTS exec_status;
DROP TYPE IF EXISTS visibility;
