"use server";

import { env } from "~/env";
import { getSessionToken } from "~/lib/auth";

export interface CreatePipelineNodeConnectionParams {
  fromNodeId: string;
  toNodeId: string;
}

export async function createPipelineNodeConnectionAction(
  params: CreatePipelineNodeConnectionParams,
) {
  const sessionToken = getSessionToken();

  const response = await fetch(`${env.API_URL}/pipeline-node-connections`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
      Authorization: `Bearer ${sessionToken}`,
    },
    body: JSON.stringify(params),
  });

  if (!response.ok) {
    throw new Error(
      `Failed to create node connection: ${response.status} - ${await response.text()}`,
    );
  }

  return response.json();
}
