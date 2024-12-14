-- Drop indexes
DROP INDEX IF EXISTS idx_pipeline_nodes_pipeline_id;
DROP INDEX IF EXISTS idx_pipeline_nodes_connections_from_node_id;
DROP INDEX IF EXISTS idx_pipeline_nodes_connections_to_node_id;

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
