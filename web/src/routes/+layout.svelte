<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { session } from '$lib/session.svelte';
	import ToastHost from '$lib/components/ToastHost.svelte';

	let { children } = $props();

	onMount(() => {
		session.load();
	});
</script>

<a class="skip-link" href="#content">Skip to content</a>

{@render children()}

<ToastHost />

<style>
	.skip-link {
		position: fixed;
		top: var(--s-2);
		left: var(--s-2);
		z-index: 100;
		padding: 8px 12px;
		background: var(--surface-2);
		border: 1px solid var(--line-strong);
		border-radius: var(--r-sm);
		color: var(--text);
		font-size: var(--fs-secondary);
		text-decoration: none;
		transform: translateY(-200%);
		transition: transform var(--dur-state) var(--ease);
	}

	.skip-link:focus-visible {
		transform: translateY(0);
	}
</style>
