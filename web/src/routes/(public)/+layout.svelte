<script lang="ts">
	import PublicFooter from '$lib/components/PublicFooter.svelte';
	import PublicHeader from '$lib/components/PublicHeader.svelte';

	let { children } = $props();
</script>

<div class="shell">
	<div class="glow" aria-hidden="true"></div>
	<PublicHeader />
	<main id="content" tabindex="-1">{@render children()}</main>
	<PublicFooter />
</div>

<style>
	.shell {
		position: relative;
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		background: var(--void);
		overflow-x: hidden;
	}

	/* The single ambient glow: behind the top third of the public shell. */
	.glow {
		position: absolute;
		top: -280px;
		left: 50%;
		width: 900px;
		height: 700px;
		transform: translateX(-50%);
		background: radial-gradient(closest-side, var(--glow), transparent 72%);
		pointer-events: none;
		z-index: 0;
	}

	main {
		position: relative;
		z-index: 1;
		flex: 1 1 auto;
		width: 100%;
		max-width: var(--content-max);
		margin: 0 auto;
		padding: 0 var(--s-6);
	}
</style>
