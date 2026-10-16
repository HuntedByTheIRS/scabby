<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { ApiError, api, type Issue, type Repository } from '$lib/api';
	import Badge from '$lib/components/Badge.svelte';
	import Button from '$lib/components/Button.svelte';
	import Card from '$lib/components/Card.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Input from '$lib/components/Input.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StatePanel, { type PanelError } from '$lib/components/StatePanel.svelte';
	import Textarea from '$lib/components/Textarea.svelte';
	import { pluralize, relativeTime } from '$lib/format';
	import { isCurrent } from '$lib/nav';
	import { session } from '$lib/session.svelte';
	import { toasts } from '$lib/toast.svelte';

	type Filter = 'open' | 'closed' | 'all';

	const params = $derived(page.params as Record<string, string>);
	const repo = $derived((page.data as { repo: Repository | null }).repo);
	const perPage = 30;

	const filter = $derived.by((): Filter => {
		const raw = page.url.searchParams.get('state');
		return raw === 'closed' || raw === 'all' ? raw : 'open';
	});

	const pageNumber = $derived.by(() => {
		const raw = Number(page.url.searchParams.get('page') ?? '1');
		return Number.isFinite(raw) && raw > 0 ? Math.floor(raw) : 1;
	});

	let issues = $state<Issue[]>([]);
	// Nullable: a collection the server did not count answers with `null` for both.
	let total = $state<number | null>(null);
	let totalPages = $state<number | null>(null);
	let loading = $state(true);
	let error = $state<PanelError | null>(null);

	let counts = $state<{ open: number | null; closed: number | null }>({ open: null, closed: null });
	let countsKnown = $state(false);

	let composing = $state(false);
	let newTitle = $state('');
	let newBody = $state('');
	let creating = $state(false);
	let createError = $state<string | null>(null);
	let createFields = $state<Record<string, string>>({});

	async function load() {
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name) return;
		loading = true;
		error = null;
		try {
			const res = await api.listIssues(owner, name, {
				state: filter,
				page: pageNumber,
				per_page: perPage
			});
			issues = res.items;
			total = res.total;
			totalPages = res.total_pages;
		} catch (e) {
			const err = e as ApiError;
			issues = [];
			error = { status: typeof err.status === 'number' ? err.status : 0, message: err.message };
		} finally {
			loading = false;
		}
	}

	async function loadCounts() {
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name || countsKnown) return;
		try {
			const [open, closed] = await Promise.all([
				api.listIssues(owner, name, { state: 'open', per_page: 1 }),
				api.listIssues(owner, name, { state: 'closed', per_page: 1 })
			]);
			counts = { open: open.total, closed: closed.total };
			countsKnown = counts.open !== null && counts.closed !== null;
		} catch {
			countsKnown = false;
		}
	}

	let loadedKey = $state('');
	$effect(() => {
		const key = `${params.owner}/${params.repo}?state=${filter}&page=${pageNumber}`;
		if (key !== loadedKey) {
			loadedKey = key;
			void load();
		}
	});

	let countsKey = $state('');
	$effect(() => {
		const key = `${params.owner}/${params.repo}`;
		if (key !== countsKey) {
			countsKey = key;
			void loadCounts();
		}
	});

	async function createIssue(event: SubmitEvent) {
		event.preventDefault();
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name) return;
		creating = true;
		createError = null;
		createFields = {};
		try {
			const issue = await api.createIssue(owner, name, {
				title: newTitle.trim(),
				body: newBody.trim() || undefined
			});
			toasts.success(`Opened issue #${issue.number}.`);
			await goto(`/${owner}/${name}/issues/${issue.number}`);
		} catch (e) {
			if (e instanceof ApiError) {
				createFields = e.fieldErrors();
				createError = e.message;
			} else {
				createError = 'Could not create the issue.';
			}
		} finally {
			creating = false;
		}
	}

	const filters: { key: Filter; label: string; count: number | null }[] = $derived([
		{ key: 'open', label: 'Open', count: countsKnown ? counts.open : null },
		{ key: 'closed', label: 'Closed', count: countsKnown ? counts.closed : null },
		{ key: 'all', label: 'All', count: null }
	]);

	const hrefFor = (target: number) =>
		`/${params.owner}/${params.repo}/issues?state=${filter}&page=${target}`;

	const filterHref = (key: Filter) => `/${params.owner}/${params.repo}/issues?state=${key}`;

	const canCreate = $derived(session.isAuthed);
	const pathname = $derived(page.url.pathname as string);
</script>

<div class="head">
	<div class="filters" role="group" aria-label="Filter issues by state">
		{#each filters as item (item.key)}
			<a
				class="chip"
				class:on={isCurrent(filter, item.key)}
				href={filterHref(item.key)}
				aria-current={isCurrent(filter, item.key) ? 'true' : undefined}
			>
				{item.label}
				{#if item.count !== null}<span class="count mono">{item.count}</span>{/if}
			</a>
		{/each}
	</div>
	{#if canCreate}
		<Button size="sm" icon="plus" onclick={() => (composing = !composing)}>
			{composing ? 'Cancel' : 'New issue'}
		</Button>
	{/if}
</div>

{#if composing}
	<Card title="Open an issue" padded={true}>
		{#if createError}
			<div class="alert" role="alert">
				<span class="glyph" aria-hidden="true"><Icon name="alert" size={15} /></span>
				<p>{createError}</p>
			</div>
		{/if}
		<form onsubmit={createIssue} novalidate>
			<Input
				label="Title"
				name="title"
				required
				bind:value={newTitle}
				error={createFields.title ?? null}
			/>
			<Textarea
				label="Description"
				name="body"
				rows={5}
				bind:value={newBody}
				hint="Markdown is accepted by the API; it is stored as text."
				error={createFields.body ?? null}
			/>
			<div class="actions">
				<Button type="submit" loading={creating} disabled={!newTitle.trim()}>
					{creating ? 'Opening' : 'Open issue'}
				</Button>
			</div>
		</form>
	</Card>
{/if}

<Card padded={false}>
	<StatePanel {loading} {error} onretry={load} skeletonRows={5} skeletonHeight={56}>
		{#if issues.length === 0}
			<EmptyState
				icon="issue"
				title={filter === 'closed' ? 'No closed issues' : 'No open issues'}
				description={filter === 'closed'
					? 'Nothing has been closed on this repository.'
					: 'Nothing is open on this repository right now.'}
			/>
		{:else}
			<ul class="rows">
				{#each issues as issue (issue.id)}
					<li>
						<a class="row" href="/{params.owner}/{params.repo}/issues/{issue.number}">
							<span class="state" aria-hidden="true"><Icon name="issue" size={16} /></span>
							<span class="body">
								<span class="title">{issue.title}</span>
								<span class="meta">
									<span class="mono number">#{issue.number}</span>
									<Badge variant={issue.state === 'open' ? 'ok' : 'danger'}>
										{issue.state}
									</Badge>
									<span>opened {relativeTime(issue.created_at)} by</span>
									<span class="mono author">{issue.author.username}</span>
								</span>
							</span>
							{#if issue.comment_count > 0}
								<span class="comments mono">
									<Icon name="book" size={13} />
									{issue.comment_count}
								</span>
							{/if}
						</a>
					</li>
				{/each}
			</ul>
			<Pagination
				page={pageNumber}
				{totalPages}
				{total}
				{perPage}
				count={issues.length}
				hrefFor={(target) => hrefFor(target)}
			/>
		{/if}
	</StatePanel>
</Card>

{#if countsKnown && counts.open !== null && counts.closed !== null}
	<p class="summary faint mono">
		{pluralize(counts.open, 'open issue')} · {pluralize(counts.closed, 'closed issue')}
	</p>
{/if}

<style>
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-4);
		margin: var(--s-4) 0 var(--s-3);
		flex-wrap: wrap;
	}

	.filters {
		display: flex;
		gap: var(--s-2);
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 11px;
		border: 1px solid var(--line);
		border-radius: var(--r-pill);
		background: var(--surface-1);
		color: var(--text-muted);
		font-size: var(--fs-secondary);
		text-decoration: none;
	}

	.chip:hover {
		background: var(--surface-2);
		color: var(--text);
		text-decoration: none;
	}

	.chip.on {
		background: var(--purple-bg);
		border-color: var(--line-strong);
		color: var(--lilac);
	}

	.count {
		font-size: var(--fs-meta);
		color: inherit;
		opacity: 0.8;
	}

	.rows {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.row {
		display: flex;
		align-items: flex-start;
		gap: var(--s-3);
		padding: 12px var(--s-4);
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

	.state {
		flex: 0 0 auto;
		color: var(--text-faint);
		margin-top: 1px;
	}

	.body {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
		flex: 1 1 auto;
	}

	.title {
		font-size: var(--fs-body);
		color: var(--text);
	}

	.meta {
		display: flex;
		align-items: center;
		gap: var(--s-2);
		font-size: var(--fs-meta);
		color: var(--text-muted);
		flex-wrap: wrap;
	}

	.number {
		color: var(--text-faint);
	}

	.author {
		color: var(--text-muted);
	}

	.comments {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		flex: 0 0 auto;
		font-size: var(--fs-meta);
		color: var(--text-faint);
	}

	.actions {
		margin-top: var(--s-2);
	}

	.alert {
		display: flex;
		gap: var(--s-3);
		padding: 10px var(--s-3);
		margin-bottom: var(--s-4);
		background: var(--danger-bg);
		border: 1px solid var(--danger);
		border-radius: var(--r-sm);
		font-size: var(--fs-secondary);
		color: var(--text);
	}

	.alert .glyph {
		color: var(--danger);
		display: inline-flex;
		margin-top: 1px;
	}

	.summary {
		margin-top: var(--s-3);
		font-size: var(--fs-meta);
	}
</style>
