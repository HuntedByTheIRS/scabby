<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title?: string;
		description?: string;
		padded?: boolean;
		children?: Snippet;
		actions?: Snippet;
		footer?: Snippet;
	}

	let { title, description, padded = true, children, actions, footer }: Props = $props();
</script>

<section class="card">
	{#if title || actions}
		<header class="head">
			<div class="titles">
				{#if title}<h3 class="title">{title}</h3>{/if}
				{#if description}<p class="desc">{description}</p>{/if}
			</div>
			{#if actions}<div class="actions">{@render actions()}</div>{/if}
		</header>
	{/if}
	<div class="body" class:padded>
		{#if children}{@render children()}{/if}
	</div>
	{#if footer}
		<footer class="foot">{@render footer()}</footer>
	{/if}
</section>

<style>
	.card {
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		overflow: hidden;
	}

	.head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--s-4);
		padding: var(--s-4) var(--s-4) var(--s-3);
		border-bottom: 1px solid var(--line);
	}

	.titles {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.title {
		font-size: var(--fs-label);
		font-weight: 600;
		color: var(--text);
	}

	.desc {
		font-size: var(--fs-secondary);
		color: var(--text-muted);
	}

	.actions {
		display: flex;
		align-items: center;
		gap: var(--s-2);
		flex: 0 0 auto;
	}

	.body.padded {
		padding: var(--s-4);
	}

	.foot {
		border-top: 1px solid var(--line);
		padding: var(--s-3) var(--s-4);
		background: var(--surface-1);
	}
</style>
