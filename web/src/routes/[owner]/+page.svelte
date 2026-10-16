<script lang="ts">
	import { page } from '$app/state';
	import { ApiError, api, type Repository, type User } from '$lib/api';
	import Avatar from '$lib/components/Avatar.svelte';
	import Badge from '$lib/components/Badge.svelte';
	import Card from '$lib/components/Card.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import ListRow from '$lib/components/ListRow.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StatePanel, { type PanelError } from '$lib/components/StatePanel.svelte';
	import Table from '$lib/components/Table.svelte';
	import { formatBytes, monthYear, relativeTime } from '$lib/format';

	const params = $derived(page.params as Record<string, string>);
	const perPage = 30;

	const pageNumber = $derived.by(() => {
		const raw = Number(page.url.searchParams.get('page') ?? '1');
		return Number.isFinite(raw) && raw > 0 ? Math.floor(raw) : 1;
	});

	let user = $state<User | null>(null);
	let repos = $state<Repository[]>([]);
	// Nullable: a collection the server did not count answers with `null` for both.
	let total = $state<number | null>(null);
	let totalPages = $state<number | null>(null);
	let loading = $state(true);
	let error = $state<PanelError | null>(null);

	async function load() {
		const username = params.owner;
		if (!username) return;
		loading = true;
		error = null;
		try {
			const [profile, list] = await Promise.all([
				api.getUserByName(username),
				api.getUserRepos(username, { page: pageNumber, per_page: perPage })
			]);
			user = profile;
			repos = list.items;
			total = list.total;
			totalPages = list.total_pages;
		} catch (e) {
			const err = e as ApiError;
			user = null;
			repos = [];
			error = { status: typeof err.status === 'number' ? err.status : 0, message: err.message };
		} finally {
			loading = false;
		}
	}

	let loadedKey = $state('');
	$effect(() => {
		const key = `${params.owner}?page=${pageNumber}`;
		if (key !== loadedKey) {
			loadedKey = key;
			void load();
		}
	});

	const hrefFor = (target: number) => {
		const base = `/${params.owner}`;
		return target === 1 ? base : `${base}?page=${target}`;
	};
</script>

<StatePanel {loading} {error} onretry={load} skeletonRows={2} skeletonHeight={64}>
	{#if user}
		<header class="profile">
			<Avatar name={user.username} src={user.avatar_url} size={96} />
			<div class="who">
				<h1>{user.display_name ?? user.username}</h1>
				<p class="handle mono">@{user.username}</p>
				{#if user.bio}
					<p class="bio">{user.bio}</p>
				{/if}
				<p class="since faint">
					<Icon name="clock" size={13} />
					Member since {monthYear(user.created_at)}
				</p>
			</div>
		</header>

		<section class="repos" aria-labelledby="repos-title">
			<div class="section-head">
				<h2 id="repos-title">Repositories</h2>
				{#if total !== null}
					<span class="count mono">{total}</span>
				{/if}
			</div>

			{#if repos.length === 0}
				<Card padded={false}>
					<EmptyState
						icon="repo"
						title="No public repositories"
						description="{user.username} has no repositories visible to you."
					/>
				</Card>
			{:else}
				<div class="list">
					<Table
						bordered={false}
						label="Repositories"
						columns={[
							{ key: 'repo', label: 'Repository' },
							{ key: 'updated', label: 'Updated', align: 'right' }
						]}
					>
						{#each repos as repo (repo.id)}
							<ListRow href="/{repo.owner.username}/{repo.name}">
								<span class="repo-line">
									<span class="repo-name mono">{repo.full_name}</span>
									<Badge
										variant={repo.visibility === 'private' ? 'neutral' : 'lilac'}
										icon={repo.visibility === 'private' ? 'lock' : 'globe'}
									>
										{repo.visibility}
									</Badge>
								</span>
								{#snippet trailing()}
									<span class="mono faint">{relativeTime(repo.updated_at)}</span>
									<span class="mono faint">{formatBytes(repo.size_bytes)}</span>
								{/snippet}
							</ListRow>
							{#if repo.description}
								<li class="desc">{repo.description}</li>
							{/if}
						{/each}
					</Table>
					<Pagination
						page={pageNumber}
						{totalPages}
						{total}
						{perPage}
						count={repos.length}
						hrefFor={(target) => hrefFor(target)}
					/>
				</div>
			{/if}
		</section>
	{/if}
</StatePanel>

<style>
	.profile {
		display: flex;
		gap: var(--s-6);
		align-items: flex-start;
		padding-bottom: var(--s-8);
		flex-wrap: wrap;
	}

	.who {
		display: flex;
		flex-direction: column;
		gap: var(--s-1);
		min-width: 0;
	}

	h1 {
		font-size: var(--fs-h1);
	}

	.handle {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.bio {
		color: var(--text);
		font-size: var(--fs-body);
		max-width: var(--prose-max);
		margin-top: var(--s-2);
	}

	.since {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: var(--fs-meta);
		margin-top: var(--s-2);
	}

	.section-head {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		margin-bottom: var(--s-3);
	}

	h2 {
		font-size: var(--fs-h2);
	}

	.count {
		font-size: var(--fs-meta);
		color: var(--text-muted);
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-pill);
		padding: 1px 8px;
	}

	.list {
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		overflow: hidden;
		background: var(--surface-1);
	}

	.repo-line {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		min-width: 0;
	}

	.repo-name {
		color: var(--lilac);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.desc {
		list-style: none;
		padding: 0 var(--s-4) 10px;
		margin: 0;
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}
</style>
