"use server";

import { env } from "~/env";
import { getSessionToken } from "~/lib/auth";

export interface UpdatePipelineTriggerParams {
  id: string;
  coords: { x: number; y: number };
}

export async function updatePipelineTriggerAction({
  id,
  coords,
}: UpdatePipelineTriggerParams) {
  const sessionToken = await getSessionToken();

  const response = await fetch(`${env.API_URL}/pipeline-triggers/${id}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${sessionToken}`,
    },
    body: JSON.stringify({ coords }),
  });

  if (!response.ok) {
    throw new Error(
      `Failed to update trigger: ${response.status} - ${await response.text()}`,
    );
  }
}
