<script lang="ts">
	import Icon from './Icon.svelte';

	interface Props {
		page: number;
		/** `null` when the server did not count the collection. */
		totalPages: number | null;
		/** `null` when the server did not count the collection: the commit list sends `null`
		 *  for both rather than walk a large history for an exact page count. */
		total: number | null;
		perPage: number;
		/** Items on the page being shown, so an uncounted list can still say whether a further
		 *  page exists: a full page of results is the only hint the server gives. */
		count?: number;
		hrefFor: (page: number) => string;
	}

	let { page, totalPages, total, perPage, count = 0, hrefFor }: Props = $props();

	const from = $derived(total === 0 ? 0 : (page - 1) * perPage + 1);
	const to = $derived(
		total === null ? from + Math.max(count - 1, 0) : Math.min(page * perPage, total)
	);
	// An uncounted list offers a next page exactly when this page came back full.
	const hasNext = $derived(totalPages === null ? count >= perPage : page < totalPages);
</script>

<div class="pagination">
	<p class="summary mono" aria-live="polite">
		{from}–{to}{#if total !== null} of {total}{/if}
	</p>
	<div class="controls">
		{#if page > 1}
			<a class="step" href={hrefFor(page - 1)} rel="prev">
				<Icon name="chevron-left" size={14} />
				<span>Previous</span>
			</a>
		{:else}
			<span class="step disabled" aria-disabled="true">
				<Icon name="chevron-left" size={14} />
				<span>Previous</span>
			</span>
		{/if}

		<span class="position mono">
			{#if totalPages === null}page {page}{:else}page {page} / {totalPages || 1}{/if}
		</span>

		{#if hasNext}
			<a class="step" href={hrefFor(page + 1)} rel="next">
				<span>Next</span>
				<Icon name="chevron-right" size={14} />
			</a>
		{:else}
			<span class="step disabled" aria-disabled="true">
				<span>Next</span>
				<Icon name="chevron-right" size={14} />
			</span>
		{/if}
	</div>
</div>

<style>
	.pagination {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-4);
		padding: var(--s-3) var(--s-4);
		border-top: 1px solid var(--line);
		flex-wrap: wrap;
	}

	.summary {
		font-size: var(--fs-meta);
		color: var(--text-faint);
	}

	.controls {
		display: flex;
		align-items: center;
		gap: var(--s-3);
	}

	.step {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 10px;
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		background: var(--surface-2);
		color: var(--text);
		font-size: var(--fs-secondary);
		text-decoration: none;
		transition: background-color var(--dur-state) var(--ease);
	}

	.step:hover {
		background: var(--surface-3);
		text-decoration: none;
	}

	.step.disabled {
		color: var(--text-faint);
		background: var(--surface-1);
		cursor: not-allowed;
	}

	.position {
		font-size: var(--fs-meta);
		color: var(--text-muted);
	}
</style>
