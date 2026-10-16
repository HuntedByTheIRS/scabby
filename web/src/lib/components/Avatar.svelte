<script lang="ts">
	import { avatarHues, initials } from '$lib/format';

	interface Props {
		name: string;
		src?: string | null;
		size?: 24 | 32 | 48 | 96;
		title?: string;
	}

	let { name, src, size = 32, title }: Props = $props();

	const hues = $derived(avatarHues(name || '?'));
	const gradient = $derived(
		`linear-gradient(135deg, hsl(${hues[0]} 62% 46%), hsl(${hues[1]} 58% 30%))`
	);
</script>

<span
	class="avatar"
	style:--size="{size}px"
	style:--grad={gradient}
	{title}
	role="img"
	aria-label={name}
>
	{#if src}
		<img {src} alt="" width={size} height={size} />
	{:else}
		<span class="initials" aria-hidden="true">{initials(name)}</span>
	{/if}
</span>

<style>
	.avatar {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: var(--size);
		height: var(--size);
		flex: 0 0 auto;
		border-radius: var(--r-pill);
		overflow: hidden;
		background: var(--grad);
		border: 1px solid var(--line);
		user-select: none;
	}

	img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.initials {
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		font-weight: 600;
		color: var(--text);
		font-size: calc(var(--size) * 0.38);
		letter-spacing: 0.02em;
		line-height: 1;
	}
</style>
