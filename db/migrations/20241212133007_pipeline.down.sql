-- Drop indexes
DROP INDEX IF EXISTS idx_pipeline_nodes_pipeline_id;
DROP INDEX IF EXISTS idx_pipeline_nodes_connections_from_node_id;
DROP INDEX IF EXISTS idx_pipeline_nodes_connections_to_node_id;

-- Drop triggers
DROP TRIGGER IF EXISTS set_updated_at_templates ON templates;
DROP TRIGGER IF EXISTS set_updated_at_pipelines ON pipelines;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_triggers ON pipeline_triggers;
DROP TRIGGER IF EXISTS set_updated_at_nodes ON nodes;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_exec ON pipeline_exec;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_nodes ON pipeline_nodes;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_nodes_exec ON pipeline_nodes_exec;
DROP TRIGGER IF EXISTS set_updated_at_pipeline_nodes_connections ON pipeline_nodes_connections;

-- Drop tables
DROP TABLE IF EXISTS pipeline_nodes_connections CASCADE;
DROP TABLE IF EXISTS pipeline_nodes_exec CASCADE;
DROP TABLE IF EXISTS pipeline_nodes CASCADE;
DROP TABLE IF EXISTS pipeline_exec CASCADE;
DROP TABLE IF EXISTS nodes CASCADE;
DROP TABLE IF EXISTS pipeline_triggers CASCADE;
DROP TABLE IF EXISTS pipelines CASCADE;
DROP TABLE IF EXISTS templates CASCADE;

-- Drop enums
DROP TYPE IF EXISTS node_container_type;
DROP TYPE IF EXISTS exec_status;
DROP TYPE IF EXISTS visibility;
