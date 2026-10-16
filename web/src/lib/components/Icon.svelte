<script lang="ts">
	import { ICONS, type IconName } from '$lib/icons';

	interface Props {
		name: IconName;
		size?: number;
		filled?: boolean;
		label?: string;
		stroke?: number;
	}
	let { name, size = 16, filled = false, label, stroke = 1.7 }: Props = $props();
</script>

<svg
	class="icon"
	class:filled
	class:spin={name === 'spinner'}
	width={size}
	height={size}
	viewBox="0 0 24 24"
	role={label ? 'img' : 'presentation'}
	aria-label={label}
	aria-hidden={label ? undefined : 'true'}
	stroke-width={stroke}
>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -- static, first-party icon markup -->
	{@html ICONS[name]}
</svg>

<style>
	.icon {
		display: inline-block;
		flex: 0 0 auto;
		fill: none;
		stroke: currentColor;
		stroke-linecap: round;
		stroke-linejoin: round;
		vertical-align: middle;
	}

	.icon.filled {
		fill: currentColor;
	}

	.spin {
		animation: icon-spin 700ms linear infinite;
	}

	@keyframes icon-spin {
		to {
			transform: rotate(360deg);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.spin {
			animation: none;
			opacity: 0.7;
		}
	}
</style>
