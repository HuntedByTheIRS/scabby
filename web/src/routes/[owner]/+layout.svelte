<script lang="ts">
	import TopBar from '$lib/components/TopBar.svelte';

	let { children } = $props();
</script>

<TopBar />

<div class="shell">
	<main id="content" tabindex="-1">{@render children()}</main>
</div>

<style>
	.shell {
		display: flex;
		min-height: calc(100vh - var(--topbar-h));
		background: var(--void);
	}

	main {
		flex: 1 1 auto;
		min-width: 0;
		width: 100%;
		max-width: var(--content-max);
		margin: 0 auto;
		padding: var(--s-8) var(--s-6);
	}
</style>
