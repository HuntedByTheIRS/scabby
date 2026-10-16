<script lang="ts">
	import type { IconName } from '$lib/icons';
	import Icon from './Icon.svelte';

	interface Props {
		variant?: 'neutral' | 'lilac' | 'ok' | 'danger' | 'warn' | 'purple';
		icon?: IconName;
		mono?: boolean;
		title?: string;
		children?: import('svelte').Snippet;
	}

	let { variant = 'neutral', icon, mono = false, title, children }: Props = $props();
</script>

<span class="badge {variant}" class:mono {title}>
	{#if icon}<Icon name={icon} size={12} />{/if}
	<span class="text">{@render children?.()}</span>
</span>

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: var(--r-pill);
		border: 1px solid var(--line);
		background: var(--surface-3);
		color: var(--text-muted);
		font-size: var(--fs-meta);
		font-weight: 500;
		line-height: 1.5;
		white-space: nowrap;
	}

	.mono {
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
	}

	.text {
		display: inline-block;
	}

	.neutral {
		background: var(--surface-3);
		color: var(--text-muted);
		border-color: var(--line);
	}

	.lilac {
		background: var(--purple-bg);
		color: var(--lilac);
		border-color: var(--line-strong);
	}

	.purple {
		background: var(--purple-bg-strong);
		color: var(--lilac);
		border-color: var(--purple);
	}

	.ok {
		background: var(--ok-bg);
		color: var(--ok);
		border-color: var(--ok);
	}

	.danger {
		background: var(--danger-bg);
		color: var(--danger);
		border-color: var(--danger);
	}

	.warn {
		background: var(--warn-bg);
		color: var(--warn);
		border-color: var(--warn);
	}
</style>
