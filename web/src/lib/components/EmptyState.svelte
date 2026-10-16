<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { IconName } from '$lib/icons';
	import Icon from './Icon.svelte';

	interface Props {
		title: string;
		description?: string;
		icon?: IconName;
		children?: Snippet;
	}

	let { title, description, icon, children }: Props = $props();
</script>

<div class="empty">
	{#if icon}
		<span class="glyph" aria-hidden="true"><Icon name={icon} size={20} /></span>
	{/if}
	<p class="title">{title}</p>
	{#if description}
		<p class="desc">{description}</p>
	{/if}
	{#if children}
		<div class="action">{@render children()}</div>
	{/if}
</div>

<style>
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--s-2);
		padding: var(--s-8) var(--s-4);
		text-align: center;
	}

	.glyph {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border-radius: var(--r-md);
		background: var(--surface-3);
		border: 1px solid var(--line);
		color: var(--text-muted);
		margin-bottom: var(--s-1);
	}

	.title {
		font-size: var(--fs-label);
		font-weight: 500;
		color: var(--text);
	}

	.desc {
		font-size: var(--fs-secondary);
		color: var(--text-muted);
		max-width: 46ch;
	}

	.action {
		margin-top: var(--s-3);
	}
</style>
