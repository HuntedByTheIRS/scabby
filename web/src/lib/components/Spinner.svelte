<script lang="ts">
	interface Props {
		size?: number;
		label?: string;
	}
	let { size = 16, label = 'Loading' }: Props = $props();
</script>

<svg
	class="spinner"
	width={size}
	height={size}
	viewBox="0 0 24 24"
	role="status"
	aria-label={label}
	stroke-width="2.4"
>
	<circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-opacity="0.22" />
	<path
		d="M21 12a9 9 0 0 0-9-9"
		fill="none"
		stroke="currentColor"
		stroke-linecap="round"
	/>
</svg>

<style>
	.spinner {
		display: inline-block;
		flex: 0 0 auto;
		vertical-align: middle;
		animation: spin 700ms linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.spinner {
			animation: none;
			opacity: 0.7;
		}
	}
</style>
