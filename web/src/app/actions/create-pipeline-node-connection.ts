"use server";

export interface CreatePipelineNodeConnectionParams {
	fromNodeId: string;
	toNodeId: string;
}

export async function createPipelineNodeConnectionAction(
	params: CreatePipelineNodeConnectionParams,
) {
	const response = await fetch(
		"http://192.168.0.2:8000/api/v0/pipeline_node_connections",
		{
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify(params),
		},
	);

	if (!response.ok) {
		throw new Error(
			`Failed to create node connection: ${response.status} - ${await response.text()}`,
		);
	}

	return response.json();
}
