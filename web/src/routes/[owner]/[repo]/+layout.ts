import { api, ApiError } from '$lib/api';
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ params }) => {
	try {
		const repo = await api.getRepo(params.owner, params.repo);
		return { repo, repoError: null };
	} catch (e) {
		const err = e as ApiError;
		return {
			repo: null,
			repoError: {
				status: typeof err.status === 'number' ? err.status : 0,
				message: err.message ?? 'Request failed'
			}
		};
	}
};
