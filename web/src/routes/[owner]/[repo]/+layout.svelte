<script lang="ts">
	import { page } from '$app/state';
	import RepoHeader from '$lib/components/RepoHeader.svelte';
	import StatePanel from '$lib/components/StatePanel.svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import type { LayoutProps } from './$types';

	let { data, children }: LayoutProps = $props();

	const active = $derived.by((): 'code' | 'commits' | 'issues' | 'settings' => {
		const path = page.url.pathname as string;
		if (path.includes('/commits')) return 'commits';
		if (path.includes('/issues')) return 'issues';
		if (path.includes('/settings')) return 'settings';
		return 'code';
	});
</script>

<TopBar />

<div class="shell">
	<main id="content" tabindex="-1">
		{#if data.repo}
			<RepoHeader repo={data.repo} {active} />
			{@render children()}
		{:else}
			<StatePanel error={data.repoError} skeletonRows={0} />
		{/if}
	</main>
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
		padding: var(--s-4) var(--s-6) var(--s-12);
	}
</style>
