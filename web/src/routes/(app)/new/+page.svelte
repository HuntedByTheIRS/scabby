<script lang="ts">
	import { goto } from '$app/navigation';
	import { ApiError, api } from '$lib/api';
	import Button from '$lib/components/Button.svelte';
	import Card from '$lib/components/Card.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Input from '$lib/components/Input.svelte';
	import Select from '$lib/components/Select.svelte';
	import Textarea from '$lib/components/Textarea.svelte';
	import { session } from '$lib/session.svelte';
	import { toasts } from '$lib/toast.svelte';

	let name = $state('');
	let description = $state('');
	let visibility = $state('public');
	let loading = $state(false);
	let formError = $state<string | null>(null);
	let fieldErrors = $state<Record<string, string>>({});

	const visibilityOptions = [
		{ value: 'public', label: 'Public — anyone can find and clone it' },
		{ value: 'private', label: 'Private — only you can see it' }
	];

	const owner = $derived(session.user?.username ?? '');
	const slug = $derived(name.trim().replace(/\s+/g, '-'));

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		formError = null;
		fieldErrors = {};
		loading = true;
		try {
			const repo = await api.createRepo({
				name: slug,
				description: description.trim() || undefined,
				private: visibility === 'private'
			});
			toasts.success(`Created ${repo.full_name}.`);
			await goto(`/${repo.owner.username}/${repo.name}`);
		} catch (e) {
			if (e instanceof ApiError) {
				if (e.status === 409) {
					fieldErrors = { name: 'A repository with that name already exists.' };
					formError = 'Choose a different name.';
				} else if (e.errors.length > 0) {
					fieldErrors = e.fieldErrors();
					formError = e.message;
				} else {
					formError = e.message;
				}
			} else {
				formError = 'Something went wrong creating the repository.';
			}
		} finally {
			loading = false;
		}
	}
</script>

<header class="page-head">
	<h1>New repository</h1>
	<p class="muted">A repository is a bare git repo on disk plus a row in Postgres.</p>
</header>

<div class="layout">
	<Card padded={true}>
		{#if formError}
			<div class="alert" role="alert">
				<span class="glyph" aria-hidden="true"><Icon name="alert" size={15} /></span>
				<p>{formError}</p>
			</div>
		{/if}

		<form onsubmit={submit} novalidate>
			<Input
				label="Repository name"
				name="name"
				required
				bind:value={name}
				hint="Becomes the last part of the clone URL. Spaces are replaced with hyphens."
				error={fieldErrors.name ?? null}
			/>
			<Textarea
				label="Description"
				name="description"
				rows={3}
				bind:value={description}
				hint="Optional. Shown on the repository page."
				error={fieldErrors.description ?? null}
			/>
			<Select
				label="Visibility"
				name="visibility"
				bind:value={visibility}
				options={visibilityOptions}
				error={fieldErrors.private ?? null}
			/>
			<div class="actions">
				<Button type="submit" {loading} disabled={!slug}>
					{loading ? 'Creating' : 'Create repository'}
				</Button>
				<Button variant="ghost" href="/dashboard">Cancel</Button>
			</div>
		</form>
	</Card>

	<aside class="preview" aria-labelledby="preview-title">
		<h2 id="preview-title" class="section-title">Preview</h2>
		<div class="preview-card">
			<div class="preview-top">
				<span class="mono path">
					<span class="owner">{owner || 'you'}</span><span class="faint">/</span
					><span class="repo">{slug || 'repository-name'}</span>
				</span>
				<span class="vis mono">{visibility}</span>
			</div>
			<p class="preview-desc">{description.trim() || 'No description'}</p>
			<p class="preview-clone mono">
				clone URL is issued by the server once the repository exists.
			</p>
		</div>
	</aside>
</div>

<style>
	.page-head {
		margin-bottom: var(--s-8);
	}

	h1 {
		font-size: var(--fs-h1);
		margin-bottom: var(--s-1);
	}

	.muted {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--s-8);
		align-items: start;
	}

	form {
		display: flex;
		flex-direction: column;
		gap: var(--s-4);
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

	.actions {
		display: flex;
		gap: var(--s-2);
		margin-top: var(--s-2);
		flex-wrap: wrap;
	}

	.section-title {
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
		font-weight: 500;
		margin-bottom: var(--s-3);
	}

	.preview-card {
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
		padding: var(--s-4);
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
	}

	.preview-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-3);
	}

	.path {
		font-size: var(--fs-body);
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.owner {
		color: var(--text-muted);
	}

	.repo {
		color: var(--lilac);
	}

	.vis {
		font-size: var(--fs-meta);
		color: var(--text-muted);
		border: 1px solid var(--line);
		border-radius: var(--r-pill);
		padding: 1px 8px;
	}

	.preview-desc {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.preview-clone {
		color: var(--text-faint);
		font-size: var(--fs-meta);
	}

	@media (min-width: 768px) {
		.layout {
			grid-template-columns: minmax(0, 1fr) minmax(0, 340px);
		}
	}
</style>
