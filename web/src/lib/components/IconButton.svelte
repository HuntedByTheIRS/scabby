<script lang="ts">
	import type { IconName } from '$lib/icons';
	import Icon from './Icon.svelte';
	import Spinner from './Spinner.svelte';

	interface Props {
		name: IconName;
		label: string;
		variant?: 'default' | 'danger';
		size?: 'sm' | 'md';
		loading?: boolean;
		disabled?: boolean;
		href?: string;
		type?: 'button' | 'submit';
		onclick?: (event: MouseEvent) => void;
	}

	let {
		name,
		label,
		variant = 'default',
		size = 'md',
		loading = false,
		disabled = false,
		href,
		type = 'button',
		onclick
	}: Props = $props();

	const blocked = $derived(disabled || loading);
	const glyph = $derived(size === 'sm' ? 14 : 16);
</script>

{#if href && !blocked}
	<a
		class="icon-btn {variant} {size}"
		{href}
		aria-label={label}
		data-tooltip={label}
		title={label}
	>
		<Icon {name} size={glyph} />
	</a>
{:else}
	<button
		class="icon-btn {variant} {size}"
		{type}
		aria-label={label}
		data-tooltip={label}
		title={label}
		disabled={blocked}
		aria-busy={loading ? 'true' : undefined}
		{onclick}
	>
		{#if loading}<Spinner size={glyph} />{:else}<Icon {name} size={glyph} />{/if}
	</button>
{/if}

<style>
	.icon-btn {
		position: relative;
		display: inline-grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border-radius: var(--r-sm);
		border: 1px solid transparent;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition:
			background-color var(--dur-state) var(--ease),
			color var(--dur-state) var(--ease),
			border-color var(--dur-state) var(--ease);
	}

	.icon-btn.sm {
		width: 26px;
		height: 26px;
	}

	.icon-btn:hover:not(:disabled) {
		background: var(--surface-2);
		color: var(--text);
	}

	.icon-btn:active:not(:disabled) {
		background: var(--surface-3);
	}

	.icon-btn.danger:hover:not(:disabled) {
		background: var(--danger-bg);
		color: var(--danger);
	}

	.icon-btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	/* Tooltip on hover and focus */
	.icon-btn::after {
		content: attr(data-tooltip);
		position: absolute;
		bottom: calc(100% + 6px);
		left: 50%;
		transform: translateX(-50%) translateY(2px);
		background: var(--surface-2);
		color: var(--text);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		padding: 3px 7px;
		font-size: var(--fs-meta);
		white-space: nowrap;
		opacity: 0;
		pointer-events: none;
		transition: opacity var(--dur-state) var(--ease), transform var(--dur-state) var(--ease);
		z-index: 30;
	}

	.icon-btn:hover::after,
	.icon-btn:focus-visible::after {
		opacity: 1;
		transform: translateX(-50%) translateY(0);
	}

	@media (prefers-reduced-motion: reduce) {
		.icon-btn::after {
			transition: opacity var(--dur-state) var(--ease);
			transform: translateX(-50%);
		}
	}
</style>
