"use server";
import { env } from "~/env";
import { getSessionToken } from "~/lib/auth";

export async function triggerPipelineAction(formData: FormData) {
	const pipelineId = formData.get("pipelineId") as string;

	const sessionToken = await getSessionToken();

	const response = await fetch(
		`${env.API_URL}/trigger/pipelines/${pipelineId}`,
		{
			method: "POST",
			headers: {
				Accept: "application/json",
				"Content-Type": "application/json",
				Authorization: `Bearer ${sessionToken}`,
			},
			body: formData.get("inputs"),
		},
	);

	if (!response.ok) {
		throw new Error("Failed to trigger pipeline");
	}

	return response.json();
}
