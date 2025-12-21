// Backend-aligned API types
// These mirror the Rust backend models in backend/src/models/mod.rs

export interface World {
	id: string;
	slug: string;
	name: string;
	description: string | null;
	minecraft_version: string;
	file_url: string | null;
	file_hash: string | null;
	size_bytes: number | null;
	created_at: string;
	updated_at: string;
}

export interface Shader {
	id: string;
	name: string;
	slug: string;
	description: string | null;
	modrinth_id: string | null;
	curseforge_id: string | null;
	website_url: string | null;
	created_at: string;
	updated_at: string;
}

export interface ShaderVersion {
	id: string;
	shader_id: string;
	version: string;
	modrinth_version_id: string | null;
	download_url: string | null;
	file_hash: string | null;
	supported_profiles: string | null; // JSON array
	created_at: string;
}

export interface Scene {
	id: string;
	name: string;
	slug: string;
	description: string | null;
	world_id: string;
	x: number;
	y: number;
	z: number;
	pitch: number;
	yaw: number;
	dimension: string;
	time_of_day_ticks: number;
	weather: string;
	weather_intensity: number;
	moon_phase: number | null;
	biome: string | null;
	definition_json: string | null;
	tags: string | null; // JSON array
	created_at: string;
}

export interface Capture {
	id: string;
	shader_version_id: string;
	scene_id: string;
	profile: string | null;
	screenshot_url: string | null;
	screenshot_path: string | null;
	video_url: string | null;
	thumbnail_url: string | null;
	avg_fps: number | null;
	min_fps: number | null;
	max_fps: number | null;
	frame_time_avg: number | null;
	frame_time_p99: number | null;
	minecraft_version: string | null;
	iris_version: string | null;
	gpu_model: string | null;
	status: string;
	error_message: string | null;
	created_at: string;
	updated_at: string;
}

// API Response Types

export interface CaptureWithContext {
	id: string;
	scene_id: string;
	shader_slug: string;
	shader_name: string;
	shader_version: string;
	profile: string | null;
	screenshot_path: string | null;
	screenshot_url: string | null;
	captured_at: string | null;
	resolution_width: number | null;
	resolution_height: number | null;
}

export interface ShaderWithCaptures extends Shader {
	versions: ShaderVersion[];
	captures: CaptureWithContext[];
}

export interface SceneWithCaptures extends Scene {
	world: World | null;
	captures: CaptureWithContext[];
}
