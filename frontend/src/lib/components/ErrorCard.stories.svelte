<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import ErrorCard from './ErrorCard.svelte';

const { Story } = defineMeta({
	title: 'Components/ErrorCard',
	component: ErrorCard,
	tags: ['autodocs'],
	parameters: {
		layout: 'padded'
	}
});
</script>

<Story
	name="Basic"
	args={{
		title: 'Something went wrong',
		message: 'An unexpected error occurred while loading shader data.',
	}}
/>

<Story
	name="With Status"
	args={{
		title: 'Not Found',
		statusLabel: '404',
		message: 'The shader pack you requested could not be found.',
	}}
/>

<Story
	name="Server Error"
	args={{
		title: 'Internal Server Error',
		statusLabel: '500',
		message: 'The server encountered an unexpected condition.',
		code: 'INTERNAL_ERROR',
		errorId: 'err_a1b2c3d4e5f6',
		requestId: 'req_x9y8z7w6v5u4',
		timestamp: new Date().toISOString(),
	}}
/>

<Story
	name="With Stack Trace"
	args={{
		title: 'Runtime Error',
		statusLabel: '500',
		message: "Cannot read properties of undefined (reading 'name')",
		code: 'TYPE_ERROR',
		errorId: 'err_stack123',
		timestamp: new Date().toISOString(),
		stack: `TypeError: Cannot read properties of undefined (reading 'name')
    at getShaderName (src/lib/utils/display.ts:42:18)
    at ShaderCard.svelte:25:14
    at Object.$$render (node_modules/svelte/internal/index.js:1337:22)
    at eval (src/routes/+page.svelte:84:34)`,
		defaultStackOpen: true,
	}}
/>

<Story
	name="With Detail"
	args={{
		title: 'Database Error',
		statusLabel: '503',
		message: 'Failed to connect to the database.',
		detail:
			'Connection to PostgreSQL at localhost:5432 timed out after 5000ms.\nRetried 3 times before giving up.',
		code: 'DB_CONNECTION_TIMEOUT',
		errorId: 'err_db_timeout_789',
	}}
/>

<Story
	name="Minimal"
	args={{
		title: 'Error',
	}}
/>
