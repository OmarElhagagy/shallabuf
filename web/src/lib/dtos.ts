export interface PipelineParticipant {
	id: string;
	name: string;
}

export interface Pipeline {
	id: string;
	name: string;
	description?: string;
	nodes: PipelineNode[];
	connections: PipelineConnection[];
	participants: PipelineParticipant[];
}

export interface PipelineNode {
	id: string;
	node_id: string;
	node_version: string;
	trigger_id?: string;
	coords: {
		x: number;
		y: number;
	};
}

export interface PipelineConnection {
	id: string;
	from_node_id: string;
	to_node_id: string;
}
