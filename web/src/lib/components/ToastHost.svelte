<script lang="ts">
	import { toasts } from '$lib/toast.svelte';
	import Toast from './Toast.svelte';
</script>

<div class="toasts" aria-live="polite" aria-relevant="additions">
	{#each toasts.items as toast (toast.id)}
		<Toast {toast} ondismiss={(id) => toasts.dismiss(id)} />
	{/each}
</div>

<style>
	.toasts {
		position: fixed;
		right: var(--s-6);
		bottom: var(--s-6);
		z-index: 60;
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
		pointer-events: none;
	}

	.toasts > :global(*) {
		pointer-events: auto;
	}

	@media (max-width: 768px) {
		.toasts {
			right: var(--s-4);
			left: var(--s-4);
			bottom: var(--s-4);
		}
	}
</style>
