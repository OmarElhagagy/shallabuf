CREATE OR REPLACE FUNCTION notify_pipeline_exec_event()
RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('pipeline_exec_events', row_to_json(NEW)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER pipeline_exec_trigger
AFTER INSERT OR UPDATE ON pipeline_exec
FOR EACH ROW
EXECUTE FUNCTION notify_pipeline_exec_event();
