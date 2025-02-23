"use server";

import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { env } from "~/env";
import { getSessionToken } from "~/lib/auth";

export async function logoutAction() {
  const sessionToken = await getSessionToken();

  const response = await fetch(`${env.API_URL}/auth/logout`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
      Authorization: `Bearer ${sessionToken}`,
    },
  });

  if (!response.ok) {
    throw new Error("Failed to login");
  }

  (await cookies()).delete("session");

  redirect("/auth/login");
}
