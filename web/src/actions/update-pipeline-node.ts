"use server";

import { env } from "~/env";
import { getSessionToken } from "~/lib/auth";

export interface UpdatePipelineNodeParams {
  id: string;
  coords: { x: number; y: number };
}

export async function updatePipelineNodeAction({
  id,
  coords,
}: UpdatePipelineNodeParams) {
  const sessionToken = await getSessionToken();

  const response = await fetch(`${env.API_URL}/pipeline-nodes/${id}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${sessionToken}`,
    },
    body: JSON.stringify({ coords }),
  });

  if (!response.ok) {
    throw new Error(
      `Failed to update node: ${response.status} - ${await response.text()}`,
    );
  }
}
