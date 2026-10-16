<script lang="ts">
	import { ApiError, api, type Repository } from '$lib/api';
	import Badge from '$lib/components/Badge.svelte';
	import Button from '$lib/components/Button.svelte';
	import Card from '$lib/components/Card.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import ListRow from '$lib/components/ListRow.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StatePanel, { type PanelError } from '$lib/components/StatePanel.svelte';
	import { formatBytes, pluralize, relativeTime, subject } from '$lib/format';
	import { session } from '$lib/session.svelte';

	let repos = $state<Repository[]>([]);
	let loading = $state(true);
	let error = $state<PanelError | null>(null);
	let started = $state(false);

	async function load(username: string) {
		loading = true;
		error = null;
		try {
			// `stats=true` adds a `git` block per repository (branches, tags, disk bytes, commit
			// count, head commit) measured on the server's rayon pool, which is what lets a card
			// show a branch count and the last commit without a request per row.
			const mine = await api.getUserRepos(username, { per_page: 100, stats: true });
			repos = mine.items;
		} catch (e) {
			const err = e as ApiError;
			error = { status: typeof err.status === 'number' ? err.status : 0, message: err.message };
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (!started && session.loaded && session.user) {
			started = true;
			void load(session.user.username);
		}
	});

	const activity = $derived(
		[...repos]
			.sort((a, b) => Date.parse(b.updated_at) - Date.parse(a.updated_at))
			.slice(0, 8)
	);
</script>

<header class="page-head">
	<div>
		<h1>Your repositories</h1>
		<p class="muted">
			{#if session.user}
				Signed in as <span class="mono">{session.user.username}</span>.
			{:else}
				Loading your account…
			{/if}
		</p>
	</div>
	<Button href="/new" icon="plus" size="sm">New repository</Button>
</header>

<StatePanel {loading} {error} onretry={() => session.user && load(session.user.username)} skeletonRows={5}>
	{#if repos.length === 0}
		<Card padded={false}>
			<EmptyState
				icon="repo"
				title="No repositories yet"
				description="Create your first repository — it is a bare git repo on disk plus a row in Postgres."
			>
				<Button href="/new" icon="plus">Create repository</Button>
			</EmptyState>
		</Card>
	{:else}
		<div class="layout">
			<section class="repos" aria-labelledby="repos-title">
				<h2 id="repos-title" class="section-title">Repositories</h2>
				<ul class="grid">
					{#each repos as repo (repo.id)}
						<li>
							<a class="repo-card" href="/{repo.owner.username}/{repo.name}">
								<span class="repo-top">
									<span class="repo-name mono">{repo.name}</span>
									<Badge
										variant={repo.visibility === 'private' ? 'neutral' : 'lilac'}
										icon={repo.visibility === 'private' ? 'lock' : 'globe'}
									>
										{repo.visibility}
									</Badge>
								</span>
								<span class="repo-desc">
									{repo.description ?? 'No description'}
								</span>
								<span class="repo-meta mono">
									<span>updated {relativeTime(repo.updated_at)}</span>
									<span>{formatBytes(repo.git?.disk_bytes ?? repo.size_bytes)}</span>
									{#if repo.git}
										<span>{pluralize(repo.git.branches, 'branch')}</span>
										{#if repo.git.tags > 0}
											<span>{pluralize(repo.git.tags, 'tag')}</span>
										{/if}
									{/if}
									{#if repo.open_issue_count > 0}
										<span>{pluralize(repo.open_issue_count, 'issue')}</span>
									{/if}
								</span>
								{#if repo.git?.head_commit}
									<span class="repo-head">
										<span class="head-glyph" aria-hidden="true"
											><Icon name="commit" size={13} /></span
										>
										<span class="head-subject">{subject(repo.git.head_commit.message)}</span>
										<span class="head-when faint mono"
											>{relativeTime(repo.git.head_commit.when)}</span
										>
									</span>
								{:else if repo.git}
									<span class="repo-head">
										<span class="head-when faint mono">no commits on disk yet</span>
									</span>
								{/if}
							</a>
						</li>
					{/each}
				</ul>
			</section>

			<aside class="activity" aria-labelledby="activity-title">
				<h2 id="activity-title" class="section-title">Recent activity</h2>
				<Card padded={false}>
					{#if activity.length === 0}
						<p class="pad muted">Nothing has changed yet.</p>
					{:else}
						<ul class="rows">
							{#each activity as repo (repo.id)}
								<ListRow href="/{repo.owner.username}/{repo.name}">
									<span class="act-icon" aria-hidden="true"
										><Icon name="commit" size={15} /></span
									>
									<span class="act">
										<span class="act-name mono">{repo.full_name}</span>
										<span class="act-when faint">updated {relativeTime(repo.updated_at)}</span>
									</span>
								</ListRow>
							{/each}
						</ul>
					{/if}
				</Card>
			</aside>
		</div>
	{/if}
</StatePanel>

<style>
	.page-head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--s-4);
		margin-bottom: var(--s-8);
		flex-wrap: wrap;
	}

	h1 {
		font-size: var(--fs-h1);
		margin-bottom: var(--s-1);
	}

	.muted {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.section-title {
		font-size: var(--fs-label);
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		font-weight: 500;
		margin-bottom: var(--s-3);
	}

	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--s-8);
		align-items: start;
	}

	.grid {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--s-3);
	}

	.repo-card {
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
		padding: var(--s-4);
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		text-decoration: none;
		transition:
			background-color var(--dur-state) var(--ease),
			border-color var(--dur-state) var(--ease);
	}

	.repo-card:hover {
		background: var(--surface-2);
		border-color: var(--line-strong);
		text-decoration: none;
	}

	.repo-top {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		justify-content: space-between;
	}

	.repo-name {
		color: var(--lilac);
		font-size: var(--fs-label);
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.repo-desc {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
		overflow: hidden;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}

	.repo-meta {
		display: flex;
		gap: var(--s-4);
		flex-wrap: wrap;
		font-size: var(--fs-meta);
		color: var(--text-faint);
	}

	.repo-head {
		display: flex;
		align-items: center;
		gap: var(--s-2);
		min-width: 0;
		font-size: var(--fs-meta);
		color: var(--text-muted);
	}

	.head-glyph {
		display: inline-flex;
		flex: 0 0 auto;
		color: var(--text-faint);
	}

	.head-subject {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.head-when {
		flex: 0 0 auto;
	}

	.rows {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.pad {
		padding: var(--s-4);
	}

	.act-icon {
		display: inline-flex;
		color: var(--text-faint);
		flex: 0 0 auto;
	}

	.act {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}

	.act-name {
		font-size: var(--fs-secondary);
		color: var(--text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.act-when {
		font-size: var(--fs-meta);
	}

	@media (min-width: 768px) {
		.grid {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}

	@media (min-width: 1280px) {
		.layout {
			grid-template-columns: minmax(0, 1fr) 320px;
		}
	}
</style>
