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

CREATE OR REPLACE FUNCTION cancel_descendant_pending_node_execs()
RETURNS trigger AS $$
BEGIN
    WITH RECURSIVE descendants AS (
        SELECT pn.id
        FROM pipeline_nodes pn
        WHERE pn.id = NEW.pipeline_node_id
        UNION
        SELECT child_pn.id
        FROM descendants d
        JOIN pipeline_node_outputs pno ON pno.pipeline_node_id = d.id
        JOIN pipeline_node_connections pnc ON pnc.from_pipeline_node_output_id = pno.id
        JOIN pipeline_node_inputs pni ON pni.id = pnc.to_pipeline_node_input_id
        JOIN pipeline_nodes child_pn ON child_pn.id = pni.pipeline_node_id
    )
    UPDATE pipeline_node_execs
    SET status = 'cancelled'
    WHERE pipeline_exec_id = NEW.pipeline_exec_id
      AND status = 'pending'
      AND pipeline_node_id IN (SELECT id FROM descendants)
      AND pipeline_node_id <> NEW.pipeline_node_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER cancel_descendant_pending_node_execs_trigger
AFTER UPDATE ON pipeline_node_execs
FOR EACH ROW
WHEN (NEW.status = 'failed')
EXECUTE FUNCTION cancel_descendant_pending_node_execs();
