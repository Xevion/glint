import { ApiErrorType, pageError } from '$lib/api/errors';
import { createGraphQLClient, query } from '$lib/graphql';
import type { PageLoad } from './$types';
import { AdminCaptureRunQuery } from './queries';

export const load: PageLoad = async ({ params, fetch }) => {
	const client = createGraphQLClient(fetch);
	const result = await query(client, AdminCaptureRunQuery, { id: params.id });

	return result.match({
		Ok: (data) => {
			if (!data.adminCaptureRun) {
				pageError(404, 'Run not found');
			}
			const run = data.adminCaptureRun!;
			return { run };
		},
		Err: (err) => {
			if (err.type === ApiErrorType.NotFound) pageError(404, 'Run not found');
			return err.throw();
		}
	});
};
