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
    coords JSONB NOT NULL,
    config JSON NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pipeline_id) REFERENCES pipelines(id) ON DELETE CASCADE
);

-- Create 'nodes' table
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    identifier_name VARCHAR NOT NULL,
    publisher_name VARCHAR NOT NULL,
    version VARCHAR NOT NULL DEFAULT 'v1',
    description VARCHAR,
    config JSON NOT NULL,
    container_type node_container_type NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    visibility visibility NOT NULL DEFAULT 'public',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (identifier_name, publisher_name, version)
);

-- Create 'pipeline_execs' table
CREATE TABLE IF NOT EXISTS pipeline_execs (
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

-- Create 'pipeline_node_execs' table
CREATE TABLE IF NOT EXISTS pipeline_node_execs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_execs_id UUID NOT NULL,
    pipeline_node_id UUID NOT NULL,
    status exec_status NOT NULL DEFAULT 'pending',
    result JSON,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    finished_at TIMESTAMP,
    FOREIGN KEY (pipeline_execs_id) REFERENCES pipeline_execs(id) ON DELETE CASCADE,
    FOREIGN KEY (pipeline_node_id) REFERENCES pipeline_nodes(id) ON DELETE CASCADE
);

-- Create 'pipeline_node_outputs' table
CREATE TABLE IF NOT EXISTS pipeline_node_outputs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR NOT NULL,
    pipeline_node_id UUID NOT NULL REFERENCES pipeline_nodes(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create 'pipeline_node_inputs' table
CREATE TABLE IF NOT EXISTS pipeline_node_inputs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR NOT NULL,
    pipeline_node_id UUID NOT NULL REFERENCES pipeline_nodes(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create 'pipeline_node_connections' table
CREATE TABLE IF NOT EXISTS pipeline_node_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_pipeline_node_output_id UUID NOT NULL REFERENCES pipeline_node_outputs(id) ON DELETE CASCADE,
    to_pipeline_node_input_id UUID NOT NULL REFERENCES pipeline_node_inputs(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_pipeline_triggers_pipeline_id
    ON pipeline_triggers(pipeline_id);

CREATE INDEX IF NOT EXISTS idx_pipeline_node_connections_to_pipeline_node_input_id
    ON pipeline_node_connections(to_pipeline_node_input_id);

CREATE INDEX IF NOT EXISTS idx_pipeline_node_connections_from_pipeline_node_output_id
    ON pipeline_node_connections(from_pipeline_node_output_id);

CREATE INDEX IF NOT EXISTS idx_pipeline_node_connections_from_to
    ON pipeline_node_connections(to_pipeline_node_input_id, from_pipeline_node_output_id);

CREATE INDEX IF NOT EXISTS idx_pipeline_nodes_pipeline_id
    ON pipeline_nodes(pipeline_id);

CREATE INDEX IF NOT EXISTS idx_nodes_name
    ON nodes(identifier_name, publisher_name, version);

CREATE INDEX IF NOT EXISTS idx_pipeline_node_inputs_pipeline_node_id
    ON pipeline_node_inputs(pipeline_node_id);

CREATE INDEX IF NOT EXISTS idx_pipeline_node_outputs_pipeline_node_id
    ON pipeline_node_outputs(pipeline_node_id);

CREATE INDEX IF NOT EXISTS idx_pipeline_execs_pipeline_id
    ON pipeline_execs(pipeline_id);

CREATE INDEX IF NOT EXISTS idx_pipeline_node_execs_pipeline_node_id
    ON pipeline_node_execs(pipeline_node_id);

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

CREATE TRIGGER set_updated_at_pipeline_nodes
BEFORE UPDATE ON pipeline_nodes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_pipeline_node_connections
BEFORE UPDATE ON pipeline_node_connections
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_pipeline_node_inputs
BEFORE UPDATE ON pipeline_node_inputs
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER set_updated_at_pipeline_node_outputs
BEFORE UPDATE ON pipeline_node_outputs
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();