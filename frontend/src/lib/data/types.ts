// Core types for the Glint shader preview catalog

export type PerformanceTier = 'potato' | 'low' | 'medium' | 'high' | 'ultra';
export type ShaderStyle = 'realistic' | 'fantasy' | 'vibrant' | 'minimal' | 'retro' | 'cinematic';
export type TimeOfDay =
	| 'dawn'
	| 'morning'
	| 'noon'
	| 'afternoon'
	| 'sunset'
	| 'dusk'
	| 'night'
	| 'midnight';
export type Weather = 'clear' | 'cloudy' | 'rain' | 'storm' | 'snow' | 'fog';
export type Dimension = 'overworld' | 'nether' | 'end';

// A shader pack with its metadata
export interface Shader {
	id: string;
	name: string;
	slug: string;
	author: string;
	authorUrl: string;
	description: string;
	longDescription: string;
	thumbnail: string;
	downloadCount: number;
	likes: number;
	style: ShaderStyle;
	tier: PerformanceTier;
	features: ShaderFeature[];
	version: string;
	mcVersions: string[];
	lastUpdated: string;
	modrinthUrl?: string;
	curseforgeUrl?: string;
	websiteUrl?: string;
}

// Shader features with categories
export interface ShaderFeature {
	name: string;
	category: 'lighting' | 'effects' | 'water' | 'atmosphere' | 'performance';
}

// A standardized test scene
export interface Scene {
	id: string;
	name: string;
	slug: string;
	description: string;
	thumbnail: string;
	dimension: Dimension;
	biome: string;
	defaultTime: TimeOfDay;
	defaultWeather: Weather;
	features: string[];
	complexity: 'simple' | 'moderate' | 'complex';
}

// A single capture: one shader + one scene + conditions
export interface Capture {
	id: string;
	shaderId: string;
	sceneId: string;
	image: string;
	timeOfDay: TimeOfDay;
	weather: Weather;
	fps: number;
	frameTimeMs: number;
	gpuUsage: number;
	vramMb: number;
	capturedAt: string;
}

// Aggregated performance metrics for a shader across all scenes
export interface ShaderPerformance {
	shaderId: string;
	avgFps: number;
	minFps: number;
	maxFps: number;
	avgFrameTime: number;
	avgGpuUsage: number;
	avgVram: number;
	sceneCount: number;
}

// Performance data for a scene across all shaders
export interface ScenePerformance {
	sceneId: string;
	avgFps: number;
	minFps: number;
	maxFps: number;
	shaderCount: number;
	hardestShader: string;
	easiestShader: string;
}
