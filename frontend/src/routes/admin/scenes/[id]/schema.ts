import { z } from 'zod/v4';

export const sceneFormSchema = z.object({
	name: z.string().min(1, 'Name is required'),
	description: z.string().default('')
});

export const createPresetSchema = z.object({
	name: z.string().min(1, 'Name is required'),
	slug: z.string().min(1, 'Slug is required'),
	time_of_day_ticks: z.number().int().min(0).max(24000).default(6000),
	weather: z.enum(['clear', 'rain', 'thunder', 'snow']).default('clear'),
	weather_intensity: z.number().min(0).max(1).default(0),
	moon_phase: z.number().int().min(0).max(7).optional()
});

export const editPresetSchema = z.object({
	name: z.string().min(1, 'Name is required'),
	time_of_day_ticks: z.number().int().min(0).max(24000),
	weather: z.enum(['clear', 'rain', 'thunder', 'snow']),
	weather_intensity: z.number().min(0).max(1),
	moon_phase: z.number().int().min(0).max(7).optional()
});
