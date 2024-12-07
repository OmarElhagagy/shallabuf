CREATE TABLE pipelines (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    name VARCHAR NOT NULL,
    description VARCHAR,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE TYPE ContainerType AS ENUM ('wasm', 'docker');

CREATE TABLE tasks (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    params_def JSONB NOT NULL,
    config JSONB NOT NULL,
    container_type ContainerType NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE TYPE PipelineStatus AS ENUM ('pending', 'running', 'completed', 'failed');

CREATE TABLE pipeline_exec (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    pipeline_id UUID REFERENCES pipelines(id) ON DELETE CASCADE,
    status PipelineStatus NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);

CREATE TABLE pipeline_tasks (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    pipeline_id UUID REFERENCES pipelines(id) ON DELETE CASCADE,
    task_id UUID REFERENCES tasks(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_pipeline_tasks_pipeline_id ON pipeline_tasks(pipeline_id);

CREATE TABLE task_connections (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    from_task_id UUID REFERENCES pipeline_tasks(id) ON DELETE CASCADE,
    to_task_id UUID REFERENCES pipeline_tasks(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX idx_task_connections_from_task_id ON task_connections(from_task_id);
CREATE INDEX idx_task_connections_to_task_id ON task_connections(to_task_id);
