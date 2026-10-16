<script lang="ts">
	import { page } from '$app/state';
	import {
		ApiError,
		api,
		isDirectory,
		type ReadmeResponse,
		type Repository,
		type TreeEntry
	} from '$lib/api';
	import Button from '$lib/components/Button.svelte';
	import Card from '$lib/components/Card.svelte';
	import CopyField from '$lib/components/CopyField.svelte';
	import FileTree from '$lib/components/FileTree.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Markdown from '$lib/components/Markdown.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StatePanel, { type PanelError } from '$lib/components/StatePanel.svelte';
	import { formatBytes, shortDate } from '$lib/format';

	const params = $derived(page.params as Record<string, string>);
	const repo = $derived((page.data as { repo: Repository | null }).repo);
	const ref = $derived(repo?.default_branch || 'main');

	let entries = $state<TreeEntry[]>([]);
	let readme = $state<ReadmeResponse | null>(null);
	let loading = $state(true);
	let error = $state<PanelError | null>(null);

	async function load() {
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name) return;
		loading = true;
		error = null;
		try {
			const contents = await api.getContents(owner, name, '', { ref });
			entries = isDirectory(contents) ? contents.entries : [];
			try {
				readme = await api.getReadme(owner, name, { ref });
			} catch {
				// 404 simply means the repository has no README.
				readme = null;
			}
		} catch (e) {
			const err = e as ApiError;
			entries = [];
			readme = null;
			error = { status: typeof err.status === 'number' ? err.status : 0, message: err.message };
		} finally {
			loading = false;
		}
	}

	let loadedKey = $state('');
	$effect(() => {
		const key = `${params.owner}/${params.repo}@${ref}`;
		if (repo && key !== loadedKey) {
			loadedKey = key;
			void load();
		}
	});

	const isEmpty = $derived(!loading && !error && entries.length === 0 && readme === null);

	const hrefFor = (entry: TreeEntry) =>
		`/${params.owner}/${params.repo}/tree/${encodeURIComponent(ref)}/${entry.path}`;

	const cloneUrl = $derived(repo?.clone_url ?? '');
	const pushSteps = $derived([
		'git init',
		'git add . && git commit -m "first commit"',
		`git remote add origin ${cloneUrl}`,
		`git branch -M ${ref}`,
		`git push -u origin ${ref}`
	]);
</script>

<StatePanel {loading} {error} onretry={load} skeletonRows={6}>
	{#if isEmpty}
		<Card padded={false}>
			<div class="empty">
				<span class="glyph" aria-hidden="true"><Icon name="upload" size={20} /></span>
				<h2>Empty repository</h2>
				<p class="muted">
					Nothing has been pushed to <span class="mono">{ref}</span> yet. Push from a local
					checkout:
				</p>
				<pre class="steps mono">{pushSteps.join('\n')}</pre>
				<div class="empty-actions">
					<CopyField value={pushSteps.join('\n')} label="Copy commands" display="Copy the push commands" />
				</div>
			</div>
		</Card>
	{:else}
		<div class="layout">
			<div class="main">
				{#if readme}
					<Card>
						<div class="readme-head">
							<span class="mono path">{readme.path}</span>
						</div>
						<div class="readme-body">
							<Markdown text={readme.text} />
						</div>
					</Card>
				{:else}
					<Card padded={false}>
						<div class="no-readme">
							<span class="mono">{repo?.full_name}</span> has no README on
							<span class="mono">{ref}</span>.
						</div>
					</Card>
				{/if}
			</div>

			<div class="tree-col">
				<div class="tree-head">
					<span class="mono branch"><Icon name="branch" size={14} />{ref}</span>
					<span class="muted">root</span>
				</div>
				<FileTree {entries} {hrefFor} />
			</div>

			<aside class="about" aria-labelledby="about-title">
				<h2 id="about-title" class="section-title">About</h2>
				<Card>
					<dl class="facts">
						{#if repo?.description}
							<dt class="visually-hidden">Description</dt>
							<dd>{repo.description}</dd>
						{/if}
						<div class="fact">
							<dt>Default branch</dt>
							<dd class="mono">{ref}</dd>
						</div>
						<div class="fact">
							<dt>Size on disk</dt>
							<dd class="mono">{formatBytes(repo?.size_bytes ?? 0)}</dd>
						</div>
						<div class="fact">
							<dt>Open issues</dt>
							<dd class="mono">{repo?.open_issue_count ?? 0}</dd>
						</div>
						<div class="fact">
							<dt>Created</dt>
							<dd class="mono">{shortDate(repo?.created_at ?? null)}</dd>
						</div>
					</dl>
					<div class="about-actions">
						<Button size="sm" variant="secondary" href={`/${params.owner}/${params.repo}/commits`} icon="commit">
							Commit history
						</Button>
					</div>
				</Card>
			</aside>
		</div>
	{/if}
</StatePanel>

<style>
	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--s-6);
		padding-top: var(--s-4);
		align-items: start;
	}

	.main {
		order: 1;
		min-width: 0;
	}

	.tree-col {
		order: 2;
		min-width: 0;
	}

	.about {
		order: 3;
	}

	.tree-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-3);
		margin-bottom: var(--s-2);
	}

	.branch {
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

	.readme-head {
		padding: var(--s-3) var(--s-4);
		border-bottom: 1px solid var(--line);
	}

	.path {
		font-size: var(--fs-meta);
		color: var(--text-faint);
	}

	.readme-body {
		padding: var(--s-6);
	}

	.no-readme {
		padding: var(--s-6);
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--s-3);
		padding: var(--s-8);
	}

	.glyph {
		display: inline-grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: var(--r-md);
		background: var(--surface-3);
		border: 1px solid var(--line);
		color: var(--purple-hi);
	}

	.empty h2 {
		font-size: var(--fs-h2);
	}

	.muted {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.steps {
		width: 100%;
		margin: 0;
		padding: var(--s-4);
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		color: var(--lilac);
		font-size: var(--fs-secondary);
		line-height: 1.7;
		overflow-x: auto;
	}

	.empty-actions {
		width: 100%;
		max-width: 420px;
	}

	.section-title {
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
		font-weight: 500;
		margin-bottom: var(--s-3);
	}

	.facts {
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: var(--s-3);
	}

	.facts dd {
		margin: 0;
	}

	.fact {
		display: flex;
		justify-content: space-between;
		gap: var(--s-3);
		font-size: var(--fs-secondary);
	}

	.fact dt {
		color: var(--text-muted);
	}

	.fact dd {
		color: var(--text);
		font-size: var(--fs-secondary);
	}

	.facts > dd {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
		padding-bottom: var(--s-3);
		border-bottom: 1px solid var(--line);
	}

	.about-actions {
		margin-top: var(--s-4);
	}

	@media (min-width: 1024px) {
		.layout {
			grid-template-columns: minmax(0, 1fr) minmax(0, 320px);
		}

		.main {
			order: 2;
			grid-column: 1;
		}

		.tree-col {
			order: 1;
			grid-column: 1;
		}

		.about {
			order: 3;
			grid-column: 2;
			grid-row: 1 / span 2;
		}
	}
</style>
