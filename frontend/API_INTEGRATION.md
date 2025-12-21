# Frontend API Integration Guide

## Overview

The frontend now has a complete API client infrastructure for communicating with the Rust backend. This implementation uses:

- **Result Pattern**: `true-myth` for type-safe error handling
- **Typed Client**: Organized endpoint classes with full TypeScript support
- **Hybrid Loading**: Server-side load functions + client-side fetching
- **Vite Proxy**: Seamless dev experience with `/api/*` proxying

## Architecture

```
frontend/src/lib/
├── api/
│   ├── client.ts           # Base ApiClient class with fetch wrapper
│   ├── config.ts           # API URL configuration
│   ├── types.ts            # Backend-aligned API types
│   ├── adapters.ts         # Transform backend → frontend types
│   ├── endpoints/
│   │   ├── shaders.ts      # Shader endpoint methods
│   │   ├── scenes.ts       # Scene endpoint methods
│   │   └── captures.ts     # Capture endpoint methods
│   ├── errors.ts           # API error types
│   └── index.ts            # Barrel exports
├── data/
│   ├── types.ts            # Frontend display types (enhanced from API)
│   └── mock.ts             # Legacy mock data (kept for reference)
```

## Usage Examples

### In Load Functions (Server-Side)

```typescript
// routes/shaders/+page.ts
import { api } from '$lib/api';
import { adaptShader } from '$lib/api/adapters';

export async function load() {
	const result = await api.shaders.list();

	return result.match({
		Ok: (apiShaders) => ({
			shaders: apiShaders.map(adaptShader)
		}),
		Err: (error) => {
			console.error('Failed to load shaders:', error);
			throw error; // SvelteKit will show error page
		}
	});
}
```

### In Components (Client-Side)

```typescript
// In a .svelte component
import { api } from '$lib/api';
import { adaptShader } from '$lib/api/adapters';
import type { Shader } from '$lib/data/types';

let shaders = $state<Shader[]>([]);
let error = $state<string | null>(null);
let loading = $state(false);

async function loadShaders() {
	loading = true;
	const result = await api.shaders.list();

	result.match({
		Ok: (data) => {
			shaders = data.map(adaptShader);
			error = null;
		},
		Err: (err) => {
			error = err.message;
		}
	});
	loading = false;
}
```

## Available Endpoints

### Shaders

- `api.shaders.list()` → `Result<Shader[], ApiError>`
- `api.shaders.getBySlug(slug)` → `Result<ShaderWithCaptures, ApiError>`

### Scenes

- `api.scenes.list()` → `Result<Scene[], ApiError>`
- `api.scenes.getBySlug(slug)` → `Result<SceneWithCaptures, ApiError>`

### Captures

- `api.captures.list()` → `Result<CaptureWithContext[], ApiError>`
- `api.captures.getById(id)` → `Result<CaptureWithContext, ApiError>`

## Configuration

### Development

The Vite dev server proxies `/api/*` requests to `http://localhost:8080` (backend).

No configuration needed - just start both servers:

```bash
# Terminal 1: Backend
cd backend && cargo run

# Terminal 2: Frontend
cd frontend && bun dev
```

### Production

Set the `PUBLIC_API_URL` environment variable:

```bash
# .env (production)
PUBLIC_API_URL=https://api.glint.example.com
```

If not set, defaults to `/api` (relative URLs).

## Type System

### Backend Types (lib/api/types.ts)

Direct mirrors of Rust backend models:

- `Shader` - Basic shader info from DB
- `Scene` - Scene with world reference
- `Capture` - Screenshot capture record
- `ShaderWithCaptures` - Shader + versions + captures
- `SceneWithCaptures` - Scene + world + captures

### Frontend Types (lib/data/types.ts)

Enhanced types with UI metadata:

- `Shader` - Extends API type with `author`, `thumbnail`, `style`, etc.
- `Scene` - Extends API type with `dimension`, `biome`, `complexity`
- `Capture` - Display-friendly with stubbed performance metrics

### Adapters (lib/api/adapters.ts)

Transform functions that:

- Map backend types → frontend display types
- Fill in missing data with defaults/placeholders
- Handle optional fields gracefully

Example:

```typescript
export function adaptShader(apiShader: ApiShader): Shader {
	return {
		...apiShader,
		author: 'Unknown', // Stubbed until backend provides
		thumbnail: '/placeholder.png',
		style: 'realistic', // Default
		tier: 'medium'
		// ... etc
	};
}
```

## Error Handling

### Error Types

- `NetworkError` - Fetch failed, network issues
- `NotFoundError` - 404 responses
- `ValidationError` - 400 responses with validation details
- `ServerError` - 500+ responses
- `UnknownError` - Fallback for unexpected errors

### Handling Errors

```typescript
const result = await api.shaders.list();

result.match({
	Ok: (data) => {
		// Success case
	},
	Err: (error) => {
		switch (error.type) {
			case 'network':
				// Show "Check your connection"
				break;
			case 'not_found':
				// Show 404 page
				break;
			case 'validation':
				// Show validation errors
				console.log(error.errors);
				break;
			case 'server':
				// Show "Server error, try again"
				break;
		}
	}
});
```

## Current Status

### ✅ Implemented

- Base API client with Result pattern
- All endpoint methods (shaders, scenes, captures)
- Error handling and typing
- Vite proxy configuration
- Type adapters for backend → frontend
- Load functions for `/shaders` and `/scenes` list pages
- Updated list page components to use real API

### ⚠️ Partial / Stubbed

- Detail pages (`/shaders/[id]`, `/scenes/[id]`) still use mock data
- Performance metrics (FPS, GPU, VRAM) - backend doesn't provide yet
- Author info, download counts - stubbed with defaults
- Filtering/sorting - client-side only, may break with missing fields

### ❌ Not Implemented (Backend Missing)

- Shader metadata (author, downloads, likes)
- Performance metrics capture
- Comparison page integration
- Scene metadata (dimension, biome, time, weather)

## Next Steps

1. **Fix Detail Pages**: Update `/shaders/[id]/+page.ts` and `/scenes/[id]/+page.ts` to use API
2. **Handle Missing Data**: Gracefully hide UI elements for missing backend data
3. **Error Pages**: Create better error displays for failed API calls
4. **Loading States**: Add loading skeletons for better UX
5. **Comparison Page**: Integrate API data into comparison UI
6. **Backend Sync**: As backend adds fields, remove adapters' stub data

## Testing

### Manual Testing

1. Start backend: `cd backend && cargo run`
2. Start frontend: `cd frontend && bun dev`
3. Visit `http://localhost:5173/shaders`
4. Check browser console for API calls
5. Verify data loads (or see error messages if backend is empty)

### Type Checking

```bash
cd frontend
bun run check
```

### Known Type Issues

- Detail pages have ~106 type errors (not yet updated for new API)
- These will be fixed when detail pages are migrated from mock data

## Notes

- **Alpha Status**: Some fields are stubbed/hardcoded - this is expected
- **Broken Filtering**: Filtering may not work correctly with missing optional fields
- **Mock Data**: `lib/data/mock.ts` is kept for reference but no longer used in list pages
- **CDN URLs**: Backend provides `screenshot_url` directly, no construction needed
- **No Auth**: API is currently unauthenticated (alpha only)
