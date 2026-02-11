import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ parent, url }) => {
	const { user } = await parent();

	if (user?.role !== 'admin') {
		redirect(302, `/login?redirect=${encodeURIComponent(url.pathname)}`);
	}

	return { isAdmin: true };
};
