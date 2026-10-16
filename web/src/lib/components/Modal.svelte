<script lang="ts">
	import type { Snippet } from 'svelte';
	import { uid } from '$lib/id';
	import Button from './Button.svelte';
	import IconButton from './IconButton.svelte';

	interface Props {
		open: boolean;
		title: string;
		description?: string;
		confirmLabel?: string;
		cancelLabel?: string;
		danger?: boolean;
		loading?: boolean;
		onconfirm?: () => void;
		onclose: () => void;
		children?: Snippet;
	}

	let {
		open,
		title,
		description,
		confirmLabel = 'Confirm',
		cancelLabel = 'Cancel',
		danger = false,
		loading = false,
		onconfirm,
		onclose,
		children
	}: Props = $props();

	let dialogEl = $state<HTMLDivElement | null>(null);
	let previouslyFocused: HTMLElement | null = null;
	const titleId = uid('modal-title');
	const descId = uid('modal-desc');

	const FOCUSABLE =
		'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';

	function focusables(): HTMLElement[] {
		if (!dialogEl) return [];
		return Array.from(dialogEl.querySelectorAll<HTMLElement>(FOCUSABLE));
	}

	// Trap focus while open, restore it on close.
	$effect(() => {
		if (!open) return;
		previouslyFocused = document.activeElement as HTMLElement | null;
		const el = dialogEl;
		if (el) {
			const items = focusables();
			(items[0] ?? el).focus();
		}
		return () => {
			previouslyFocused?.focus?.();
		};
	});

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			onclose();
			return;
		}
		if (event.key !== 'Tab' || !dialogEl) return;
		const items = focusables();
		if (items.length === 0) {
			event.preventDefault();
			return;
		}
		const first = items[0];
		const last = items[items.length - 1];
		const active = document.activeElement as HTMLElement | null;
		const inside = active !== null && dialogEl.contains(active);
		if (event.shiftKey && (!inside || active === first)) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && (!inside || active === last)) {
			event.preventDefault();
			first.focus();
		}
	}
</script>

<svelte:window onkeydown={open ? onKeydown : undefined} />

{#if open}
	<div class="overlay">
		<button
			type="button"
			class="backdrop"
			aria-label="Close dialog"
			tabindex="-1"
			onclick={onclose}
		></button>
		<div
			class="dialog"
			bind:this={dialogEl}
			role="alertdialog"
			aria-modal="true"
			aria-labelledby={titleId}
			aria-describedby={description ? descId : undefined}
			tabindex="-1"
		>
			<header class="head">
				<h2 id={titleId}>{title}</h2>
				<IconButton name="close" label="Close dialog" size="sm" onclick={onclose} />
			</header>
			{#if description}
				<p class="desc" id={descId}>{description}</p>
			{/if}
			{#if children}
				<div class="body">{@render children()}</div>
			{/if}
			<footer class="foot">
				<Button variant="secondary" onclick={onclose} disabled={loading}>{cancelLabel}</Button>
				{#if onconfirm}
					<Button
						variant={danger ? 'danger' : 'primary'}
						onclick={onconfirm}
						loading={loading}
					>
						{confirmLabel}
					</Button>
				{/if}
			</footer>
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		z-index: 70;
		display: grid;
		place-items: center;
		padding: var(--s-4);
	}

	.backdrop {
		position: absolute;
		inset: 0;
		border: none;
		padding: 0;
		background: var(--scrim);
		cursor: default;
	}

	.dialog {
		position: relative;
		width: min(440px, 100%);
		background: var(--surface-1);
		border: 1px solid var(--line-strong);
		border-radius: var(--r-lg);
		box-shadow: var(--shadow-pop);
		animation: dialog-in var(--dur-enter) var(--ease);
	}

	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-4);
		padding: var(--s-4) var(--s-4) var(--s-3);
	}

	h2 {
		font-size: var(--fs-h2);
		font-weight: 600;
		color: var(--text);
	}

	.desc {
		padding: 0 var(--s-4);
		color: var(--text-muted);
		font-size: var(--fs-body);
	}

	.body {
		padding: var(--s-4);
	}

	.foot {
		display: flex;
		justify-content: flex-end;
		gap: var(--s-2);
		padding: var(--s-3) var(--s-4) var(--s-4);
	}

	@keyframes dialog-in {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.dialog {
			animation: none;
		}
	}
</style>
