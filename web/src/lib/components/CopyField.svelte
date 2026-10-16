<script lang="ts">
	import { truncateMiddle } from '$lib/format';
	import Icon from './Icon.svelte';

	interface Props {
		value: string;
		display?: string;
		label?: string;
	}

	let { value, display, label = 'Copy' }: Props = $props();

	type Status = 'idle' | 'copied' | 'failed';
	let status = $state<Status>('idle');

	async function copy() {
		try {
			await navigator.clipboard.writeText(value);
			status = 'copied';
		} catch {
			status = 'failed';
		}
		setTimeout(() => (status = 'idle'), 2000);
	}

	const shown = $derived(display ?? truncateMiddle(value));
</script>

<div class="copyfield">
	<code class="value" title={value}>{shown}</code>
	<button type="button" class="action" onclick={copy}>
		{#if status === 'copied'}
			<Icon name="check" size={14} />
		{:else}
			<Icon name="copy" size={14} />
		{/if}
		<span class="text" aria-live="polite">
			{#if status === 'copied'}Copied{:else if status === 'failed'}Copy failed{:else}{label}{/if}
		</span>
	</button>
</div>

<style>
	.copyfield {
		display: flex;
		align-items: center;
		gap: var(--s-2);
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		padding: 3px 3px 3px 11px;
		max-width: 100%;
	}

	.value {
		flex: 1 1 auto;
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		color: var(--text);
		font-size: var(--fs-secondary);
	}

	.action {
		flex: 0 0 auto;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 9px;
		border-radius: var(--r-sm);
		border: 1px solid var(--line);
		background: var(--surface-2);
		color: var(--text-muted);
		font-family: inherit;
		font-size: var(--fs-meta);
		font-weight: 500;
		cursor: pointer;
		transition:
			background-color var(--dur-state) var(--ease),
			color var(--dur-state) var(--ease);
	}

	.action:hover {
		background: var(--surface-1);
		color: var(--text);
	}

	.action:active {
		background: var(--surface-3);
	}

	.text {
		white-space: nowrap;
	}
</style>
