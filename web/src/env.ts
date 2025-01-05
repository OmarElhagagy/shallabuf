import { createEnv } from "@t3-oss/env-nextjs";
import { z } from "zod";

export const env = createEnv({
	/*
	 * Server-side Environment variables, not available on the client.
	 * Will throw if you access these variables on the client.
	 */
	server: {
		API_URL: z.string().url(),
	},
	/*
	 * Environment variables available on the client (and server).
	 *
	 * 💡 You'll get type errors if these are not prefixed with NEXT_PUBLIC_.
	 */
	client: {},
	/*
	 * Due to how Next.js bundles environment variables on Edge and Client,
	 * we need to manually destructure them to make sure all are included in bundle.
	 *
	 * 💡 You'll get type errors if not all variables from `server` & `client` are included here.
	 * If you're using Next.js < 13.4.4, you'll need to specify the runtimeEnv manually
	 * runtimeEnv: {
	 *   DATABASE_URL: process.env.DATABASE_URL,
	 *   OPEN_AI_API_KEY: process.env.OPEN_AI_API_KEY,
	 * },
	 * For Next.js >= 13.4.4, you can just reference process.env:
	 */
	experimental__runtimeEnv: process.env,
});
