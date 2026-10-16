<script lang="ts">
	import { SITE } from '$lib/site';
	import { session } from '$lib/session.svelte';
	import Button from './Button.svelte';
	import Icon from './Icon.svelte';
</script>

<header class="public-header">
	<div class="inner">
		<a class="brand" href="/" aria-label="Scabby home">
			<span class="mark" aria-hidden="true"><Icon name="repo" size={16} /></span>
			<span class="wordmark mono">scabby</span>
		</a>
		<nav aria-label="Public">
			<a href={SITE.docs}>Docs</a>
			<a href={SITE.apiReference}>API</a>
			<a href={SITE.repo}>Source</a>
		</nav>
		<div class="actions">
			{#if session.user}
				<Button size="sm" variant="secondary" href="/dashboard">Dashboard</Button>
			{:else}
				<Button size="sm" variant="ghost" href="/login">Sign in</Button>
				<Button size="sm" variant="secondary" href="/register">Create account</Button>
			{/if}
		</div>
	</div>
</header>

<style>
	.public-header {
		position: relative;
		z-index: 2;
	}

	.inner {
		display: flex;
		align-items: center;
		gap: var(--s-6);
		max-width: var(--content-max);
		margin: 0 auto;
		padding: var(--s-6);
	}

	.brand {
		display: inline-flex;
		align-items: center;
		gap: var(--s-2);
		color: var(--text);
		text-decoration: none;
	}

	.brand:hover {
		text-decoration: none;
	}

	.mark {
		display: inline-grid;
		place-items: center;
		width: 28px;
		height: 28px;
		border-radius: var(--r-sm);
		background: var(--purple-bg);
		border: 1px solid var(--line-strong);
		color: var(--purple-hi);
	}

	.wordmark {
		font-size: var(--fs-label);
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	nav {
		display: flex;
		align-items: center;
		gap: var(--s-4);
		flex: 1 1 auto;
		margin-left: var(--s-4);
	}

	nav a {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
		text-decoration: none;
	}

	nav a:hover {
		color: var(--text);
		text-decoration: none;
	}

	.actions {
		display: flex;
		align-items: center;
		gap: var(--s-2);
	}

	@media (max-width: 767px) {
		.inner {
			padding: var(--s-4);
			gap: var(--s-3);
		}

		nav {
			display: none;
		}
	}
</style>
