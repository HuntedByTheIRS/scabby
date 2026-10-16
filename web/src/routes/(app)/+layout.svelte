<script lang="ts">
	import { goto } from '$app/navigation';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import { session } from '$lib/session.svelte';

	let { children } = $props();

	let navOpen = $state(false);

	// These routes are private: send signed-out visitors to the sign-in form.
	$effect(() => {
		if (session.loaded && !session.isAuthed) {
			goto('/login', { replaceState: true });
		}
	});
</script>

<TopBar showMenu onmenu={() => (navOpen = true)} />

<div class="shell">
	<Sidebar open={navOpen} onclose={() => (navOpen = false)} />
	<main id="content" tabindex="-1">{@render children()}</main>
</div>

<style>
	.shell {
		display: flex;
		align-items: stretch;
		min-height: calc(100vh - var(--topbar-h));
		background: var(--void);
	}

	main {
		flex: 1 1 auto;
		min-width: 0;
		max-width: var(--content-max);
		padding: var(--s-8) var(--s-6);
	}
</style>
