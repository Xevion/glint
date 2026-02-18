import { graphql } from '$lib/graphql/tada';

export const ScenesQuery = graphql(`
	query Scenes($first: Int!, $after: String) {
		scenes(first: $first, after: $after) {
			edges {
				cursor
				node {
					id
					name
					slug
					description
					dimension
					active
					createdAt
					imagePath
					thumbhash
					captureCount
					version {
						timeOfDayTicks
						weather
						biome
					}
				}
			}
			pageInfo {
				hasNextPage
				endCursor
			}
			totalCount
		}
	}
`);

export const SceneByIdQuery = graphql(`
	query SceneById($id: String!) {
		scene(id: $id) {
			id
			name
			slug
			description
			dimension
			active
			createdAt
			imagePath
			thumbhash
			captureCount
			version {
				timeOfDayTicks
				weather
				weatherIntensity
				moonPhase
				biome
				fov
				renderDistance
			}
			presets {
				id
				name
				slug
				timeOfDayTicks
				weather
				weatherIntensity
				moonPhase
				sortOrder
			}
		}
	}
`);
