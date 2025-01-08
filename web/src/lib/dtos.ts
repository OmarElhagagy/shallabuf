export interface Node {
	id: string;
	name: string;
	description?: string;
	config: TaskNodeConfig;
}

export interface PipelineParticipant {
	id: string;
	name: string;
}

export interface Pipeline {
	id: string;
	name: string;
	description?: string;
	trigger: {
		id: string;
		config: {
			[key: string]: string;
		};
	};
	nodes: PipelineNode[];
	connections: PipelineNodeConnection[];
	participants?: PipelineParticipant[];
}

export interface PipelineNode {
	id: string;
	nodeId: string;
	nodeVersion: string;
	triggerId?: string;
	inputs: { id: string; key: string }[];
	outputs: { id: string; key: string }[];
	coords: {
		x: number;
		y: number;
	};
}

export interface PipelineNodeConnection {
	id: string;
	toPipelineNodeInputId: string;
	fromPipelineNodeOutputId: string;
}

export enum NodeType {
	Task = "task",
	Trigger = "trigger",
}

export interface TaskNodeConfigV0InputText {
	text: {
		default?: string;
	};
}

export interface TaskNodeConfigV0InputSelect {
	select: {
		options: {
			value: string;
			label: {
				[key: string]: string;
			};
		}[];
		default?: string;
	};
}

export type TaskNodeConfigV0Input =
	| TaskNodeConfigV0InputText
	| TaskNodeConfigV0InputSelect
	| "binary";

export const isTaskNodeConfigV0InputText = (
	input: TaskNodeConfigV0Input,
): input is TaskNodeConfigV0InputText => {
	return (input as TaskNodeConfigV0InputText).text !== undefined;
};

export const isTaskNodeConfigV0InputSelect = (
	input: TaskNodeConfigV0Input,
): input is TaskNodeConfigV0InputSelect => {
	return (input as TaskNodeConfigV0InputSelect).select !== undefined;
};

export const isTaskNodeConfigV0InputBinary = (
	input: TaskNodeConfigV0Input,
): input is "binary" => {
	return input === "binary";
};

export type TaskNodeConfigV0Output = "text" | "status" | "binary";

export interface TaskNodeConfigV0 {
	version: "v0";
	inputs: {
		key: string;
		input: TaskNodeConfigV0Input;
		label: {
			[key: string]: string;
		};
		description?: string;
		required: boolean;
	}[];
	outputs: {
		key: string;
		output: TaskNodeConfigV0Output;
		label: {
			[key: string]: string;
		};
	}[];
}

export type TaskNodeConfig = TaskNodeConfigV0;
