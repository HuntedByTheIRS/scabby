<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { IconName } from '$lib/icons';
	import Icon from './Icon.svelte';
	import Spinner from './Spinner.svelte';

	interface Props {
		variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
		size?: 'sm' | 'md';
		type?: 'button' | 'submit' | 'reset';
		loading?: boolean;
		disabled?: boolean;
		href?: string;
		full?: boolean;
		icon?: IconName;
		iconAfter?: IconName;
		title?: string;
		children?: Snippet;
		onclick?: (event: MouseEvent) => void;
	}

	let {
		variant = 'primary',
		size = 'md',
		type = 'button',
		loading = false,
		disabled = false,
		href,
		full = false,
		icon,
		iconAfter,
		title,
		children,
		onclick
	}: Props = $props();

	const blocked = $derived(disabled || loading);
	const glyph = $derived(size === 'sm' ? 14 : 15);
</script>

{#snippet content()}
	{#if loading}
		<span class="busy" aria-hidden="true"><Spinner size={glyph} /></span>
	{/if}
	<span class="inner" class:hidden={loading}>
		{#if icon}<Icon name={icon} size={glyph} />{/if}
		{#if children}<span class="label">{@render children()}</span>{/if}
		{#if iconAfter}<Icon name={iconAfter} size={glyph} />{/if}
	</span>
{/snippet}

{#if href && !blocked}
	<a
		class="btn {variant} {size}"
		class:full
		{href}
		{title}
		data-loading={loading ? 'true' : undefined}
	>
		{@render content()}
	</a>
{:else if href}
	<span
		class="btn {variant} {size}"
		class:full
		class:blocked
		role="link"
		aria-disabled="true"
		{title}>{@render content()}</span
	>
{:else}
	<button
		class="btn {variant} {size}"
		class:full
		{type}
		{title}
		disabled={blocked}
		aria-busy={loading ? 'true' : undefined}
		data-loading={loading ? 'true' : undefined}
		{onclick}
	>
		{@render content()}
	</button>
{/if}

<style>
	.btn {
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: var(--s-2);
		border-radius: var(--r-sm);
		border: 1px solid transparent;
		font-weight: 500;
		line-height: 1;
		cursor: pointer;
		text-decoration: none;
		white-space: nowrap;
		transition:
			background-color var(--dur-state) var(--ease),
			border-color var(--dur-state) var(--ease),
			color var(--dur-state) var(--ease);
	}

	.sm {
		padding: 5px 10px;
		font-size: var(--fs-secondary);
	}

	.md {
		padding: 7px 14px;
		font-size: var(--fs-body);
	}

	.full {
		width: 100%;
	}

	.inner {
		display: inline-flex;
		align-items: center;
		gap: var(--s-2);
	}

	.inner.hidden {
		visibility: hidden;
	}

	.busy {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
	}

	.label {
		display: inline-block;
	}

	.primary {
		background: var(--purple);
		color: var(--void);
	}

	.primary:hover:not(.blocked) {
		background: var(--purple-hi);
	}

	.primary:active:not(.blocked) {
		background: var(--purple-deep);
		color: var(--text);
	}

	.secondary {
		background: var(--surface-2);
		border-color: var(--line);
		color: var(--text);
	}

	.secondary:hover:not(.blocked) {
		background: var(--surface-3);
		border-color: var(--line-strong);
	}

	.secondary:active:not(.blocked) {
		background: var(--surface-1);
	}

	.ghost {
		background: transparent;
		color: var(--text-muted);
	}

	.ghost:hover:not(.blocked) {
		background: var(--surface-2);
		color: var(--text);
	}

	.ghost:active:not(.blocked) {
		background: var(--surface-3);
	}

	.danger {
		background: var(--danger);
		color: var(--void);
	}

	.danger:hover:not(.blocked) {
		background: var(--danger);
		filter: brightness(1.12);
	}

	.danger:active:not(.blocked) {
		filter: brightness(0.92);
	}

	.btn:disabled,
	.btn.blocked {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
