"use server";

import { env } from "~/env";

export interface CreatePipelineNodeParams {
	pipelineId: string;
	nodeId: string;
	nodeVersion: string;
	coords: { x: number; y: number };
}

export async function createPipelineNodeAction(
	params: CreatePipelineNodeParams,
) {
	const response = await fetch(`${env.API_URL}/pipeline_nodes`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(params),
	});

	if (!response.ok) {
		throw new Error(
			`Failed to create node: ${response.status} - ${await response.text()}`,
		);
	}

	return response.json();
}
