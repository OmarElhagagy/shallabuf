"use server";

import { revalidatePath } from "next/cache";
import { env } from "~/env";
import { getSessionToken } from "~/lib/auth";
import { createPipelineSchema } from "~/lib/schemas";

export async function createPipelineAction(
  _currentState: {
    errors: {
      teamId: string[] | undefined;
      name?: string[] | undefined;
      description?: string[] | undefined;
    };
  },
  formData: FormData,
) {
  const parsed = createPipelineSchema.safeParse({
    teamId: formData.get("teamId"),
    name: formData.get("name"),
    description: formData.get("description"),
  });

  if (!parsed.success) {
    return {
      errors: parsed.error.flatten().fieldErrors,
    };
  }

  const sessionToken = await getSessionToken();

  const response = await fetch(`${env.API_URL}/pipelines`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
      Authorization: `Bearer ${sessionToken}`,
    },
    body: JSON.stringify({
      teamId: parsed.data.teamId,
      name: parsed.data.name,
      description: parsed.data.description,
    }),
  });

  if (!response.ok) {
    console.error("Failed to create a pipeline", response);
    throw new Error("Failed to create a pipeline");
  }

  revalidatePath("/pipelines");
  return response.json();
}
