CREATE OR REPLACE FUNCTION notify_pipeline_execs_event()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('pipeline_execs_events', row_to_json(NEW)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER pipeline_execs_trigger
AFTER INSERT OR UPDATE ON pipeline_execs
FOR EACH ROW
EXECUTE FUNCTION notify_pipeline_execs_event();

CREATE TRIGGER pipeline_node_execs_trigger
AFTER INSERT OR UPDATE ON pipeline_node_execs
FOR EACH ROW
EXECUTE FUNCTION notify_pipeline_execs_event();
