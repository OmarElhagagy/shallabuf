import type { Participant } from "~/app/pipelines/[id]/_components/editor";

export interface Pipeline {
	id: string;
	name: string;
	description?: string;
	nodes: PipelineNode[];
	connections: PipelineConnection[];
	participants: Participant[];
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
