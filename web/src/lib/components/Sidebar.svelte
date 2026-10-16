<script lang="ts">
	import { page } from '$app/state';
	import { session } from '$lib/session.svelte';
	import { isCurrent } from '$lib/nav';
	import type { IconName } from '$lib/icons';
	import Icon from './Icon.svelte';

	interface Props {
		open?: boolean;
		onclose?: () => void;
	}

	let { open = false, onclose }: Props = $props();

	const links: { href: string; label: string; icon: IconName; requiresAuth: boolean }[] = [
		{ href: '/dashboard', label: 'Repositories', icon: 'repo', requiresAuth: true },
		{ href: '/new', label: 'New repository', icon: 'plus', requiresAuth: true },
		{ href: '/settings', label: 'Settings', icon: 'settings', requiresAuth: true }
	];

	const visible = $derived(links.filter((l) => !l.requiresAuth || session.isAuthed));
	const path = $derived(page.url.pathname as string);
</script>

<aside class="sidebar" class:open aria-label="Repository navigation">
	<nav>
		<ul>
			{#each visible as link (link.href)}
				<li>
					<a
						href={link.href}
						aria-current={isCurrent(path, link.href) ? 'page' : undefined}
						onclick={() => onclose?.()}
					>
						<Icon name={link.icon} size={16} />
						<span class="label">{link.label}</span>
					</a>
				</li>
			{/each}
		</ul>

		{#if session.user}
			<h2 class="section">Your work</h2>
			<ul>
				<li>
					<a
						href="/{session.user.username}"
						aria-current={isCurrent(path, `/${session.user.username}`) ? 'page' : undefined}
						onclick={() => onclose?.()}
					>
						<Icon name="user" size={16} />
						<span class="label">Your profile</span>
					</a>
				</li>
			</ul>
		{/if}
	</nav>
</aside>

{#if open}
	<button type="button" class="scrim" aria-label="Close navigation" onclick={() => onclose?.()}
	></button>
{/if}

<style>
	.sidebar {
		width: var(--sidebar-w);
		flex: 0 0 auto;
		background: var(--base);
		border-right: 1px solid var(--line);
		min-height: calc(100vh - var(--topbar-h));
		padding: var(--s-4) var(--s-3);
	}

	nav {
		position: sticky;
		top: calc(var(--topbar-h) + var(--s-4));
	}

	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	a {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		padding: 7px 10px;
		border-radius: var(--r-sm);
		color: var(--text-muted);
		font-size: var(--fs-secondary);
		text-decoration: none;
		white-space: nowrap;
		transition:
			background-color var(--dur-state) var(--ease),
			color var(--dur-state) var(--ease);
	}

	a:hover {
		background: var(--surface-2);
		color: var(--text);
		text-decoration: none;
	}

	a[aria-current='page'] {
		background: var(--purple-bg);
		color: var(--lilac);
	}

	.section {
		margin: var(--s-4) 0 var(--s-2);
		padding: 0 10px;
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
		font-weight: 500;
	}

	.scrim {
		display: none;
	}

	/* Icon rail under 1024px */
	@media (max-width: 1023px) {
		.sidebar {
			width: 56px;
			padding: var(--s-4) var(--s-2);
		}

		.label,
		.section {
			display: none;
		}

		a {
			justify-content: center;
			padding: 8px;
		}
	}

	/* Drawer under 768px */
	@media (max-width: 767px) {
		.sidebar {
			position: fixed;
			top: var(--topbar-h);
			bottom: 0;
			left: 0;
			width: var(--sidebar-w);
			transform: translateX(-100%);
			transition: transform var(--dur-enter) var(--ease);
			z-index: 50;
			overflow-y: auto;
		}

		.sidebar.open {
			transform: translateX(0);
		}

		.label,
		.section {
			display: block;
		}

		a {
			justify-content: flex-start;
			padding: 8px 10px;
		}

		.scrim {
			display: block;
			position: fixed;
			inset: var(--topbar-h) 0 0 0;
			border: none;
			padding: 0;
			background: var(--scrim);
			z-index: 45;
			cursor: pointer;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.sidebar {
			transition: none;
		}
	}
</style>
