<script lang="ts">
	import { page } from '$app/state';
	import {
		ApiError,
		api,
		isDirectory,
		type ContentsFile,
		type Repository,
		type TreeEntry
	} from '$lib/api';
	import Card from '$lib/components/Card.svelte';
	import FileTree from '$lib/components/FileTree.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StatePanel, { type PanelError } from '$lib/components/StatePanel.svelte';
	import { formatBytes } from '$lib/format';

	const params = $derived(page.params as Record<string, string>);
	const repo = $derived((page.data as { repo: Repository | null }).repo);
	const ref = $derived(params.ref ?? repo?.default_branch ?? 'main');
	const path = $derived(params.path ?? '');

	let entries = $state<TreeEntry[]>([]);
	let file = $state<ContentsFile | null>(null);
	let loading = $state(true);
	let error = $state<PanelError | null>(null);

	async function load() {
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name) return;
		loading = true;
		error = null;
		entries = [];
		file = null;
		try {
			const contents = await api.getContents(owner, name, path, { ref });
			if (isDirectory(contents)) {
				entries = contents.entries;
			} else {
				file = contents;
			}
		} catch (e) {
			const err = e as ApiError;
			error = { status: typeof err.status === 'number' ? err.status : 0, message: err.message };
		} finally {
			loading = false;
		}
	}

	let loadedKey = $state('');
	$effect(() => {
		const key = `${params.owner}/${params.repo}@${ref}/${path}`;
		if (key !== loadedKey) {
			loadedKey = key;
			void load();
		}
	});

	const segments = $derived(path.split('/').filter(Boolean));
	const treeBase = $derived(`/${params.owner}/${params.repo}/tree/${encodeURIComponent(ref)}`);

	const crumbs = $derived([
		{ label: params.owner ?? '', href: `/${params.owner}` },
		{ label: params.repo ?? '', href: `/${params.owner}/${params.repo}` },
		...segments.map((segment, index) => ({
			label: segment,
			href: `${treeBase}/${segments.slice(0, index + 1).join('/')}`
		}))
	]);

	const hrefFor = (entry: TreeEntry) => `${treeBase}/${entry.path}`;

	const lines = $derived(file?.text !== null && file?.text !== undefined ? file.text.split('\n') : []);

	const errorMessage = $derived(
		error?.status === 404 ? `No path “${path || '/'}” at ${ref}.` : (error?.message ?? '')
	);
</script>

<nav class="crumbs" aria-label="Breadcrumb">
	<ol>
		{#each crumbs as crumb, index (crumb.href)}
			<li>
				{#if index === crumbs.length - 1}
					<span class="crumb current mono" aria-current="page">{crumb.label}</span>
				{:else}
					<a class="crumb mono" href={crumb.href}>{crumb.label}</a>
					<span class="sep" aria-hidden="true">/</span>
				{/if}
			</li>
		{/each}
	</ol>
</nav>

{#if loading}
	<div class="panel">
		<Skeleton rows={8} height={26} />
	</div>
{:else if error}
	<Card padded={false}>
		<div class="error-panel" role="alert">
			<span class="glyph" aria-hidden="true"><Icon name="alert" size={18} /></span>
			<p class="title">{error.status === 404 ? 'Path not found' : 'Could not load this path'}</p>
			<p class="muted">{errorMessage}</p>
			<button class="retry" type="button" onclick={load}>Retry</button>
		</div>
	</Card>
{:else if file}
	<Card padded={false}>
		<div class="file-head">
			<span class="mono path">{file.path}</span>
			<span class="meta mono">
				{#if file.binary}
					binary
				{:else}
					{lines.length} lines · {formatBytes(file.size)}
				{/if}
			</span>
		</div>
		{#if file.binary || file.text === null}
			<div class="binary">
				This file is not text, so it is not rendered inline. Fetch the raw bytes from
				<span class="mono">/api/v1/repos/{params.owner}/{params.repo}/raw/{ref}/{file.path}</span>.
			</div>
		{:else}
			<div class="code" role="region" aria-label="{file.path} contents">
				<pre class="mono">{#each lines as line, index (index)}<span class="ln" aria-hidden="true"
							>{String(index + 1).padStart(4, ' ')}</span
						><span class="lt">{line}</span>{/each}</pre>
			</div>
		{/if}
	</Card>
{:else}
	<div class="tree-head">
		<span class="mono branch"><Icon name="branch" size={14} />{ref}</span>
		<span class="muted">root</span>
	</div>
	{#if entries.length === 0}
		<Card padded={false}>
			<div class="empty-row">This directory is empty.</div>
		</Card>
	{:else}
		<FileTree {entries} {hrefFor} />
	{/if}
{/if}

<style>
	.crumbs {
		margin-bottom: var(--s-4);
	}

	.crumbs ol {
		display: flex;
		align-items: center;
		gap: 4px;
		list-style: none;
		margin: 0;
		padding: 0;
		flex-wrap: wrap;
	}

	.crumbs li {
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}

	.crumb {
		font-size: var(--fs-secondary);
		color: var(--purple-hi);
		text-decoration: none;
	}

	.crumb:hover {
		text-decoration: underline;
	}

	.crumb.current {
		color: var(--text);
	}

	.sep {
		color: var(--text-faint);
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

	.panel {
		padding: var(--s-4);
	}

	.file-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-4);
		padding: var(--s-3) var(--s-4);
		border-bottom: 1px solid var(--line);
		background: var(--base);
		flex-wrap: wrap;
	}

	.path {
		color: var(--text);
		font-size: var(--fs-secondary);
	}

	.meta {
		color: var(--text-faint);
		font-size: var(--fs-meta);
	}

	.code {
		overflow-x: auto;
		max-height: 70vh;
		overflow-y: auto;
	}

	pre {
		margin: 0;
		font-size: var(--fs-secondary);
		line-height: 1.65;
	}

	.ln {
		display: inline-block;
		width: 4.5em;
		padding-right: 12px;
		text-align: right;
		color: var(--text-faint);
		user-select: none;
		border-right: 1px solid var(--line);
		margin-right: 12px;
	}

	.lt {
		white-space: pre;
		color: var(--text);
	}

	.binary,
	.empty-row {
		padding: var(--s-6);
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.error-panel {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--s-2);
		padding: var(--s-8);
	}

	.glyph {
		display: inline-grid;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: var(--r-md);
		background: var(--danger-bg);
		border: 1px solid var(--danger);
		color: var(--danger);
	}

	.title {
		font-size: var(--fs-label);
		color: var(--text);
	}

	.muted {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.retry {
		margin-top: var(--s-2);
		border: 1px solid var(--line);
		background: var(--surface-2);
		color: var(--text);
		font-family: inherit;
		font-size: var(--fs-secondary);
		padding: 5px 12px;
		border-radius: var(--r-sm);
		cursor: pointer;
	}

	.retry:hover {
		background: var(--surface-3);
	}
</style>
