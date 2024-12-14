interface Pipeline {
	id: string;
	name: string;
	description?: string;
	nodes: PipelineNode[];
	connections: PipelineConnection[];
}

interface PipelineNode {
	id: string;
	node_id: string;
	node_version: string;
	trigger_id?: string;
	coords: {
		x: number;
		y: number;
	};
}

interface PipelineConnection {
	id: string;
	from_node_id: string;
	to_node_id: string;
}
