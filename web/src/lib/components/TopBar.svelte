<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { session } from '$lib/session.svelte';
	import { toasts } from '$lib/toast.svelte';
	import { isCurrent } from '$lib/nav';
	import Avatar from './Avatar.svelte';
	import Button from './Button.svelte';
	import Icon from './Icon.svelte';
	import IconButton from './IconButton.svelte';

	interface Props {
		/** Shown only under 768px, where the app sidebar becomes a drawer. */
		onmenu?: () => void;
		showMenu?: boolean;
	}

	let { onmenu, showMenu = false }: Props = $props();

	let menuOpen = $state(false);
	const path = $derived(page.url.pathname as string);

	async function signOut() {
		menuOpen = false;
		await session.logout();
		toasts.info('Signed out.');
		await goto('/');
	}

	function onWindowClick(event: MouseEvent) {
		const target = event.target as HTMLElement | null;
		if (menuOpen && target && !target.closest('.menu')) menuOpen = false;
	}
</script>

<svelte:window onclick={onWindowClick} />

<header class="topbar">
	<div class="inner">
		{#if showMenu}
			<span class="menu-slot">
				<IconButton name="branch" label="Open navigation" onclick={() => onmenu?.()} />
			</span>
		{/if}
		<a class="brand" href="/" aria-label="Scabby home">
			<span class="mark" aria-hidden="true"><Icon name="repo" size={16} /></span>
			<span class="wordmark mono">scabby</span>
		</a>

		<nav class="links" aria-label="Primary">
			<a href="/dashboard" aria-current={isCurrent(path, '/dashboard') ? 'page' : undefined}>
				Dashboard
			</a>
			<a href="/new" aria-current={isCurrent(path, '/new') ? 'page' : undefined}>New</a>
		</nav>

		<div class="account">
			{#if session.user}
				<div class="menu">
					<button
						type="button"
						class="avatar-btn"
						aria-haspopup="menu"
						aria-expanded={menuOpen}
						aria-label="Account menu for {session.user.username}"
						onclick={() => (menuOpen = !menuOpen)}
					>
						<Avatar name={session.user.username} src={session.user.avatar_url} size={24} />
					</button>
					{#if menuOpen}
						<div class="pop" role="menu">
							<span class="who">
								<strong>{session.user.display_name ?? session.user.username}</strong>
								<span class="faint">@{session.user.username}</span>
							</span>
							<a role="menuitem" href="/{session.user.username}">Your profile</a>
							<a role="menuitem" href="/settings">Settings</a>
							<button role="menuitem" type="button" onclick={signOut}>Sign out</button>
						</div>
					{/if}
				</div>
			{:else if session.loading}
				<span class="ghost-avatar" aria-hidden="true"></span>
			{:else}
				<Button size="sm" variant="secondary" href="/login">Sign in</Button>
			{/if}
		</div>
	</div>
</header>

<style>
	.topbar {
		position: sticky;
		top: 0;
		z-index: 40;
		height: var(--topbar-h);
		background: var(--base);
		border-bottom: 1px solid var(--line);
	}

	.inner {
		display: flex;
		align-items: center;
		gap: var(--s-4);
		max-width: var(--content-max);
		margin: 0 auto;
		height: 100%;
		padding: 0 var(--s-6);
	}

	.menu-slot {
		display: none;
	}

	.brand {
		display: inline-flex;
		align-items: center;
		gap: var(--s-2);
		color: var(--text);
		text-decoration: none;
	}

	.brand:hover {
		text-decoration: none;
	}

	.mark {
		display: inline-grid;
		place-items: center;
		width: 26px;
		height: 26px;
		border-radius: var(--r-sm);
		background: var(--purple-bg);
		color: var(--purple-hi);
		border: 1px solid var(--line-strong);
	}

	.wordmark {
		font-size: var(--fs-label);
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	.links {
		display: flex;
		align-items: center;
		gap: var(--s-1);
		flex: 1 1 auto;
	}

	.links a {
		padding: 5px 9px;
		border-radius: var(--r-sm);
		color: var(--text-muted);
		font-size: var(--fs-secondary);
		text-decoration: none;
	}

	.links a:hover {
		background: var(--surface-2);
		color: var(--text);
		text-decoration: none;
	}

	.links a[aria-current='page'] {
		color: var(--lilac);
		background: var(--purple-bg);
	}

	.account {
		position: relative;
		flex: 0 0 auto;
	}

	.menu {
		position: relative;
	}

	.avatar-btn {
		display: inline-flex;
		padding: 0;
		border: none;
		background: none;
		border-radius: var(--r-pill);
		cursor: pointer;
	}

	.ghost-avatar {
		display: inline-block;
		width: 24px;
		height: 24px;
		border-radius: var(--r-pill);
		background: var(--surface-2);
	}

	.pop {
		position: absolute;
		right: 0;
		top: calc(100% + 8px);
		min-width: 178px;
		display: flex;
		flex-direction: column;
		padding: var(--s-1);
		background: var(--surface-2);
		border: 1px solid var(--line-strong);
		border-radius: var(--r-md);
		box-shadow: var(--shadow-pop);
	}

	.who {
		display: flex;
		flex-direction: column;
		padding: var(--s-2) var(--s-3);
		font-size: var(--fs-secondary);
		border-bottom: 1px solid var(--line);
		margin-bottom: var(--s-1);
	}

	.pop a,
	.pop button {
		display: block;
		width: 100%;
		text-align: left;
		padding: 7px 9px;
		border: none;
		background: none;
		border-radius: var(--r-sm);
		color: var(--text);
		font-size: var(--fs-secondary);
		font-family: inherit;
		text-decoration: none;
		cursor: pointer;
	}

	.pop a:hover,
	.pop button:hover {
		background: var(--surface-3);
		text-decoration: none;
	}

	@media (max-width: 767px) {
		.inner {
			padding: 0 var(--s-4);
		}

		.menu-slot {
			display: inline-flex;
		}
	}
</style>
