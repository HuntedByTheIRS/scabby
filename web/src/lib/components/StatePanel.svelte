<script lang="ts">
	import type { Snippet } from 'svelte';
	import Button from './Button.svelte';
	import Skeleton from './Skeleton.svelte';

	export interface PanelError {
		status: number;
		message: string;
	}

	interface Props {
		loading?: boolean;
		error?: PanelError | null;
		skeletonRows?: number;
		skeletonHeight?: number;
		onretry?: () => void;
		children?: Snippet;
	}

	let {
		loading = false,
		error = null,
		skeletonRows = 4,
		skeletonHeight = 44,
		onretry,
		children
	}: Props = $props();

	const heading = $derived(
		error === null
			? ''
			: error.status === 404
				? 'Not found'
				: error.status === 403
					? 'Not allowed'
					: error.status === 401
						? 'Sign in required'
						: error.status === 0
							? 'Cannot reach the server'
							: 'Something went wrong'
	);
</script>

{#if loading}
	<div class="panel" aria-busy="true" aria-live="polite">
		<span class="visually-hidden">Loading</span>
		<Skeleton rows={skeletonRows} height={skeletonHeight} />
	</div>
{:else if error}
	<div class="panel error" role="alert">
		<p class="heading">{heading}</p>
		<p class="message">{error.message}</p>
		{#if onretry}
			<div class="retry">
				<Button variant="secondary" size="sm" onclick={onretry}>Retry</Button>
			</div>
		{/if}
	</div>
{:else if children}
	{@render children()}
{/if}

<style>
	.panel {
		padding: var(--s-4);
	}

	.panel.error {
		display: flex;
		flex-direction: column;
		gap: var(--s-1);
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
	}

	.heading {
		font-size: var(--fs-label);
		font-weight: 500;
		color: var(--text);
	}

	.message {
		font-size: var(--fs-secondary);
		color: var(--text-muted);
	}

	.retry {
		margin-top: var(--s-2);
	}
</style>
