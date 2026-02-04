import type { PageLoad } from './$types';

export const load: PageLoad = ({ url }) => {
	// Get user_code from URL query params
	const userCode = url.searchParams.get('code');

	return {
		userCode
	};
};
