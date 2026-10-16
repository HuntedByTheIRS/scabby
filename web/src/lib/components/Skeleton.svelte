<script lang="ts">
	interface Props {
		rows?: number;
		/** Height of each row; skeletons should match the real row height (CLS). */
		height?: number;
		width?: string;
		radius?: number;
	}

	let { rows = 3, height = 44, width = '100%', radius = 6 }: Props = $props();
</script>

<div class="skeleton" aria-hidden="true">
	{#each Array(rows) as _, index (index)}
		<div class="line" style:height="{height}px" style:width style:border-radius="{radius}px"></div>
	{/each}
</div>

<style>
	.skeleton {
		display: flex;
		flex-direction: column;
	}

	.line {
		background: linear-gradient(
			90deg,
			var(--surface-2) 0%,
			var(--surface-3) 50%,
			var(--surface-2) 100%
		);
		background-size: 200% 100%;
		animation: shimmer 1.4s ease-in-out infinite;
	}

	.line + .line {
		margin-top: 1px;
	}

	@keyframes shimmer {
		0% {
			background-position: 200% 0;
		}
		100% {
			background-position: -200% 0;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.line {
			animation: none;
			background: var(--surface-2);
		}
	}
</style>
