import { api, ApiError, type User } from './api';

class SessionStore {
	user = $state<User | null>(null);
	loading = $state(false);
	loaded = $state(false);
	error = $state<string | null>(null);

	get isAuthed() {
		return this.user !== null;
	}

	/** Load the signed-in user once; safe to call repeatedly. */
	async load(force = false) {
		if (this.loaded && !force) return;
		if (this.loading) return;
		this.loading = true;
		this.error = null;
		try {
			const { user } = await api.getSession();
			this.user = user;
		} catch (e) {
			// 401 is the normal signed-out answer, not an error to surface.
			if (e instanceof ApiError && e.status !== 401) {
				this.error = e.message;
			}
			this.user = null;
		} finally {
			this.loading = false;
			this.loaded = true;
		}
	}

	async login(username_or_email: string, password: string) {
		const { user } = await api.login({ username_or_email, password });
		this.user = user;
		this.loaded = true;
		return user;
	}

	async register(username: string, email: string, password: string) {
		const { user } = await api.register({ username, email, password });
		this.user = user;
		this.loaded = true;
		return user;
	}

	async logout() {
		try {
			await api.logout();
		} finally {
			this.user = null;
			this.loaded = true;
		}
	}

	/** Reflect a profile edit made elsewhere (settings -> profile). */
	set(user: User | null) {
		this.user = user;
	}
}

export const session = new SessionStore();
