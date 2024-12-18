-- Create 'visibility' enum type
CREATE TYPE visibility AS ENUM ('public', 'private');

-- Create 'exec_status' enum type
CREATE TYPE exec_status AS ENUM ('pending', 'running', 'completed', 'failed');

-- Create 'node_container_type' enum type
CREATE TYPE node_container_type AS ENUM ('wasm', 'docker');

-- Create 'templates' table
CREATE TABLE IF NOT EXISTS templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    config JSON NOT NULL,
    visibility visibility NOT NULL DEFAULT 'public',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create 'pipelines' table
CREATE TABLE IF NOT EXISTS pipelines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description VARCHAR,
    from_template_id UUID,
    team_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (from_template_id) REFERENCES templates(id) ON DELETE SET NULL,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE RESTRICT
);

-- Create 'pipeline_triggers' table
CREATE TABLE IF NOT EXISTS pipeline_triggers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_id UUID NOT NULL,
    config JSON NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pipeline_id) REFERENCES pipelines(id) ON DELETE CASCADE
);

-- Create 'nodes' table
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    publisher_name VARCHAR NOT NULL,
    description VARCHAR,
    config JSON NOT NULL,
    container_type node_container_type NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    versions TEXT[] NOT NULL DEFAULT '{"latest"}',
    visibility visibility NOT NULL DEFAULT 'public',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create 'pipeline_exec' table
CREATE TABLE IF NOT EXISTS pipeline_exec (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_id UUID NOT NULL,
    status exec_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    finished_at TIMESTAMP,
    FOREIGN KEY (pipeline_id) REFERENCES pipelines(id) ON DELETE CASCADE
);

-- Create 'pipeline_nodes' table
CREATE TABLE IF NOT EXISTS pipeline_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_id UUID NOT NULL,
    node_id UUID NOT NULL,
    node_version VARCHAR NOT NULL,
    coords JSONB NOT NULL,
    trigger_id UUID,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pipeline_id) REFERENCES pipelines(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (trigger_id) REFERENCES pipeline_triggers(id) ON DELETE SET NULL
);

-- Create 'pipeline_nodes_exec' table
CREATE TABLE IF NOT EXISTS pipeline_nodes_exec (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_exec_id UUID NOT NULL,
    pipeline_node_id UUID NOT NULL,
    status exec_status NOT NULL DEFAULT 'pending',
    result JSON,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    finished_at TIMESTAMP,
    FOREIGN KEY (pipeline_exec_id) REFERENCES pipeline_exec(id) ON DELETE CASCADE,
    FOREIGN KEY (pipeline_node_id) REFERENCES pipeline_nodes(id) ON DELETE CASCADE
);

-- Create 'pipeline_nodes_connections' table
CREATE TABLE IF NOT EXISTS pipeline_nodes_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_node_id UUID NOT NULL,
    to_node_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (from_node_id) REFERENCES pipeline_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (to_node_id) REFERENCES pipeline_nodes(id) ON DELETE CASCADE
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_pipeline_nodes_connections_from_node_id
    ON pipeline_nodes_connections(from_node_id);

CREATE INDEX IF NOT EXISTS idx_pipeline_nodes_connections_to_node_id
    ON pipeline_nodes_connections(to_node_id);

CREATE INDEX IF NOT EXISTS idx_pipeline_nodes_pipeline_id
    ON pipeline_nodes(pipeline_id);

-- Create triggers
CREATE TRIGGER set_updated_at_templates
BEFORE UPDATE ON templates
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_pipelines
BEFORE UPDATE ON pipelines
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_pipeline_triggers
BEFORE UPDATE ON pipeline_triggers
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_nodes
BEFORE UPDATE ON nodes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_pipeline_exec
BEFORE UPDATE ON pipeline_exec
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_pipeline_nodes
BEFORE UPDATE ON pipeline_nodes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_pipeline_nodes_exec
BEFORE UPDATE ON pipeline_nodes_exec
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_pipeline_nodes_connections
BEFORE UPDATE ON pipeline_nodes_connections
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
