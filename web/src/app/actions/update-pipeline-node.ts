"use server";

export interface UpdatePipelineNodeParams {
	id: string;
	coords: { x: number; y: number };
}

export async function updatePipelineNodeAction({
	id,
	coords,
}: UpdatePipelineNodeParams) {
	const response = await fetch(
		`http://192.168.0.2:8000/api/v0/pipeline_nodes/${id}`,
		{
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({ coords }),
		},
	);

	if (!response.ok) {
		throw new Error("Failed to update node");
	}
}
