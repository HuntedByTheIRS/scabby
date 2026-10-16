<script lang="ts">
	import { diffStats, parseUnifiedDiff, type DiffLine } from '$lib/diff';

	interface Props {
		/** Raw unified-diff text. Parsed here; nothing is synthesized if absent. */
		patch?: string | null;
		/** Pre-parsed lines, when the caller already has them. */
		lines?: DiffLine[];
		label?: string;
	}

	let { patch, lines, label = 'Diff' }: Props = $props();

	const parsed = $derived<DiffLine[]>(lines ?? (patch ? parseUnifiedDiff(patch) : []));
	const stats = $derived(diffStats(parsed));
</script>

{#if parsed.length > 0}
	<div class="diff" aria-label={label}>
		<div class="stats">
			<span class="add mono">+{stats.additions}</span>
			<span class="del mono">−{stats.deletions}</span>
		</div>
		<div class="lines" role="table" aria-label="{label} lines">
			{#each parsed as line, index (index)}
				<div class="line {line.type}" role="row">
					<span class="gutter mono" role="cell" aria-hidden="true"
						>{line.oldNo ?? ''}</span
					>
					<span class="gutter mono" role="cell" aria-hidden="true"
						>{line.newNo ?? ''}</span
					>
					<span class="marker" role="cell" aria-hidden="true">
						{line.type === 'add' ? '+' : line.type === 'del' ? '−' : ' '}
					</span>
					<span class="text" role="cell">{line.text}</span>
				</div>
			{/each}
		</div>
	</div>
{/if}

<style>
	.diff {
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		overflow: hidden;
		background: var(--surface-1);
	}

	.stats {
		display: flex;
		gap: var(--s-3);
		padding: 6px var(--s-4);
		border-bottom: 1px solid var(--line);
		font-size: var(--fs-meta);
	}

	.stats .add {
		color: var(--ok);
	}

	.stats .del {
		color: var(--danger);
	}

	.lines {
		overflow-x: auto;
		font-size: var(--fs-secondary);
	}

	.line {
		display: grid;
		grid-template-columns: 48px 48px 18px minmax(0, 1fr);
		align-items: stretch;
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		line-height: 1.6;
	}

	.gutter {
		color: var(--text-faint);
		text-align: right;
		padding: 0 8px;
		user-select: none;
		border-right: 1px solid var(--line);
	}

	.marker {
		color: var(--text-faint);
		text-align: center;
		user-select: none;
	}

	.text {
		white-space: pre;
		padding-right: var(--s-4);
		color: var(--text);
	}

	.add {
		background: var(--add-bg);
	}

	.del {
		background: var(--del-bg);
	}

	.add .text {
		color: var(--ok);
	}

	.del .text {
		color: var(--danger);
	}

	.hunk {
		background: var(--surface-3);
	}

	.hunk .text {
		color: var(--lilac);
	}

	.hunk .gutter,
	.hunk .marker {
		background: var(--surface-3);
	}

	.meta .text {
		color: var(--text-faint);
	}

	.context .text {
		color: var(--text-muted);
	}
</style>
