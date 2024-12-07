DROP INDEX IF EXISTS idx_task_connections_to_task_id;
DROP INDEX IF EXISTS idx_task_connections_from_task_id;
DROP INDEX IF EXISTS idx_pipeline_tasks_pipeline_id;

DROP TABLE IF EXISTS task_connections;
DROP TABLE IF EXISTS pipeline_tasks;
DROP TABLE IF EXISTS pipeline_exec;
DROP TYPE IF EXISTS PipelineStatus;
DROP TABLE IF EXISTS tasks;
DROP TYPE IF EXISTS ContainerType;
DROP TABLE IF EXISTS pipelines;
