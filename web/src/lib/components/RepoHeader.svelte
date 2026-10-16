<script lang="ts">
	import type { Repository } from '$lib/api';
	import { session } from '$lib/session.svelte';
	import Badge from './Badge.svelte';
	import CopyField from './CopyField.svelte';
	import Icon from './Icon.svelte';
	import Tabs from './Tabs.svelte';

	interface Props {
		repo: Repository;
		active: 'code' | 'commits' | 'issues' | 'settings';
	}

	let { repo, active }: Props = $props();

	let protocol = $state<'https' | 'ssh'>('https');

	const base = $derived(`/${repo.owner.username}/${repo.name}`);
	const sshUrl = $derived(`git@${location.host}:${repo.full_name}.git`);
	const cloneValue = $derived(protocol === 'https' ? repo.clone_url : sshUrl);
	const isOwner = $derived(session.user !== null && session.user.username === repo.owner.username);

	const tabs = $derived([
		{ key: 'code', label: 'Code', href: base, icon: 'code' as const },
		{ key: 'commits', label: 'Commits', href: `${base}/commits`, icon: 'commit' as const },
		{
			key: 'issues',
			label: 'Issues',
			href: `${base}/issues`,
			count: repo.open_issue_count,
			icon: 'issue' as const
		},
		...(isOwner
			? [{ key: 'settings', label: 'Settings', href: `${base}/settings`, icon: 'settings' as const }]
			: [])
	]);
</script>

<div class="band">
	<div class="row">
		<h1 class="name">
			<a href={`/${repo.owner.username}`} class="owner">{repo.owner.username}</a>
			<span class="slash" aria-hidden="true">/</span>
			<a href={base} class="repo mono">{repo.name}</a>
		</h1>
		<Badge variant={repo.visibility === 'private' ? 'neutral' : 'lilac'} icon={repo.visibility === 'private' ? 'lock' : 'globe'}>
			{repo.visibility}
		</Badge>
	</div>

	{#if repo.description}
		<p class="description">{repo.description}</p>
	{/if}

	<div class="clone">
		<div class="switch" role="group" aria-label="Clone protocol">
			<button
				type="button"
				class:on={protocol === 'https'}
				aria-pressed={protocol === 'https'}
				onclick={() => (protocol = 'https')}>HTTPS</button
			>
			<button
				type="button"
				class:on={protocol === 'ssh'}
				aria-pressed={protocol === 'ssh'}
				onclick={() => (protocol = 'ssh')}>SSH</button
			>
		</div>
		<div class="field">
			<CopyField value={cloneValue} label="Copy clone URL" />
		</div>
	</div>

	<div class="meta">
		{#if repo.default_branch}
			<span class="item mono"><Icon name="branch" size={14} />{repo.default_branch}</span>
		{/if}
		{#if repo.git}
			<span class="item mono"><Icon name="repo" size={14} />repository on disk</span>
		{/if}
	</div>
</div>

<Tabs items={tabs} {active} label="Repository sections" />

<style>
	.band {
		display: flex;
		flex-direction: column;
		gap: var(--s-3);
		padding: var(--s-4) 0;
	}

	.row {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		flex-wrap: wrap;
	}

	.name {
		display: flex;
		align-items: center;
		gap: 2px;
		font-size: var(--fs-h1);
		font-weight: 600;
		min-width: 0;
	}

	.owner {
		color: var(--text-muted);
		text-decoration: none;
		font-weight: 400;
	}

	.owner:hover {
		color: var(--text);
		text-decoration: none;
	}

	.slash {
		color: var(--text-faint);
		padding: 0 2px;
	}

	.repo {
		color: var(--text);
		text-decoration: none;
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.description {
		color: var(--text-muted);
		font-size: var(--fs-body);
		max-width: var(--prose-max);
	}

	.clone {
		display: flex;
		align-items: center;
		gap: var(--s-2);
		max-width: 560px;
	}

	.field {
		flex: 1 1 auto;
		min-width: 0;
	}

	.switch {
		display: inline-flex;
		flex: 0 0 auto;
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		overflow: hidden;
	}

	.switch button {
		border: none;
		background: var(--surface-3);
		color: var(--text-muted);
		font-family: inherit;
		font-size: var(--fs-meta);
		font-weight: 500;
		padding: 6px 10px;
		cursor: pointer;
		transition:
			background-color var(--dur-state) var(--ease),
			color var(--dur-state) var(--ease);
	}

	.switch button:hover {
		color: var(--text);
	}

	.switch button.on {
		background: var(--purple-bg-strong);
		color: var(--lilac);
	}

	.meta {
		display: flex;
		gap: var(--s-4);
		flex-wrap: wrap;
	}

	.item {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: var(--fs-meta);
		color: var(--text-faint);
	}

	@media (max-width: 767px) {
		.clone {
			flex-direction: column;
			align-items: stretch;
		}
	}
</style>
