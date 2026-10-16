<script lang="ts">
	import type { IconName } from '$lib/icons';
	import Icon from './Icon.svelte';

	interface TabItem {
		key: string;
		label: string;
		href: string;
		count?: number;
		icon?: IconName;
	}

	interface Props {
		items: TabItem[];
		active: string;
		label?: string;
	}

	let { items, active, label = 'Sections' }: Props = $props();
</script>

<nav class="tabs" aria-label={label}>
	<ul>
		{#each items as item (item.key)}
			<li>
				<a
					href={item.href}
					class:active={item.key === active}
					aria-current={item.key === active ? 'page' : undefined}
				>
					{#if item.icon}<Icon name={item.icon} size={15} />{/if}
					<span>{item.label}</span>
					{#if item.count !== undefined}<span class="count">{item.count}</span>{/if}
				</a>
			</li>
		{/each}
	</ul>
</nav>

<style>
	.tabs {
		border-bottom: 1px solid var(--line);
	}

	ul {
		display: flex;
		align-items: stretch;
		gap: var(--s-1);
		list-style: none;
		margin: 0;
		padding: 0;
		overflow-x: auto;
		scrollbar-width: none;
	}

	ul::-webkit-scrollbar {
		display: none;
	}

	li {
		flex: 0 0 auto;
	}

	a {
		display: inline-flex;
		align-items: center;
		gap: var(--s-2);
		padding: 10px 12px;
		color: var(--text-muted);
		font-size: var(--fs-body);
		text-decoration: none;
		border-bottom: 2px solid transparent;
		white-space: nowrap;
		transition:
			color var(--dur-state) var(--ease),
			border-color var(--dur-state) var(--ease),
			background-color var(--dur-state) var(--ease);
	}

	a:hover {
		color: var(--text);
		background: var(--surface-2);
		text-decoration: none;
	}

	a.active,
	a[aria-current='page'] {
		color: var(--lilac);
		border-bottom-color: var(--purple);
	}

	.count {
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		font-size: var(--fs-meta);
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-pill);
		padding: 0 6px;
		color: var(--text-muted);
	}

	a.active .count {
		background: var(--purple-bg);
		border-color: var(--line-strong);
		color: var(--lilac);
	}
</style>
