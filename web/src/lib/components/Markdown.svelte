<script lang="ts">
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';

	interface Props {
		text: string;
	}

	let { text }: Props = $props();

	marked.setOptions({ gfm: true, breaks: false });

	// Sanitized: README content is untrusted input from the repository.
	const html = $derived(
		DOMPurify.sanitize(marked.parse(text, { async: false }) as string, {
			USE_PROFILES: { html: true }
		})
	);
</script>

<div class="markdown">
	<!-- eslint-disable-next-line svelte/no-at-html-tags -- sanitized with DOMPurify above -->
	{@html html}
</div>

<style>
	.markdown {
		max-width: var(--prose-max);
		color: var(--text);
		line-height: 1.6;
		font-size: var(--fs-body);
		overflow-wrap: anywhere;
	}

	.markdown :global(h1),
	.markdown :global(h2),
	.markdown :global(h3),
	.markdown :global(h4) {
		font-weight: 600;
		line-height: 1.25;
		margin: var(--s-6) 0 var(--s-3);
	}

	.markdown :global(h1) {
		font-size: var(--fs-h1);
	}

	.markdown :global(h2) {
		font-size: var(--fs-h2);
		padding-bottom: var(--s-2);
		border-bottom: 1px solid var(--line);
	}

	.markdown :global(h3) {
		font-size: var(--fs-label);
	}

	.markdown :global(h4) {
		font-size: var(--fs-body);
		color: var(--text-muted);
	}

	.markdown :global(p) {
		margin: 0 0 var(--s-4);
	}

	.markdown :global(a) {
		color: var(--purple-hi);
	}

	.markdown :global(ul),
	.markdown :global(ol) {
		margin: 0 0 var(--s-4);
		padding-left: var(--s-6);
	}

	.markdown :global(li) {
		margin-bottom: var(--s-1);
	}

	.markdown :global(code) {
		font-family: var(--font-mono);
		font-size: 0.8125em;
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		padding: 1px 5px;
	}

	.markdown :global(pre) {
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		padding: var(--s-4);
		overflow-x: auto;
		margin: 0 0 var(--s-4);
	}

	.markdown :global(pre code) {
		background: none;
		border: none;
		padding: 0;
		font-size: var(--fs-secondary);
		line-height: 1.6;
	}

	.markdown :global(blockquote) {
		margin: 0 0 var(--s-4);
		padding: 0 var(--s-4);
		border-left: 2px solid var(--line-strong);
		color: var(--text-muted);
	}

	.markdown :global(hr) {
		border: none;
		border-top: 1px solid var(--line);
		margin: var(--s-6) 0;
	}

	.markdown :global(table) {
		width: 100%;
		border-collapse: collapse;
		margin: 0 0 var(--s-4);
		font-size: var(--fs-secondary);
	}

	.markdown :global(th),
	.markdown :global(td) {
		border: 1px solid var(--line);
		padding: 6px 10px;
		text-align: left;
	}

	.markdown :global(th) {
		background: var(--surface-2);
		font-weight: 500;
	}

	.markdown :global(img) {
		max-width: 100%;
		border-radius: var(--r-md);
	}

	.markdown :global(input[type='checkbox']) {
		margin-right: 6px;
	}
</style>
