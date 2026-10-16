<script lang="ts">
	import { page } from '$app/state';
	import { ApiError, api, type CommitDetail, type CommitFileStatus, type Repository } from '$lib/api';
	import Avatar from '$lib/components/Avatar.svelte';
	import Badge from '$lib/components/Badge.svelte';
	import Card from '$lib/components/Card.svelte';
	import CopyField from '$lib/components/CopyField.svelte';
	import DiffView from '$lib/components/DiffView.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StatePanel, { type PanelError } from '$lib/components/StatePanel.svelte';
	import { relativeTime, subject } from '$lib/format';

	const params = $derived(page.params as Record<string, string>);
	const repo = $derived((page.data as { repo: Repository | null }).repo);
	const sha = $derived(params.sha ?? '');

	let commit = $state<CommitDetail | null>(null);
	let loading = $state(true);
	let error = $state<PanelError | null>(null);

	async function load() {
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name || !sha) return;
		loading = true;
		error = null;
		try {
			commit = await api.getCommit(owner, name, sha);
		} catch (e) {
			const err = e as ApiError;
			commit = null;
			error = { status: typeof err.status === 'number' ? err.status : 0, message: err.message };
		} finally {
			loading = false;
		}
	}

	let loadedKey = $state('');
	$effect(() => {
		const key = `${params.owner}/${params.repo}@${sha}`;
		if (key !== loadedKey) {
			loadedKey = key;
			void load();
		}
	});

	const files = $derived(commit?.files ?? []);

	const totals = $derived({
		additions: commit?.stats?.additions ?? files.reduce((n, f) => n + f.additions, 0),
		deletions: commit?.stats?.deletions ?? files.reduce((n, f) => n + f.deletions, 0)
	});

	const withPatch = $derived(files.filter((f) => typeof f.patch === 'string' && f.patch.length > 0));

	function statusVariant(status: CommitFileStatus): 'ok' | 'danger' | 'neutral' | 'lilac' {
		if (status === 'added') return 'ok';
		if (status === 'deleted') return 'danger';
		if (status === 'renamed') return 'lilac';
		return 'neutral';
	}

	function shortSha(value: string): string {
		return value.slice(0, 7);
	}
</script>

<StatePanel {loading} {error} onretry={load} skeletonRows={6} skeletonHeight={40}>
	{#if commit}
		<article class="commit">
			<header class="head">
				<h1 class="subject">{subject(commit.message)}</h1>
				<div class="byline">
					<Avatar name={commit.author.name || 'unknown'} size={32} />
					<div class="who">
						<span class="author">{commit.author.name || 'unknown'}</span>
						<span class="faint mono">{commit.author.email}</span>
					</div>
					<span class="sep" aria-hidden="true">·</span>
					<span class="faint" title={commit.when}>authored {relativeTime(commit.when)}</span>
				</div>
				{#if commit.body}
					<pre class="body">{commit.body}</pre>
				{/if}
			</header>

			<div class="grid">
				<div class="facts">
					<div class="fact">
						<span class="k">Commit</span>
						<span class="v"><CopyField value={commit.sha} label="Copy SHA" display={shortSha(commit.sha)} /></span>
					</div>
					<div class="fact">
						<span class="k">Committer</span>
						<span class="v mono">{commit.committer.name} &lt;{commit.committer.email}&gt;</span>
					</div>
					{#if commit.parents.length > 0}
						<div class="fact">
							<span class="k">Parents</span>
							<span class="v parents">
								{#each commit.parents as parent (parent)}
									<a class="mono" href="/{params.owner}/{params.repo}/commit/{parent}"
										>{shortSha(parent)}</a
									>
								{/each}
							</span>
						</div>
					{:else}
						<div class="fact">
							<span class="k">Parents</span>
							<span class="v faint">none — root commit</span>
						</div>
					{/if}
					<div class="fact">
						<span class="k">Changed</span>
						<span class="v mono">
							{files.length} files
							<span class="add">+{totals.additions}</span>
							<span class="del">−{totals.deletions}</span>
						</span>
					</div>
				</div>
			</div>

			<section class="files" aria-labelledby="files-title">
				<h2 id="files-title" class="section-title">Files changed</h2>
				{#if files.length === 0}
					<Card padded={false}>
						<p class="empty-row">
							The API returned no file list for this commit. The per-file summary is unavailable
							here.
						</p>
					</Card>
				{:else}
					<Card padded={false}>
						<ul class="file-rows">
							{#each files as file (file.path)}
								<li class="file-row">
									<Badge variant={statusVariant(file.status)}>{file.status}</Badge>
									<span class="file-path mono">{file.path}</span>
									<span class="counts mono">
										{#if file.binary}
											<span class="faint">binary</span>
										{:else}
											<span class="add">+{file.additions}</span>
											<span class="del">−{file.deletions}</span>
										{/if}
									</span>
								</li>
							{/each}
						</ul>
					</Card>
				{/if}
			</section>

			{#if withPatch.length > 0}
				<section class="diff" aria-labelledby="diff-title">
					<h2 id="diff-title" class="section-title">Diff</h2>
					{#each withPatch as file (file.path)}
						<div class="diff-file">
							<p class="diff-path mono">{file.path}</p>
							<DiffView patch={file.patch} label={file.path} />
						</div>
					{/each}
				</section>
			{:else}
				<section class="diff" aria-labelledby="diff-title">
					<h2 id="diff-title" class="section-title">Diff</h2>
					<Card padded={false}>
						<div class="no-diff">
							<span class="glyph" aria-hidden="true"><Icon name="alert" size={16} /></span>
							<p>
								This API version does not return per-line patches, so there is no diff to render.
								The per-file line counts above are the whole summary the server reports.
							</p>
						</div>
					</Card>
				</section>
			{/if}
		</article>
	{/if}
</StatePanel>

<style>
	.commit {
		padding-top: var(--s-4);
		display: flex;
		flex-direction: column;
		gap: var(--s-6);
	}

	.head {
		display: flex;
		flex-direction: column;
		gap: var(--s-3);
	}

	.subject {
		font-size: var(--fs-h1);
		color: var(--text);
	}

	.byline {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		flex-wrap: wrap;
		font-size: var(--fs-secondary);
	}

	.who {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.author {
		color: var(--text);
	}

	.sep {
		color: var(--text-faint);
	}

	.body {
		margin: 0;
		padding: var(--s-4);
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		color: var(--text-muted);
		font-size: var(--fs-secondary);
		line-height: 1.6;
		white-space: pre-wrap;
		overflow-x: auto;
	}

	.facts {
		display: flex;
		flex-direction: column;
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
		overflow: hidden;
	}

	.fact {
		display: grid;
		grid-template-columns: 120px minmax(0, 1fr);
		gap: var(--s-4);
		padding: 8px var(--s-4);
		align-items: center;
	}

	.fact + .fact {
		border-top: 1px solid var(--line);
	}

	.k {
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
	}

	.v {
		color: var(--text);
		font-size: var(--fs-secondary);
		overflow-wrap: anywhere;
	}

	.parents {
		display: flex;
		gap: var(--s-3);
		flex-wrap: wrap;
	}

	.section-title {
		font-size: var(--fs-h2);
		margin-bottom: var(--s-3);
	}

	.file-rows {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.file-row {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		padding: 8px var(--s-4);
	}

	.file-row + .file-row {
		border-top: 1px solid var(--line);
	}

	.file-path {
		flex: 1 1 auto;
		min-width: 0;
		font-size: var(--fs-secondary);
		color: var(--text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.counts {
		flex: 0 0 auto;
		display: flex;
		gap: var(--s-3);
		font-size: var(--fs-meta);
	}

	.add {
		color: var(--ok);
	}

	.del {
		color: var(--danger);
	}

	.empty-row {
		padding: var(--s-6);
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.no-diff {
		display: flex;
		gap: var(--s-3);
		padding: var(--s-6);
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.no-diff .glyph {
		color: var(--warn);
		display: inline-flex;
		flex: 0 0 auto;
		margin-top: 1px;
	}

	.diff {
		display: flex;
		flex-direction: column;
		gap: var(--s-3);
	}

	.diff-file {
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
	}

	.diff-path {
		font-size: var(--fs-secondary);
		color: var(--text-muted);
	}
</style>
