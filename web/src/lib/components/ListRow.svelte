<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		href?: string;
		children: Snippet;
		trailing?: Snippet;
		selected?: boolean;
	}

	let { href, children, trailing, selected = false }: Props = $props();
</script>

<li class="item">
	{#if href}
		<a class="row" {href} class:selected>
			<span class="main">{@render children()}</span>
			{#if trailing}<span class="trailing">{@render trailing()}</span>{/if}
		</a>
	{:else}
		<div class="row" class:selected>
			<span class="main">{@render children()}</span>
			{#if trailing}<span class="trailing">{@render trailing()}</span>{/if}
		</div>
	{/if}
</li>

<style>
	.item {
		list-style: none;
	}

	/* Separator between rows. A sibling selector cannot match here because one
	   ListRow is one <li>; :first-child is resolved by the parent list. */
	.item .row {
		border-top: 1px solid var(--line);
	}

	.item:first-child .row {
		border-top: none;
	}

	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-4);
		padding: 10px var(--s-4);
		color: var(--text);
		text-decoration: none;
		line-height: 1.35;
		transition: background-color var(--dur-state) var(--ease);
	}

	a.row:hover {
		background: var(--surface-2);
		text-decoration: none;
	}

	.row.selected {
		background: var(--surface-2);
	}

	a.row:active {
		background: var(--surface-3);
	}

	.main {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		min-width: 0;
		flex: 1 1 auto;
	}

	.trailing {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		flex: 0 0 auto;
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}
</style>
