<script lang="ts">
	import type { Toast } from '$lib/toast.svelte';
	import Icon from './Icon.svelte';

	interface Props {
		toast: Toast;
		ondismiss: (id: number) => void;
	}

	let { toast, ondismiss }: Props = $props();

	const icon = $derived(
		toast.kind === 'success' ? 'check' : toast.kind === 'error' ? 'alert' : 'book'
	);
</script>

<div class="toast {toast.kind}" role="status">
	<span class="glyph" aria-hidden="true"><Icon name={icon} size={15} /></span>
	<p class="message">{toast.message}</p>
	<button
		type="button"
		class="dismiss"
		aria-label="Dismiss notification"
		onclick={() => ondismiss(toast.id)}
	>
		<Icon name="close" size={14} />
	</button>
</div>

<style>
	.toast {
		display: flex;
		align-items: flex-start;
		gap: var(--s-3);
		min-width: 280px;
		max-width: 380px;
		padding: 10px 10px 10px var(--s-3);
		background: var(--surface-2);
		border: 1px solid var(--line-strong);
		border-radius: var(--r-md);
		box-shadow: var(--shadow-pop);
		animation: toast-in var(--dur-enter) var(--ease);
	}

	.glyph {
		display: inline-flex;
		margin-top: 1px;
		color: var(--text-muted);
	}

	.success .glyph {
		color: var(--ok);
	}

	.error .glyph {
		color: var(--danger);
	}

	.message {
		flex: 1 1 auto;
		font-size: var(--fs-secondary);
		color: var(--text);
		line-height: 1.4;
		overflow-wrap: anywhere;
	}

	.dismiss {
		flex: 0 0 auto;
		display: inline-grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border-radius: var(--r-sm);
		border: none;
		background: transparent;
		color: var(--text-faint);
		cursor: pointer;
	}

	.dismiss:hover {
		background: var(--surface-3);
		color: var(--text);
	}

	@keyframes toast-in {
		from {
			opacity: 0;
			transform: translateY(6px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.toast {
			animation: none;
		}
	}
</style>
