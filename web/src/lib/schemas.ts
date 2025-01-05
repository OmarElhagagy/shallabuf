import { z } from "zod";

export const loginSchema = z.object({
	email: z.string().email(),
	password: z.string().nonempty("Password is required"),
});

export const createPipelineSchema = z.object({
	teamId: z.string().nonempty("Team ID is required"),
	name: z.string().nonempty("Name is required"),
	description: z.string().optional(),
});
