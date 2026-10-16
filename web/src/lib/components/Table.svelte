<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Column {
		key: string;
		label: string;
		width?: string;
		align?: 'left' | 'right';
		mono?: boolean;
	}

	interface Props {
		columns: Column[];
		label: string;
		bordered?: boolean;
		children: Snippet;
	}

	let { columns, label, bordered = true, children }: Props = $props();
</script>

<div class="table" class:bordered>
	<div class="head" aria-hidden="true" style:--cols={columns.length}>
		{#each columns as column (column.key)}
			<span class="cell" class:right={column.align === 'right'} class:mono={column.mono}>
				{column.label}
			</span>
		{/each}
	</div>
	<!-- Column labels above are decorative; each row carries its own accessible text. -->
	<ul class="rows" aria-label={label}>
		{@render children()}
	</ul>
</div>

<style>
	.table {
		overflow: hidden;
	}

	.table.bordered {
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		background: var(--surface-1);
	}

	.head {
		display: grid;
		grid-template-columns: repeat(var(--cols, 3), minmax(0, 1fr));
		gap: var(--s-4);
		padding: 8px var(--s-4);
		background: var(--surface-1);
		border-bottom: 1px solid var(--line);
	}

	.cell {
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
		font-weight: 500;
	}

	.cell.mono {
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
	}

	.right {
		text-align: right;
	}

	.rows {
		list-style: none;
		margin: 0;
		padding: 0;
	}
</style>
