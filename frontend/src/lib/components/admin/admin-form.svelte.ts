import { invalidateAll } from '$app/navigation';
import type { ApiError } from '$lib/api/errors';
import type { Result } from 'true-myth';
import { untrack } from 'svelte';

type FieldExtractor<T> = (source: T) => string;

interface AdminFormConfig<T> {
	/** Reactive getter for the source entity (e.g. `() => shader`) */
	source: () => T;
	/** Map of field names to extractors that pull initial values from the source */
	fields: Record<string, FieldExtractor<T>>;
	/**
	 * Called with only the changed fields and the current source entity.
	 * Return `Result.ok()` to trigger invalidation, `Result.err()` to display an error.
	 */
	onSave: (changes: Record<string, string>, source: T) => Promise<Result<unknown, ApiError>>;
	/** Called after successful save. Defaults to `invalidateAll()`. */
	onSuccess?: () => void;
}

/**
 * Composable for admin detail page edit forms.
 *
 * Manages reactive edit fields initialized from a source entity, dirty tracking,
 * automatic field reset when the source changes, and the save ceremony
 * (loading state, error handling, data invalidation).
 *
 * @example
 * ```ts
 * const form = createAdminForm({
 *   source: () => shader,
 *   fields: {
 *     name: (s) => s.name,
 *     description: (s) => s.description ?? '',
 *   },
 *   onSave: (changes, source) => api.admin.updateShader(source.id, changes),
 * });
 * // Template: bind:value={form.fields.name}
 * // Template: disabled={form.saving || !form.isDirty}
 * // Template: onclick={form.save}
 * ```
 */
export function createAdminForm<T>(config: AdminFormConfig<T>) {
	const fieldKeys = Object.keys(config.fields);

	// Initialize eagerly from current source
	const initial: Record<string, string> = {};
	const src = config.source();
	for (const key of fieldKeys) {
		initial[key] = config.fields[key](src);
	}

	const fields = $state(initial);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// Reset fields when source entity changes (e.g. after invalidateAll or navigation)
	$effect(() => {
		const source = config.source();
		untrack(() => {
			for (const key of fieldKeys) {
				fields[key] = config.fields[key](source);
			}
		});
	});

	const isDirty = $derived.by(() => {
		const source = config.source();
		return fieldKeys.some((key) => fields[key] !== config.fields[key](source));
	});

	async function save() {
		if (!isDirty) return;

		saving = true;
		error = null;

		try {
			const source = config.source();
			const changes: Record<string, string> = {};
			for (const key of fieldKeys) {
				if (fields[key] !== config.fields[key](source)) {
					changes[key] = fields[key];
				}
			}

			const result = await config.onSave(changes, source);
			result.match({
				Ok: () => {
					if (config.onSuccess) {
						config.onSuccess();
					} else {
						void invalidateAll();
					}
				},
				Err: (err) => {
					error = err.message;
				}
			});
		} finally {
			saving = false;
		}
	}

	return {
		/** Reactive field values — use with `bind:value={form.fields.name}` */
		get fields() {
			return fields;
		},
		/** Whether any field differs from the source entity */
		get isDirty() {
			return isDirty;
		},
		/** Whether a save is in progress */
		get saving() {
			return saving;
		},
		/** Error message from the last failed operation (save or external) */
		get error() {
			return error;
		},
		/** Allow external operations (delete, sync) to share this error state */
		set error(v: string | null) {
			error = v;
		},
		/** Save only the changed fields. No-op if form is clean. */
		save
	};
}

interface AdminActionConfig {
	/** The async operation to perform */
	action: () => Promise<Result<unknown, ApiError>>;
	/** Called on success (e.g. `() => void goto('/admin/shaders')`) */
	onSuccess: () => void;
	/** Write errors to a shared error state (e.g. `(msg) => form.error = msg`) */
	setError: (message: string) => void;
}

/**
 * Composable for one-off admin actions (delete, disable, reactivate, etc.).
 *
 * Manages loading state and delegates error display to a shared setter,
 * typically the form's error state.
 *
 * @example
 * ```ts
 * const deleteAction = createAdminAction({
 *   action: () => api.admin.deleteShader(shader.id),
 *   onSuccess: () => void goto('/admin/shaders'),
 *   setError: (msg) => (form.error = msg),
 * });
 * // Template: disabled={deleteAction.loading}
 * // Template: onclick={deleteAction.execute}
 * ```
 */
export function createAdminAction(config: AdminActionConfig) {
	let loading = $state(false);

	async function execute() {
		loading = true;
		try {
			const result = await config.action();
			result.match({
				Ok: () => config.onSuccess(),
				Err: (err) => config.setError(err.message)
			});
		} finally {
			loading = false;
		}
	}

	return {
		/** Whether the action is in progress */
		get loading() {
			return loading;
		},
		/** Execute the action */
		execute
	};
}
