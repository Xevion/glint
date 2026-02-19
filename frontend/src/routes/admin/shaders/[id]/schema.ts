import { z } from 'zod/v4';

export const shaderFormSchema = z.object({
	name: z.string().min(1, 'Name is required'),
	description: z.string().default(''),
	modrinth_id: z.string().default(''),
	curseforge_id: z.string().default(''),
	website_url: z.union([z.string().url('Must be a valid URL'), z.literal('')]).default(''),
	capture_enabled: z.boolean().default(true),
	preferred_version_id: z.string().nullable().default(null)
});

export type ShaderFormSchema = typeof shaderFormSchema;
