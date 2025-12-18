# Glint Frontend

SvelteKit-based web application for browsing and comparing Minecraft shaders.

## Tech Stack

- **Framework**: SvelteKit 2.x with Svelte 5
- **Language**: TypeScript
- **Styling**: Tailwind CSS v4
- **Components**: shadcn-svelte
- **Package Manager**: Bun

## Project Structure

```
frontend/
├── src/
│   ├── lib/
│   │   ├── components/    # Reusable components
│   │   │   ├── ui/        # shadcn-svelte components
│   │   │   └── Navigation.svelte
│   │   └── utils.ts       # Utility functions (cn, etc.)
│   ├── routes/            # SvelteKit routes
│   │   ├── +layout.svelte
│   │   ├── +page.svelte   # Landing page
│   │   ├── shaders/       # Shader catalog & detail pages
│   │   ├── scenes/        # Scene catalog & detail pages
│   │   └── compare/       # Comparison tool
│   └── app.html
├── static/                # Static assets
└── package.json
```

## Routes

| Route                | Description                                  |
|----------------------|----------------------------------------------|
| `/`                  | Landing page with dual-pane navigation       |
| `/shaders`           | List of all shaders                          |
| `/shaders/[id]`      | Shader detail with all scenes                |
| `/scenes`            | List of all scenes                           |
| `/scenes/[id]`       | Scene detail with all shaders                |
| `/compare`           | Side-by-side shader comparison tool          |

## Development

```bash
# Install dependencies
bun install

# Start dev server (with hot reload)
bun dev

# Type checking
bun run check

# Linting
bun run lint

# Format code
bun run format

# Build for production
bun run build

# Preview production build
bun run preview
```

## Adding Components

Use shadcn-svelte CLI to add components:

```bash
bunx shadcn-svelte@latest add [component-name]
```

## Environment Variables

Create a `.env.local` file for local configuration:

```env
# API endpoint (when backend is ready)
PUBLIC_API_URL=http://localhost:8080

# CDN base URL (when storage is configured)
PUBLIC_CDN_URL=https://cdn.example.com
```

## Code Style

- Use TypeScript for all new files
- Follow the existing component patterns
- Use Tailwind CSS classes (avoid custom CSS when possible)
- Leverage shadcn-svelte components for UI consistency
- Use absolute imports (`$lib/...`) over relative imports

## Future Enhancements

- [ ] API integration with Rust backend
- [ ] Real shader data from capture system
- [ ] Interactive comparison sliders
- [ ] Performance graphs and charts
- [ ] Filter and search functionality
- [ ] OAuth authentication (Discord/GitHub)
- [ ] Favorites and user preferences
- [ ] Embeddable widgets for shader developers
