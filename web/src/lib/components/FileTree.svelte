<script lang="ts">
	import type { TreeEntry } from '$lib/api';
	import { formatBytes } from '$lib/format';
	import Icon from './Icon.svelte';

	export interface LastCommit {
		sha: string;
		subject: string;
		when: string;
	}

	interface Props {
		entries: TreeEntry[];
		hrefFor: (entry: TreeEntry) => string;
		/** Per-path last commit. The v1 API does not expose this, so callers that
		 *  have no data leave it undefined and the column renders an em dash. */
		lastCommitFor?: (entry: TreeEntry) => LastCommit | null;
		showLastCommit?: boolean;
	}

	let { entries, hrefFor, lastCommitFor, showLastCommit = true }: Props = $props();
</script>

<ul class="tree">
	{#each entries as entry (entry.path)}
		<li class="entry">
			<a class="row" class:dir={entry.type === 'dir'} href={hrefFor(entry)}>
				<span class="name">
					<span class="glyph" aria-hidden="true">
						<Icon name={entry.type === 'dir' ? 'folder' : 'file'} size={15} />
					</span>
					<span class="label">{entry.name}</span>
					<span class="visually-hidden"
						>{entry.type === 'dir' ? 'directory' : 'file'}</span
					>
				</span>
				<span class="size mono">{entry.type === 'dir' ? '—' : formatBytes(entry.size)}</span>
				{#if showLastCommit}
					{@const commit = lastCommitFor?.(entry) ?? null}
					<span class="commit">
						{#if commit}
							<span class="subject">{commit.subject}</span>
							<span class="when faint">{commit.when}</span>
						{:else}
							<span class="unknown" title="Per-path last commit is not exposed by the v1 API"
								>—</span
							>
						{/if}
					</span>
				{/if}
			</a>
		</li>
	{/each}
</ul>

<style>
	.tree {
		list-style: none;
		margin: 0;
		padding: 0;
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		overflow: hidden;
		background: var(--surface-1);
	}

	.entry + .entry {
		border-top: 1px solid var(--line);
	}

	.row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 90px;
		align-items: center;
		gap: var(--s-4);
		padding: 8px var(--s-4);
		color: var(--text-muted);
		text-decoration: none;
		line-height: 1.35;
		transition: background-color var(--dur-state) var(--ease);
	}

	.row:has(.commit) {
		grid-template-columns: minmax(0, 1fr) 90px minmax(0, 220px);
	}

	.row:hover {
		background: var(--surface-2);
		text-decoration: none;
	}

	.row:active {
		background: var(--surface-3);
	}

	.name {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		min-width: 0;
	}

	.glyph {
		display: inline-flex;
		color: var(--text-faint);
	}

	.row.dir .glyph {
		color: var(--purple-hi);
	}

	.label {
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		font-size: var(--fs-body);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.row.dir .label {
		color: var(--text);
		font-weight: 500;
	}

	.size {
		font-size: var(--fs-secondary);
		color: var(--text-faint);
		text-align: right;
	}

	.commit {
		display: flex;
		align-items: baseline;
		gap: var(--s-2);
		min-width: 0;
		font-size: var(--fs-secondary);
	}

	.subject {
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.when {
		flex: 0 0 auto;
	}

	.unknown {
		color: var(--text-faint);
	}

	@media (max-width: 768px) {
		.row,
		.row:has(.commit) {
			grid-template-columns: minmax(0, 1fr) 70px;
		}

		.commit {
			display: none;
		}
	}
</style>
