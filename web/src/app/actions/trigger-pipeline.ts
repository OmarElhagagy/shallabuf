"use server";
import { getSessionToken } from "~/lib/auth";

export async function triggerPipelineAction(pipelineId: string) {
	const sessionToken = await getSessionToken();

	const response = await fetch(
		`http://localhost:8000/api/v0/trigger/pipelines/${pipelineId}`,
		{
			method: "POST",
			headers: {
				Accept: "application/json",
				"Content-Type": "application/json",
				Authorization: `Bearer ${sessionToken}`,
			},
			body: JSON.stringify({}),
		},
	);

	if (!response.ok) {
		throw new Error("Failed to trigger pipeline");
	}

	return response.json();
}
