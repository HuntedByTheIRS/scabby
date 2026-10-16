<script lang="ts">
	import Icon from './Icon.svelte';

	export interface Sample {
		language: string;
		code: string;
	}

	interface Props {
		samples: Sample[];
		title?: string;
	}

	let { samples, title }: Props = $props();

	let selected = $state(0);
	let copied = $state(false);

	const current = $derived(samples[selected] ?? samples[0]);

	async function copy() {
		try {
			await navigator.clipboard.writeText(current.code);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		} catch {
			copied = false;
		}
	}
</script>

<figure class="sample">
	<figcaption>
		{#if title}<span class="title">{title}</span>{/if}
		<div class="langs" role="group" aria-label="Sample language">
			{#each samples as sample, index (sample.language)}
				<button
					type="button"
					class="lang mono"
					class:on={index === selected}
					aria-pressed={index === selected}
					onclick={() => (selected = index)}
				>
					{sample.language}
				</button>
			{/each}
		</div>
		<button type="button" class="copy" onclick={copy}>
			<Icon name={copied ? 'check' : 'copy'} size={14} />
			<span aria-live="polite">{copied ? 'Copied' : 'Copy'}</span>
		</button>
	</figcaption>
	<pre><code class="mono">{current.code}</code></pre>
</figure>

<style>
	.sample {
		margin: 0;
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		overflow: hidden;
	}

	figcaption {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		padding: 6px 8px 6px var(--s-4);
		border-bottom: 1px solid var(--line);
		background: var(--base);
	}

	.title {
		font-size: var(--fs-meta);
		color: var(--text-faint);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		flex: 1 1 auto;
	}

	.langs {
		display: inline-flex;
		gap: 2px;
		flex: 0 0 auto;
	}

	.lang {
		border: none;
		background: none;
		color: var(--text-faint);
		font-size: var(--fs-meta);
		padding: 3px 7px;
		border-radius: var(--r-sm);
		cursor: pointer;
	}

	.lang:hover {
		color: var(--text-muted);
		background: var(--surface-2);
	}

	.lang.on {
		background: var(--purple-bg);
		color: var(--lilac);
	}

	.copy {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		flex: 0 0 auto;
		border: 1px solid var(--line);
		background: var(--surface-2);
		color: var(--text-muted);
		font-family: inherit;
		font-size: var(--fs-meta);
		padding: 3px 8px;
		border-radius: var(--r-sm);
		cursor: pointer;
	}

	.copy:hover {
		color: var(--text);
		background: var(--surface-3);
	}

	pre {
		margin: 0;
		padding: var(--s-4);
		overflow-x: auto;
		font-size: var(--fs-secondary);
		line-height: 1.65;
	}

	code {
		color: var(--text);
	}
</style>
