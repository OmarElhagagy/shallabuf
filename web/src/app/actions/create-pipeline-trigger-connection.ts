"use server";

import { env } from "~/env";
import { getSessionToken } from "~/lib/auth";

export interface CreatePipelineTriggerConnectionParams {
	triggerId: string;
	nodeId: string;
}

export async function createPipelineTriggerConnectionAction(
	params: CreatePipelineTriggerConnectionParams,
) {
	const sessionToken = getSessionToken();

	const response = await fetch(
		`${env.API_URL}/pipeline-nodes/${params.nodeId}`,
		{
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Accept: "application/json",
				Authorization: `Bearer ${sessionToken}`,
			},
			body: JSON.stringify({
				triggerId: params.triggerId,
			}),
		},
	);

	if (!response.ok) {
		throw new Error(
			`Failed to create node connection: ${response.status} - ${await response.text()}`,
		);
	}

	return response.json();
}
