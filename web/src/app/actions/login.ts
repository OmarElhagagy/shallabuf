"use server";

import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { loginSchema } from "~/lib/schemas";

export interface LoginParams {
	email: string;
	password: string;
}

export interface Session {
	token: string;
	expiresAt: string;
}

export async function loginAction(
	_currentState: {
		errors: {
			email?: string[] | undefined;
			password?: string[] | undefined;
		};
	},
	formData: FormData,
) {
	const parsed = loginSchema.safeParse({
		email: formData.get("email"),
		password: formData.get("password"),
	});

	if (!parsed.success) {
		return {
			errors: parsed.error.flatten().fieldErrors,
		};
	}

	const response = await fetch("http://192.168.0.2:8000/api/v0/auth/login", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			email: parsed.data.email,
			password: parsed.data.password,
		}),
	});

	if (!response.ok) {
		throw new Error("Failed to login");
	}

	const session: Session = await response.json();

	(await cookies()).set("session", session.token, {
		httpOnly: true,
		path: "/",
		secure: process.env.NODE_ENV === "production",
		sameSite: "lax",
		expires: new Date(session.expiresAt),
	});

	redirect("/pipelines");
}
