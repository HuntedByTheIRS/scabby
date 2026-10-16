<script lang="ts">
	import { SITE } from '$lib/site';
</script>

<footer class="public-footer">
	<div class="inner">
		<p class="wordmark mono">{SITE.name}</p>
		<nav aria-label="Footer">
			<a href={SITE.docs}>Docs</a>
			<a href={SITE.apiReference}>API reference</a>
			<a href={SITE.repo}>Source</a>
			<a href={SITE.licenseUrl}>{SITE.license}</a>
		</nav>
	</div>
</footer>

<style>
	.public-footer {
		position: relative;
		z-index: 1;
		border-top: 1px solid var(--line);
		margin-top: var(--s-16);
	}

	.inner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-4);
		max-width: var(--content-max);
		margin: 0 auto;
		padding: var(--s-6);
		flex-wrap: wrap;
	}

	.wordmark {
		font-size: var(--fs-secondary);
		color: var(--text-faint);
	}

	nav {
		display: flex;
		gap: var(--s-6);
		flex-wrap: wrap;
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
</style>
