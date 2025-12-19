// Mock data for Glint - Shader Preview Catalog
// Simulates the capture database that would come from the backend

import type {
	Shader,
	Scene,
	Capture,
	ShaderPerformance,
	ScenePerformance,
	PerformanceTier,
	ShaderStyle,
	TimeOfDay,
	Weather,
	Dimension
} from './types';

// Re-export types for convenience
export type {
	Shader,
	Scene,
	Capture,
	ShaderPerformance,
	ScenePerformance,
	PerformanceTier,
	ShaderStyle,
	TimeOfDay,
	Weather,
	Dimension
} from './types';

const WALLPAPER_COUNT = 112;

// Seeded random for consistent results per item
function seededRandom(seed: number): () => number {
	return () => {
		seed = (seed * 1103515245 + 12345) & 0x7fffffff;
		return seed / 0x7fffffff;
	};
}

export function getRandomWallpaper(seed: number): string {
	const rng = seededRandom(seed);
	const index = Math.floor(rng() * WALLPAPER_COUNT);
	return `/wallpapers/${index}.jpg`;
}

// Raw shader data (without computed fields)
const SHADERS_DATA: Omit<Shader, 'thumbnail' | 'downloadCount' | 'likes' | 'lastUpdated'>[] = [
	{
		id: 'complementary-reimagined',
		name: 'Complementary Reimagined',
		slug: 'complementary-reimagined',
		author: 'EminGT',
		authorUrl: 'https://modrinth.com/user/EminGT',
		description: 'A complete visual overhaul balancing performance and beauty',
		longDescription:
			'Complementary Reimagined brings stunning visuals without sacrificing playability. Features advanced lighting, volumetric effects, and beautiful water rendering while maintaining excellent performance across a wide range of hardware.',
		style: 'realistic',
		tier: 'medium',
		features: [
			{ name: 'Volumetric Lighting', category: 'lighting' },
			{ name: 'PBR Support', category: 'lighting' },
			{ name: 'TAA', category: 'performance' },
			{ name: 'Bloom', category: 'effects' },
			{ name: 'Reflective Water', category: 'water' },
			{ name: 'Waving Foliage', category: 'effects' },
			{ name: 'Custom Sky', category: 'atmosphere' }
		],
		version: '5.2.1',
		mcVersions: ['1.20.4', '1.21', '1.21.1'],
		modrinthUrl: 'https://modrinth.com/shader/complementary-reimagined',
		curseforgeUrl: 'https://www.curseforge.com/minecraft/shaders/complementary-reimagined'
	},
	{
		id: 'bsl-shaders',
		name: 'BSL Shaders',
		slug: 'bsl-shaders',
		author: 'CaptTatsu',
		authorUrl: 'https://modrinth.com/user/CaptTatsu',
		description: 'Beautiful shaders with extensive customization options',
		longDescription:
			'BSL Shaders offers a perfect balance between visual quality and performance. With over 200 customizable settings, you can fine-tune every aspect of your visuals from color grading to shadow quality.',
		style: 'vibrant',
		tier: 'medium',
		features: [
			{ name: 'Volumetric Fog', category: 'atmosphere' },
			{ name: 'Color Grading', category: 'effects' },
			{ name: 'Depth of Field', category: 'effects' },
			{ name: 'Motion Blur', category: 'effects' },
			{ name: 'Caustics', category: 'water' },
			{ name: 'Ambient Occlusion', category: 'lighting' }
		],
		version: '8.2.08',
		mcVersions: ['1.20.4', '1.21'],
		modrinthUrl: 'https://modrinth.com/shader/bsl-shaders'
	},
	{
		id: 'seus-ptgi',
		name: 'SEUS PTGI',
		slug: 'seus-ptgi',
		author: 'Sonic Ether',
		authorUrl: 'https://www.patreon.com/sonicether',
		description: 'Path-traced global illumination for ultimate realism',
		longDescription:
			'SEUS PTGI represents the cutting edge of Minecraft rendering. Using real-time path tracing, it delivers physically accurate lighting with global illumination, reflections, and soft shadows that rival modern games.',
		style: 'realistic',
		tier: 'ultra',
		features: [
			{ name: 'Path Tracing', category: 'lighting' },
			{ name: 'Global Illumination', category: 'lighting' },
			{ name: 'Ray Traced Reflections', category: 'effects' },
			{ name: 'Soft Shadows', category: 'lighting' },
			{ name: 'Volumetric Clouds', category: 'atmosphere' },
			{ name: 'PBR Materials', category: 'lighting' }
		],
		version: 'HRR 3',
		mcVersions: ['1.20.4'],
		websiteUrl: 'https://www.patreon.com/sonicether'
	},
	{
		id: 'sildurs-vibrant',
		name: "Sildur's Vibrant",
		slug: 'sildurs-vibrant',
		author: 'Sildur',
		authorUrl: 'https://sildurs-shaders.github.io/',
		description: 'Vibrant colors with excellent performance profiles',
		longDescription:
			"Sildur's Vibrant Shaders brings vivid colors and dynamic lighting while remaining accessible to a wide range of hardware. Available in multiple performance presets from Lite to Extreme.",
		style: 'vibrant',
		tier: 'low',
		features: [
			{ name: 'Godrays', category: 'lighting' },
			{ name: 'Bloom', category: 'effects' },
			{ name: 'Water Reflections', category: 'water' },
			{ name: 'Depth of Field', category: 'effects' },
			{ name: 'Multiple Presets', category: 'performance' }
		],
		version: '1.51',
		mcVersions: ['1.20.4', '1.21', '1.21.1'],
		modrinthUrl: 'https://modrinth.com/shader/sildurs-vibrant-shaders',
		curseforgeUrl: 'https://www.curseforge.com/minecraft/shaders/sildurs-vibrant-shaders'
	},
	{
		id: 'nostalgia',
		name: 'Nostalgia Shader',
		slug: 'nostalgia',
		author: 'RRe36',
		authorUrl: 'https://modrinth.com/user/RRe36',
		description: 'Classic aesthetic meets modern rendering techniques',
		longDescription:
			'Nostalgia Shader captures the warm, cozy feeling of older shader packs while leveraging modern rendering techniques. Perfect for those who want beauty without departing too far from vanilla aesthetics.',
		style: 'retro',
		tier: 'low',
		features: [
			{ name: 'Soft Lighting', category: 'lighting' },
			{ name: 'Subtle Bloom', category: 'effects' },
			{ name: 'Classic Water', category: 'water' },
			{ name: 'Lightweight', category: 'performance' }
		],
		version: '5.0',
		mcVersions: ['1.20.4', '1.21'],
		modrinthUrl: 'https://modrinth.com/shader/nostalgia-shader'
	},
	{
		id: 'kappa',
		name: 'Kappa Shader',
		slug: 'kappa',
		author: 'RRe36',
		authorUrl: 'https://modrinth.com/user/RRe36',
		description: 'High-end photorealistic rendering for enthusiasts',
		longDescription:
			'Kappa Shader pushes the boundaries of what Minecraft can look like. With advanced atmospheric scattering, physically-based materials, and cinematic color grading, it transforms your world into a visual masterpiece.',
		style: 'cinematic',
		tier: 'high',
		features: [
			{ name: 'Atmospheric Scattering', category: 'atmosphere' },
			{ name: 'Volumetric Clouds', category: 'atmosphere' },
			{ name: 'PBR Pipeline', category: 'lighting' },
			{ name: 'Screen-Space Reflections', category: 'effects' },
			{ name: 'Advanced Fog', category: 'atmosphere' }
		],
		version: '5.2',
		mcVersions: ['1.20.4'],
		modrinthUrl: 'https://modrinth.com/shader/kappa-shader'
	},
	{
		id: 'chocapic13',
		name: 'Chocapic13 Shaders',
		slug: 'chocapic13',
		author: 'Chocapic13',
		authorUrl: 'https://www.curseforge.com/members/chocapic13',
		description: 'One of the most influential shader packs in Minecraft history',
		longDescription:
			'Chocapic13 Shaders has been a cornerstone of the shader community for years. Known for its balanced approach and extensive compatibility, it remains a popular choice for players seeking enhanced visuals.',
		style: 'realistic',
		tier: 'medium',
		features: [
			{ name: 'Dynamic Shadows', category: 'lighting' },
			{ name: 'Godrays', category: 'lighting' },
			{ name: 'Bloom', category: 'effects' },
			{ name: 'Water Shaders', category: 'water' },
			{ name: 'Fog Effects', category: 'atmosphere' }
		],
		version: '9.1',
		mcVersions: ['1.20.4', '1.21'],
		curseforgeUrl: 'https://www.curseforge.com/minecraft/shaders/chocapic13-shaders'
	},
	{
		id: 'projectluma',
		name: 'projectLUMA',
		slug: 'projectluma',
		author: 'Lumas',
		authorUrl: 'https://dedelner.net/',
		description: 'Lightweight yet visually impressive shaders',
		longDescription:
			'projectLUMA delivers impressive visual fidelity while maintaining excellent performance. Its efficient design makes it perfect for players who want better graphics without upgrading their hardware.',
		style: 'minimal',
		tier: 'potato',
		features: [
			{ name: 'Efficient Lighting', category: 'lighting' },
			{ name: 'Clean Water', category: 'water' },
			{ name: 'Subtle Effects', category: 'effects' },
			{ name: 'Low VRAM', category: 'performance' }
		],
		version: '1.32',
		mcVersions: ['1.20.4', '1.21', '1.21.1'],
		modrinthUrl: 'https://modrinth.com/shader/projectluma'
	},
	{
		id: 'continuum',
		name: 'Continuum',
		slug: 'continuum',
		author: 'Continuum Team',
		authorUrl: 'https://continuum.graphics/',
		description: 'Film-grade color grading and cinematic visuals',
		longDescription:
			'Continuum brings Hollywood-quality post-processing to Minecraft. With its sophisticated color grading, film grain, and cinematic depth of field, every screenshot looks like a movie still.',
		style: 'cinematic',
		tier: 'ultra',
		features: [
			{ name: 'Film Color Grading', category: 'effects' },
			{ name: 'Cinematic DoF', category: 'effects' },
			{ name: 'Film Grain', category: 'effects' },
			{ name: 'Advanced GI', category: 'lighting' },
			{ name: 'Ray Traced Effects', category: 'lighting' }
		],
		version: '2.1',
		mcVersions: ['1.20.4'],
		websiteUrl: 'https://continuum.graphics/'
	},
	{
		id: 'seus-renewed',
		name: 'SEUS Renewed',
		slug: 'seus-renewed',
		author: 'Sonic Ether',
		authorUrl: 'https://www.sonicether.com/',
		description: 'The classic SEUS experience, modernized',
		longDescription:
			'SEUS Renewed takes the beloved classic SEUS shader and updates it for modern Minecraft. It maintains the signature SEUS look while improving performance and compatibility.',
		style: 'realistic',
		tier: 'medium',
		features: [
			{ name: 'Classic SEUS Look', category: 'lighting' },
			{ name: 'Godrays', category: 'lighting' },
			{ name: 'Water Reflections', category: 'water' },
			{ name: 'Bloom', category: 'effects' },
			{ name: 'Waving Plants', category: 'effects' }
		],
		version: '1.0.1',
		mcVersions: ['1.20.4', '1.21'],
		modrinthUrl: 'https://modrinth.com/shader/seus-renewed'
	},
	{
		id: 'astralex',
		name: 'AstraLex Shaders',
		slug: 'astralex',
		author: 'LexBoosT',
		authorUrl: 'https://modrinth.com/user/LexBoosT',
		description: 'Fantasy-inspired shaders with magical atmosphere',
		longDescription:
			'AstraLex brings a magical, fantasy-inspired aesthetic to Minecraft. With ethereal lighting, mystical fog effects, and dreamy color palettes, it transforms your world into something out of a fairy tale.',
		style: 'fantasy',
		tier: 'medium',
		features: [
			{ name: 'Fantasy Lighting', category: 'lighting' },
			{ name: 'Ethereal Fog', category: 'atmosphere' },
			{ name: 'Magical Colors', category: 'effects' },
			{ name: 'Soft Shadows', category: 'lighting' },
			{ name: 'Dreamy Water', category: 'water' }
		],
		version: '3.1',
		mcVersions: ['1.20.4', '1.21'],
		modrinthUrl: 'https://modrinth.com/shader/astralex'
	},
	{
		id: 'photon',
		name: 'Photon Shader',
		slug: 'photon',
		author: 'SixthSurge',
		authorUrl: 'https://modrinth.com/user/SixthSurge',
		description: 'Modern path-traced shaders with excellent optimization',
		longDescription:
			'Photon brings cutting-edge path tracing technology to Minecraft with surprising efficiency. Its clever optimizations make ray-traced lighting accessible to more players than ever before.',
		style: 'realistic',
		tier: 'high',
		features: [
			{ name: 'Path Tracing', category: 'lighting' },
			{ name: 'Global Illumination', category: 'lighting' },
			{ name: 'Optimized Rays', category: 'performance' },
			{ name: 'PBR Support', category: 'lighting' },
			{ name: 'Volumetrics', category: 'atmosphere' }
		],
		version: '1.1.0',
		mcVersions: ['1.20.4', '1.21'],
		modrinthUrl: 'https://modrinth.com/shader/photon-shader'
	}
];

// Raw scene data (without thumbnails)
const SCENES_DATA: Omit<Scene, 'thumbnail'>[] = [
	{
		id: 'forest-clearing',
		name: 'Sunlit Forest Clearing',
		slug: 'forest-clearing',
		description:
			'A peaceful forest clearing with sunlight streaming through the canopy, showcasing foliage rendering and light ray effects',
		dimension: 'overworld',
		biome: 'Forest',
		defaultTime: 'morning',
		defaultWeather: 'clear',
		features: ['Dense Foliage', 'Light Rays', 'Mixed Shadows', 'Grass Rendering'],
		complexity: 'moderate'
	},
	{
		id: 'village-sunset',
		name: 'Village at Sunset',
		slug: 'village-sunset',
		description:
			'A cozy village bathed in warm sunset light with torch-lit interiors, testing color temperature and emissive lighting',
		dimension: 'overworld',
		biome: 'Plains',
		defaultTime: 'sunset',
		defaultWeather: 'clear',
		features: ['Warm Lighting', 'Torch Glow', 'Building Shadows', 'Sky Gradient'],
		complexity: 'complex'
	},
	{
		id: 'ocean-depths',
		name: 'Ocean Monument',
		slug: 'ocean-depths',
		description:
			'An underwater ocean monument with caustic light patterns and deep ocean fog effects',
		dimension: 'overworld',
		biome: 'Deep Ocean',
		defaultTime: 'noon',
		defaultWeather: 'clear',
		features: ['Underwater Caustics', 'Ocean Fog', 'Prismarine Glow', 'Water Absorption'],
		complexity: 'complex'
	},
	{
		id: 'cave-system',
		name: 'Crystal Caverns',
		slug: 'cave-system',
		description:
			'A lush cave system with amethyst geodes and glowing elements, perfect for testing ambient occlusion and point lights',
		dimension: 'overworld',
		biome: 'Lush Caves',
		defaultTime: 'noon',
		defaultWeather: 'clear',
		features: ['Amethyst Glow', 'Cave Darkness', 'Ambient Occlusion', 'Moss Rendering'],
		complexity: 'moderate'
	},
	{
		id: 'nether-fortress',
		name: 'Nether Fortress',
		slug: 'nether-fortress',
		description:
			'A dramatic nether fortress with lava lakes and fire particles, testing emissive materials and harsh lighting',
		dimension: 'nether',
		biome: 'Nether Wastes',
		defaultTime: 'noon',
		defaultWeather: 'clear',
		features: ['Lava Glow', 'Fire Particles', 'Nether Fog', 'Dark Atmosphere'],
		complexity: 'complex'
	},
	{
		id: 'end-gateway',
		name: 'End Island',
		slug: 'end-gateway',
		description:
			'The main End island with the dragon perch, testing void rendering and the unique End atmosphere',
		dimension: 'end',
		biome: 'The End',
		defaultTime: 'noon',
		defaultWeather: 'clear',
		features: ['Void Sky', 'End Stone', 'Obsidian Towers', 'Dragon Effects'],
		complexity: 'moderate'
	},
	{
		id: 'mountain-vista',
		name: 'Mountain Vista',
		slug: 'mountain-vista',
		description:
			'Dramatic mountain peaks with volumetric clouds and distant fog, ideal for testing atmospheric rendering',
		dimension: 'overworld',
		biome: 'Jagged Peaks',
		defaultTime: 'morning',
		defaultWeather: 'cloudy',
		features: ['Volumetric Clouds', 'Distance Fog', 'Snow Rendering', 'Height Fog'],
		complexity: 'simple'
	},
	{
		id: 'flower-meadow',
		name: 'Flower Meadow',
		slug: 'flower-meadow',
		description:
			'A vibrant flower meadow under bright sunlight with bees and butterflies, showcasing color saturation',
		dimension: 'overworld',
		biome: 'Flower Forest',
		defaultTime: 'noon',
		defaultWeather: 'clear',
		features: ['Bright Colors', 'Particle Effects', 'Waving Flowers', 'Bee Particles'],
		complexity: 'simple'
	},
	{
		id: 'ancient-city',
		name: 'Ancient City',
		slug: 'ancient-city',
		description:
			'The eerie Ancient City in the Deep Dark, testing extreme low-light rendering and sculk effects',
		dimension: 'overworld',
		biome: 'Deep Dark',
		defaultTime: 'noon',
		defaultWeather: 'clear',
		features: ['Sculk Glow', 'Extreme Darkness', 'Soul Lanterns', 'Eerie Atmosphere'],
		complexity: 'complex'
	},
	{
		id: 'bamboo-jungle',
		name: 'Bamboo Grove',
		slug: 'bamboo-jungle',
		description:
			'Dense bamboo jungle with filtered light and jungle fog, testing dense vegetation rendering',
		dimension: 'overworld',
		biome: 'Bamboo Jungle',
		defaultTime: 'afternoon',
		defaultWeather: 'rain',
		features: ['Dense Bamboo', 'Filtered Light', 'Rain Effects', 'Wet Surfaces'],
		complexity: 'complex'
	},
	{
		id: 'desert-temple',
		name: 'Desert Temple',
		slug: 'desert-temple',
		description:
			'A desert temple under harsh midday sun with heat shimmer effects and stark shadows',
		dimension: 'overworld',
		biome: 'Desert',
		defaultTime: 'noon',
		defaultWeather: 'clear',
		features: ['Harsh Shadows', 'Heat Haze', 'Sand Particles', 'Bright Exposure'],
		complexity: 'simple'
	},
	{
		id: 'snowy-night',
		name: 'Snowy Village Night',
		slug: 'snowy-night',
		description: 'A snow-covered village at night with aurora effects and warm interior lighting',
		dimension: 'overworld',
		biome: 'Snowy Taiga',
		defaultTime: 'night',
		defaultWeather: 'snow',
		features: ['Aurora Borealis', 'Snow Particles', 'Night Lighting', 'Warm Interiors'],
		complexity: 'complex'
	}
];

// Data generation functions

function generateShaders(): Shader[] {
	return SHADERS_DATA.map((shader, index) => {
		const rng = seededRandom(index * 1337);
		return {
			...shader,
			thumbnail: getRandomWallpaper(index * 7919),
			downloadCount: Math.floor(rng() * 8000000) + 500000,
			likes: Math.floor(rng() * 100000) + 5000,
			lastUpdated: new Date(Date.now() - Math.floor(rng() * 90 * 24 * 60 * 60 * 1000)).toISOString()
		};
	});
}

function generateScenes(): Scene[] {
	return SCENES_DATA.map((scene, index) => ({
		...scene,
		thumbnail: getRandomWallpaper(index * 3571 + 500)
	}));
}

function generateCaptures(): Capture[] {
	const shaders = getAllShaders();
	const scenes = getAllScenes();
	const captures: Capture[] = [];

	let captureIndex = 0;
	for (const shader of shaders) {
		for (const scene of scenes) {
			const rng = seededRandom(captureIndex * 2749);

			// Base FPS depends on shader tier and scene complexity
			const tierMultiplier: Record<PerformanceTier, number> = {
				potato: 1.0,
				low: 0.85,
				medium: 0.65,
				high: 0.45,
				ultra: 0.25
			};
			const complexityMultiplier: Record<string, number> = {
				simple: 1.2,
				moderate: 1.0,
				complex: 0.75
			};

			const baseFps = 144 * tierMultiplier[shader.tier] * complexityMultiplier[scene.complexity];
			const fps = Math.round(baseFps * (0.85 + rng() * 0.3));

			captures.push({
				id: `${shader.id}-${scene.id}`,
				shaderId: shader.id,
				sceneId: scene.id,
				image: getRandomWallpaper(captureIndex * 4231),
				timeOfDay: scene.defaultTime,
				weather: scene.defaultWeather,
				fps,
				frameTimeMs: parseFloat((1000 / fps).toFixed(2)),
				gpuUsage: Math.min(99, Math.round(70 + rng() * 25)),
				vramMb: Math.round(2000 + rng() * 6000),
				capturedAt: new Date(
					Date.now() - Math.floor(rng() * 30 * 24 * 60 * 60 * 1000)
				).toISOString()
			});
			captureIndex++;
		}
	}

	return captures;
}

// Caches
let _shadersCache: Shader[] | null = null;
let _scenesCache: Scene[] | null = null;
let _capturesCache: Capture[] | null = null;

// Data accessors

export function getAllShaders(): Shader[] {
	_shadersCache ??= generateShaders();
	return _shadersCache;
}

export function getAllScenes(): Scene[] {
	_scenesCache ??= generateScenes();
	return _scenesCache;
}

export function getAllCaptures(): Capture[] {
	_capturesCache ??= generateCaptures();
	return _capturesCache;
}

export function getShaderById(id: string): Shader | undefined {
	return getAllShaders().find((s) => s.id === id);
}

export function getSceneById(id: string): Scene | undefined {
	return getAllScenes().find((s) => s.id === id);
}

export function getCapturesForShader(shaderId: string): Capture[] {
	return getAllCaptures().filter((c) => c.shaderId === shaderId);
}

export function getCapturesForScene(sceneId: string): Capture[] {
	return getAllCaptures().filter((c) => c.sceneId === sceneId);
}

export function getCapture(shaderId: string, sceneId: string): Capture | undefined {
	return getAllCaptures().find((c) => c.shaderId === shaderId && c.sceneId === sceneId);
}

export function getShaderPerformance(shaderId: string): ShaderPerformance | undefined {
	const captures = getCapturesForShader(shaderId);
	if (captures.length === 0) return undefined;

	const fpsList = captures.map((c) => c.fps);
	return {
		shaderId,
		avgFps: Math.round(fpsList.reduce((a, b) => a + b, 0) / fpsList.length),
		minFps: Math.min(...fpsList),
		maxFps: Math.max(...fpsList),
		avgFrameTime: parseFloat(
			(captures.reduce((a, c) => a + c.frameTimeMs, 0) / captures.length).toFixed(2)
		),
		avgGpuUsage: Math.round(captures.reduce((a, c) => a + c.gpuUsage, 0) / captures.length),
		avgVram: Math.round(captures.reduce((a, c) => a + c.vramMb, 0) / captures.length),
		sceneCount: captures.length
	};
}

export function getScenePerformance(sceneId: string): ScenePerformance | undefined {
	const captures = getCapturesForScene(sceneId);
	if (captures.length === 0) return undefined;

	const fpsList = captures.map((c) => c.fps);
	const sorted = [...captures].sort((a, b) => b.fps - a.fps);

	return {
		sceneId,
		avgFps: Math.round(fpsList.reduce((a, b) => a + b, 0) / fpsList.length),
		minFps: Math.min(...fpsList),
		maxFps: Math.max(...fpsList),
		shaderCount: captures.length,
		easiestShader: sorted[0]?.shaderId || '',
		hardestShader: sorted[sorted.length - 1]?.shaderId || ''
	};
}

// Formatting utilities

export function formatNumber(num: number): string {
	if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
	if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
	return num.toString();
}

export function formatDate(isoString: string): string {
	const date = new Date(isoString);
	const now = new Date();
	const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));

	if (diffDays === 0) return 'Today';
	if (diffDays === 1) return 'Yesterday';
	if (diffDays < 7) return `${diffDays} days ago`;
	if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
	if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
	return `${Math.floor(diffDays / 365)} years ago`;
}

// Color utilities for badges

export function getTierColor(tier: PerformanceTier): string {
	const colors: Record<PerformanceTier, string> = {
		potato: 'bg-lime-900/70 text-lime-100',
		low: 'bg-emerald-900/70 text-emerald-100',
		medium: 'bg-amber-900/70 text-amber-100',
		high: 'bg-orange-900/70 text-orange-100',
		ultra: 'bg-red-900/70 text-red-100'
	};
	return colors[tier];
}

export function getTierLabel(tier: PerformanceTier): string {
	const labels: Record<PerformanceTier, string> = {
		potato: 'Featherlight',
		low: 'Lightweight',
		medium: 'Balanced',
		high: 'Heavy',
		ultra: 'Intensive'
	};
	return labels[tier];
}

export function getStyleColor(style: ShaderStyle): string {
	const colors: Record<ShaderStyle, string> = {
		realistic: 'bg-blue-500/15 text-blue-700 dark:text-blue-300 ring-1 ring-blue-500/30',
		fantasy: 'bg-purple-500/15 text-purple-700 dark:text-purple-300 ring-1 ring-purple-500/30',
		vibrant: 'bg-pink-500/15 text-pink-700 dark:text-pink-300 ring-1 ring-pink-500/30',
		minimal: 'bg-gray-500/15 text-gray-700 dark:text-gray-300 ring-1 ring-gray-500/30',
		retro: 'bg-amber-500/15 text-amber-700 dark:text-amber-300 ring-1 ring-amber-500/30',
		cinematic: 'bg-rose-500/15 text-rose-700 dark:text-rose-300 ring-1 ring-rose-500/30'
	};
	return colors[style];
}

export function getTimeColor(time: TimeOfDay): string {
	const colors: Record<TimeOfDay, string> = {
		dawn: 'bg-orange-900/70 text-orange-100',
		morning: 'bg-amber-900/70 text-amber-100',
		noon: 'bg-yellow-900/70 text-yellow-100',
		afternoon: 'bg-orange-900/70 text-orange-100',
		sunset: 'bg-rose-900/70 text-rose-100',
		dusk: 'bg-purple-900/70 text-purple-100',
		night: 'bg-indigo-900/70 text-indigo-100',
		midnight: 'bg-slate-900/70 text-slate-100'
	};
	return colors[time];
}

export function getWeatherColor(weather: Weather): string {
	const colors: Record<Weather, string> = {
		clear: 'bg-sky-900/70 text-sky-100',
		cloudy: 'bg-gray-900/70 text-gray-100',
		rain: 'bg-blue-900/70 text-blue-100',
		storm: 'bg-purple-900/70 text-purple-100',
		snow: 'bg-cyan-900/70 text-cyan-100',
		fog: 'bg-gray-900/70 text-gray-100'
	};
	return colors[weather];
}

export function getDimensionColor(dimension: Dimension): string {
	const colors: Record<Dimension, string> = {
		overworld: 'bg-green-500/15 text-green-700 dark:text-green-300 ring-1 ring-green-500/30',
		nether: 'bg-red-500/15 text-red-700 dark:text-red-300 ring-1 ring-red-500/30',
		end: 'bg-purple-500/15 text-purple-700 dark:text-purple-300 ring-1 ring-purple-500/30'
	};
	return colors[dimension];
}

export function getBiomeColor(biome: string): string {
	switch (biome.toLowerCase()) {
		case 'forest':
			return 'bg-green-500/15 text-green-700 dark:text-green-300 ring-1 ring-green-500/30';
		case 'plains':
			return 'bg-lime-500/15 text-lime-700 dark:text-lime-300 ring-1 ring-lime-500/30';
		case 'cave':
		case 'deep dark':
		case 'lush caves':
			return 'bg-stone-500/15 text-stone-700 dark:text-stone-300 ring-1 ring-stone-500/30';
		case 'nether':
		case 'nether wastes':
			return 'bg-red-500/15 text-red-700 dark:text-red-300 ring-1 ring-red-500/30';
		case 'end':
		case 'the end':
			return 'bg-violet-500/15 text-violet-700 dark:text-violet-300 ring-1 ring-violet-500/30';
		case 'ocean':
		case 'deep ocean':
			return 'bg-cyan-500/15 text-cyan-700 dark:text-cyan-300 ring-1 ring-cyan-500/30';
		case 'mountains':
		case 'jagged peaks':
			return 'bg-slate-500/15 text-slate-700 dark:text-slate-300 ring-1 ring-slate-500/30';
		case 'jungle':
		case 'bamboo jungle':
			return 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 ring-1 ring-emerald-500/30';
		case 'desert':
			return 'bg-amber-500/15 text-amber-700 dark:text-amber-300 ring-1 ring-amber-500/30';
		case 'taiga':
		case 'snowy taiga':
			return 'bg-teal-500/15 text-teal-700 dark:text-teal-300 ring-1 ring-teal-500/30';
		case 'flower forest':
			return 'bg-pink-500/15 text-pink-700 dark:text-pink-300 ring-1 ring-pink-500/30';
		default:
			return 'bg-gray-500/15 text-gray-700 dark:text-gray-300 ring-1 ring-gray-500/30';
	}
}

export function getFpsColor(fps: number): string {
	if (fps >= 60) return 'text-emerald-500';
	if (fps >= 30) return 'text-amber-500';
	return 'text-red-500';
}

export function getFpsBgColor(fps: number): string {
	if (fps >= 60) return 'bg-emerald-500';
	if (fps >= 30) return 'bg-amber-500';
	return 'bg-red-500';
}

// Icon path helpers for time/weather badges

export function getTimeIconPath(time: TimeOfDay): string {
	switch (time) {
		case 'dawn':
			return 'M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z';
		case 'morning':
		case 'noon':
		case 'afternoon':
		case 'sunset':
		case 'dusk':
			return 'M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z';
		case 'night':
		case 'midnight':
			return 'M21.752 15.002A9.718 9.718 0 0118 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 003 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 009.002-5.998z';
		default:
			return 'M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z';
	}
}

export function getWeatherIconPath(weather: Weather): string {
	switch (weather) {
		case 'clear':
			return 'M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z';
		case 'cloudy':
		case 'fog':
		case 'rain':
		case 'snow':
			return 'M2.25 15a4.5 4.5 0 004.5 4.5H18a3.75 3.75 0 001.332-7.257 3 3 0 00-3.758-3.848 5.25 5.25 0 00-10.233 2.33A4.502 4.502 0 002.25 15z';
		case 'storm':
			return 'M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z';
		default:
			return 'M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z';
	}
}
