<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { ApiError, api, type Repository } from '$lib/api';
	import Button from '$lib/components/Button.svelte';
	import Card from '$lib/components/Card.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Input from '$lib/components/Input.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import Select from '$lib/components/Select.svelte';
	import Textarea from '$lib/components/Textarea.svelte';
	import { session } from '$lib/session.svelte';
	import { toasts } from '$lib/toast.svelte';

	const params = $derived(page.params as Record<string, string>);
	const repo = $derived((page.data as { repo: Repository | null }).repo);

	let description = $state('');
	let visibility = $state('public');
	let defaultBranch = $state('main');
	let ready = $state(false);
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let saveFields = $state<Record<string, string>>({});

	let confirming = $state(false);
	let deleting = $state(false);

	const visibilityOptions = [
		{ value: 'public', label: 'Public' },
		{ value: 'private', label: 'Private' }
	];

	$effect(() => {
		if (!ready && repo) {
			description = repo.description ?? '';
			visibility = repo.visibility;
			defaultBranch = repo.default_branch || 'main';
			ready = true;
		}
	});

	const isOwner = $derived(session.user !== null && repo !== null && session.user.username === repo.owner.username);

	async function save(event: SubmitEvent) {
		event.preventDefault();
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name) return;
		saving = true;
		saveError = null;
		saveFields = {};
		try {
			const updated = await api.updateRepo(owner, name, {
				description: description.trim() || null,
				private: visibility === 'private',
				default_branch: defaultBranch.trim() || 'main'
			});
			description = updated.description ?? '';
			visibility = updated.visibility;
			defaultBranch = updated.default_branch || 'main';
			toasts.success('Repository settings saved.');
		} catch (e) {
			if (e instanceof ApiError) {
				saveFields = e.fieldErrors();
				saveError = e.message;
			} else {
				saveError = 'Could not save the settings.';
			}
		} finally {
			saving = false;
		}
	}

	async function destroyRepo() {
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name) return;
		deleting = true;
		try {
			await api.deleteRepo(owner, name);
			toasts.success('Repository deleted.');
			await goto('/dashboard');
		} catch (e) {
			toasts.error(e instanceof ApiError ? e.message : 'Could not delete the repository.');
			deleting = false;
			confirming = false;
		}
	}
</script>

<div class="stack">
	{#if !isOwner && repo}
		<Card padded={true}>
			<div class="notice">
				<span class="glyph" aria-hidden="true"><Icon name="lock" size={16} /></span>
				<p>
					Only the owner of <span class="mono">{repo.full_name}</span> can change these settings. The
					form is read-only for you.
				</p>
			</div>
		</Card>
	{/if}

	<Card title="General" description="Shown on the repository page and in the API." padded={true}>
		{#if saveError}
			<div class="alert" role="alert">
				<span class="glyph" aria-hidden="true"><Icon name="alert" size={15} /></span>
				<p>{saveError}</p>
			</div>
		{/if}
		<form onsubmit={save} novalidate>
			<Textarea
				label="Description"
				name="description"
				rows={3}
				bind:value={description}
				disabled={!isOwner}
				error={saveFields.description ?? null}
			/>
			<Select
				label="Visibility"
				name="visibility"
				bind:value={visibility}
				options={visibilityOptions}
				disabled={!isOwner}
				hint="Private repositories return 404 to anyone who cannot see them."
				error={saveFields.private ?? null}
			/>
			<Input
				label="Default branch"
				name="default_branch"
				bind:value={defaultBranch}
				disabled={!isOwner}
				hint="Used for the repository page, the README and the commit list."
				error={saveFields.default_branch ?? null}
			/>
			<div class="actions">
				<Button type="submit" loading={saving} disabled={!isOwner}>Save changes</Button>
			</div>
		</form>
	</Card>

	<Card>
		<div class="danger">
			<div>
				<h3>Delete this repository</h3>
				<p class="muted">
					Removes <span class="mono">{repo?.full_name ?? ''}</span> from Postgres and deletes the bare
					repository from disk. This cannot be undone.
				</p>
			</div>
			<Button
				variant="danger"
				icon="trash"
				disabled={!isOwner}
				onclick={() => (confirming = true)}>Delete repository</Button
			>
		</div>
	</Card>
</div>

<Modal
	open={confirming}
	title="Delete repository"
	description={repo
		? `Everything under ${repo.full_name} — commits, issues and the on-disk repository — is removed. There is no undo.`
		: ''}
	confirmLabel="Delete repository"
	danger
	loading={deleting}
	onconfirm={destroyRepo}
	onclose={() => (confirming = false)}
/>

<style>
	.stack {
		display: flex;
		flex-direction: column;
		gap: var(--s-6);
		padding-top: var(--s-4);
	}

	form {
		display: flex;
		flex-direction: column;
		gap: var(--s-4);
	}

	.actions {
		display: flex;
		gap: var(--s-2);
		margin-top: var(--s-1);
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

	.notice {
		display: flex;
		gap: var(--s-3);
		color: var(--text-muted);
		font-size: var(--fs-secondary);
	}

	.notice .glyph {
		color: var(--warn);
		display: inline-flex;
		flex: 0 0 auto;
		margin-top: 1px;
	}

	.danger {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--s-6);
		padding: var(--s-4);
		border: 1px solid var(--danger);
		border-radius: var(--r-md);
		background: var(--danger-bg);
		flex-wrap: wrap;
	}

	.danger h3 {
		font-size: var(--fs-label);
		color: var(--text);
		margin-bottom: var(--s-1);
	}

	.muted {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
		max-width: 56ch;
	}
</style>
