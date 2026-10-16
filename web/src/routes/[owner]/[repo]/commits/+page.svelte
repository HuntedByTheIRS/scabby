<script lang="ts">
	import { page } from '$app/state';
	import { ApiError, api, type CommitSummary, type Repository } from '$lib/api';
	import Avatar from '$lib/components/Avatar.svelte';
	import Card from '$lib/components/Card.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StatePanel, { type PanelError } from '$lib/components/StatePanel.svelte';
	import { pluralize, relativeTime, subject } from '$lib/format';

	const params = $derived(page.params as Record<string, string>);
	const repo = $derived((page.data as { repo: Repository | null }).repo);
	const perPage = 30;

	const ref = $derived(page.url.searchParams.get('ref') ?? repo?.default_branch ?? 'main');
	const pageNumber = $derived.by(() => {
		const raw = Number(page.url.searchParams.get('page') ?? '1');
		return Number.isFinite(raw) && raw > 0 ? Math.floor(raw) : 1;
	});

	let commits = $state<CommitSummary[]>([]);
	// The commit list sends `null` for both when the history is too long to count, so these are
	// nullable and the header and the pagination both have to cope.
	let total = $state<number | null>(null);
	let totalPages = $state<number | null>(null);
	let loading = $state(true);
	let error = $state<PanelError | null>(null);

	async function load() {
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name) return;
		loading = true;
		error = null;
		try {
			const res = await api.listCommits(owner, name, { ref, page: pageNumber, per_page: perPage });
			commits = res.items;
			total = res.total;
			totalPages = res.total_pages;
		} catch (e) {
			const err = e as ApiError;
			commits = [];
			error = { status: typeof err.status === 'number' ? err.status : 0, message: err.message };
		} finally {
			loading = false;
		}
	}

	let loadedKey = $state('');
	$effect(() => {
		const key = `${params.owner}/${params.repo}@${ref}?page=${pageNumber}`;
		if (key !== loadedKey) {
			loadedKey = key;
			void load();
		}
	});

	const hrefFor = (target: number) =>
		`/${params.owner}/${params.repo}/commits?ref=${encodeURIComponent(ref)}&page=${target}`;
</script>

<div class="head">
	<h2 class="visually-hidden">Commit history</h2>
	<span class="ref mono"><Icon name="branch" size={14} />{ref}</span>
	<span class="muted mono">
		{#if total === null}
			uncounted history
		{:else}
			{pluralize(total, 'commit')}
		{/if}
	</span>
</div>

<Card padded={false}>
	<StatePanel {loading} {error} onretry={load} skeletonRows={6} skeletonHeight={54}>
		{#if commits.length === 0}
			<EmptyState
				icon="commit"
				title="No commits"
				description="This branch has no commits yet, or the ref does not exist."
			/>
		{:else}
			<ul class="rows">
				{#each commits as commit (commit.sha)}
					<li>
						<a class="row" href="/{params.owner}/{params.repo}/commit/{commit.sha}">
							<span class="dot" aria-hidden="true"><Icon name="commit" size={15} /></span>
							<span class="body">
								<span class="subject">{subject(commit.message)}</span>
								<span class="meta">
									<Avatar name={commit.author.name || 'unknown'} size={24} />
									<span class="author">{commit.author.name || 'unknown'}</span>
									<span class="range" aria-hidden="true">·</span>
									<time class="faint" datetime={commit.when}>{relativeTime(commit.when)}</time>
								</span>
							</span>
							<span class="sha mono" title={commit.sha}>{commit.short_sha}</span>
						</a>
					</li>
				{/each}
			</ul>
			<Pagination
				page={pageNumber}
				{totalPages}
				{total}
				{perPage}
				count={commits.length}
				hrefFor={(target) => hrefFor(target)}
			/>
		{/if}
	</StatePanel>
</Card>

<style>
	.head {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		margin: var(--s-4) 0 var(--s-3);
	}

	.ref {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		padding: 4px 9px;
		font-size: var(--fs-secondary);
		color: var(--text);
	}

	.muted {
		color: var(--text-muted);
		font-size: var(--fs-meta);
	}

	.rows {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.row {
		display: flex;
		align-items: center;
		gap: var(--s-4);
		padding: 10px var(--s-4);
		text-decoration: none;
		color: var(--text);
		transition: background-color var(--dur-state) var(--ease);
	}

	.row:hover {
		background: var(--surface-2);
		text-decoration: none;
	}

	li + li .row {
		border-top: 1px solid var(--line);
	}

	.dot {
		flex: 0 0 auto;
		color: var(--text-faint);
	}

	.body {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
		flex: 1 1 auto;
	}

	.subject {
		color: var(--text);
		font-size: var(--fs-body);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.meta {
		display: flex;
		align-items: center;
		gap: var(--s-2);
		font-size: var(--fs-meta);
		color: var(--text-muted);
	}

	.author {
		color: var(--text-muted);
	}

	.range {
		color: var(--text-faint);
	}

	.sha {
		flex: 0 0 auto;
		font-size: var(--fs-meta);
		color: var(--purple-hi);
	}

	@media (max-width: 767px) {
		.row {
			flex-wrap: wrap;
		}

		.subject {
			white-space: normal;
		}
	}
</style>
