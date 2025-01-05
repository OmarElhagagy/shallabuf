"use server";

import { env } from "~/env";

export interface UpdatePipelineNodeParams {
	id: string;
	coords: { x: number; y: number };
}

export async function updatePipelineNodeAction({
	id,
	coords,
}: UpdatePipelineNodeParams) {
	const response = await fetch(`${env.API_URL}/pipeline_nodes/${id}`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({ coords }),
	});

	if (!response.ok) {
		throw new Error(
			`Failed to update node: ${response.status} - ${await response.text()}`,
		);
	}
}
