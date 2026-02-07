import type { PageLoad } from './$types';

export const load: PageLoad = ({ url }) => {
	const userCode = url.searchParams.get('code');

	return {
		userCode,
		currentUrl: url.href
	};
};
