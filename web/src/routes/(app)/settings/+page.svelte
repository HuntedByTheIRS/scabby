<script lang="ts">
	import { ApiError, api, type ApiToken, type SshKey } from '$lib/api';
	import Button from '$lib/components/Button.svelte';
	import Card from '$lib/components/Card.svelte';
	import CopyField from '$lib/components/CopyField.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import IconButton from '$lib/components/IconButton.svelte';
	import Input from '$lib/components/Input.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import Select from '$lib/components/Select.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StatePanel, { type PanelError } from '$lib/components/StatePanel.svelte';
	import Textarea from '$lib/components/Textarea.svelte';
	import { shortDate } from '$lib/format';
	import { session } from '$lib/session.svelte';
	import { toasts } from '$lib/toast.svelte';

	type Tab = 'profile' | 'tokens' | 'keys';
	let tab = $state<Tab>('profile');

	const tabs: { key: Tab; label: string }[] = [
		{ key: 'profile', label: 'Profile' },
		{ key: 'tokens', label: 'Access tokens' },
		{ key: 'keys', label: 'SSH keys' }
	];

	/* ------------------------------------------------------------- profile -- */

	let displayName = $state('');
	let bio = $state('');
	let profileSaving = $state(false);
	let profileError = $state<string | null>(null);
	let profileFields = $state<Record<string, string>>({});
	let profileReady = $state(false);

	$effect(() => {
		if (!profileReady && session.user) {
			displayName = session.user.display_name ?? '';
			bio = session.user.bio ?? '';
			profileReady = true;
		}
	});

	async function saveProfile(event: SubmitEvent) {
		event.preventDefault();
		profileSaving = true;
		profileError = null;
		profileFields = {};
		try {
			const updated = await api.updateUser({
				display_name: displayName.trim() || null,
				bio: bio.trim() || null
			});
			session.set(updated);
			toasts.success('Profile updated.');
		} catch (e) {
			if (e instanceof ApiError) {
				profileFields = e.fieldErrors();
				profileError = e.message;
			} else {
				profileError = 'Could not save your profile.';
			}
		} finally {
			profileSaving = false;
		}
	}

	/* -------------------------------------------------------------- tokens -- */

	let tokens = $state<ApiToken[]>([]);
	let tokensLoading = $state(true);
	let tokensError = $state<PanelError | null>(null);

	let tokenName = $state('');
	let scopeRepo = $state(true);
	let scopeUser = $state(false);
	let expires = $state('30');
	let tokenCreating = $state(false);
	let tokenFormError = $state<string | null>(null);
	let tokenFields = $state<Record<string, string>>({});
	let newPlaintext = $state<string | null>(null);
	let tokenToDelete = $state<ApiToken | null>(null);
	let deletingToken = $state(false);

	const expiryOptions = [
		{ value: '0', label: 'No expiry' },
		{ value: '30', label: '30 days' },
		{ value: '90', label: '90 days' },
		{ value: '365', label: '1 year' }
	];

	async function loadTokens() {
		tokensLoading = true;
		tokensError = null;
		try {
			const res = await api.listTokens();
			tokens = res.items;
		} catch (e) {
			const err = e as ApiError;
			tokensError = { status: err.status ?? 0, message: err.message };
		} finally {
			tokensLoading = false;
		}
	}

	async function createToken(event: SubmitEvent) {
		event.preventDefault();
		tokenCreating = true;
		tokenFormError = null;
		tokenFields = {};
		newPlaintext = null;
		try {
			const scopes: string[] = [];
			if (scopeRepo) scopes.push('repo');
			if (scopeUser) scopes.push('user');
			const created = await api.createToken({
				name: tokenName.trim(),
				scopes: scopes.length > 0 ? scopes : undefined,
				expires_in_days: Number(expires) > 0 ? Number(expires) : undefined
			});
			newPlaintext = created.plaintext;
			tokens = [created.token, ...tokens];
			tokenName = '';
			toasts.success('Token created.');
		} catch (e) {
			if (e instanceof ApiError) {
				tokenFields = e.fieldErrors();
				tokenFormError = e.message;
			} else {
				tokenFormError = 'Could not create the token.';
			}
		} finally {
			tokenCreating = false;
		}
	}

	async function confirmDeleteToken() {
		if (!tokenToDelete) return;
		deletingToken = true;
		try {
			await api.deleteToken(tokenToDelete.id);
			tokens = tokens.filter((t) => t.id !== tokenToDelete?.id);
			toasts.success('Token revoked.');
			tokenToDelete = null;
		} catch (e) {
			toasts.error(e instanceof ApiError ? e.message : 'Could not revoke the token.');
		} finally {
			deletingToken = false;
		}
	}

	/* ---------------------------------------------------------------- keys -- */

	let keys = $state<SshKey[]>([]);
	let keysLoading = $state(true);
	let keysError = $state<PanelError | null>(null);

	let keyTitle = $state('');
	let keyValue = $state('');
	let keyCreating = $state(false);
	let keyFormError = $state<string | null>(null);
	let keyFields = $state<Record<string, string>>({});
	let keyToDelete = $state<SshKey | null>(null);
	let deletingKey = $state(false);

	async function loadKeys() {
		keysLoading = true;
		keysError = null;
		try {
			const res = await api.listKeys();
			keys = res.items;
		} catch (e) {
			const err = e as ApiError;
			keysError = { status: err.status ?? 0, message: err.message };
		} finally {
			keysLoading = false;
		}
	}

	async function addKey(event: SubmitEvent) {
		event.preventDefault();
		keyCreating = true;
		keyFormError = null;
		keyFields = {};
		try {
			const created = await api.addKey({ title: keyTitle.trim(), key: keyValue.trim() });
			keys = [created, ...keys];
			keyTitle = '';
			keyValue = '';
			toasts.success('SSH key added.');
		} catch (e) {
			if (e instanceof ApiError) {
				keyFields = e.fieldErrors();
				keyFormError = e.message;
			} else {
				keyFormError = 'Could not add the key.';
			}
		} finally {
			keyCreating = false;
		}
	}

	async function confirmDeleteKey() {
		if (!keyToDelete) return;
		deletingKey = true;
		try {
			await api.deleteKey(keyToDelete.id);
			keys = keys.filter((k) => k.id !== keyToDelete?.id);
			toasts.success('SSH key removed.');
			keyToDelete = null;
		} catch (e) {
			toasts.error(e instanceof ApiError ? e.message : 'Could not remove the key.');
		} finally {
			deletingKey = false;
		}
	}

	let booted = $state(false);
	$effect(() => {
		if (!booted && session.user) {
			booted = true;
			void loadTokens();
			void loadKeys();
		}
	});
</script>

<header class="page-head">
	<h1>Settings</h1>
	<p class="muted">Your profile, the tokens scripts use, and the keys git uses.</p>
</header>

<div class="layout">
	<nav class="tabs" aria-label="Settings sections">
		<ul>
			{#each tabs as item (item.key)}
				<li>
					<button
						type="button"
						class:on={tab === item.key}
						aria-current={tab === item.key ? 'true' : undefined}
						onclick={() => (tab = item.key)}
					>
						{item.label}
					</button>
				</li>
			{/each}
		</ul>
	</nav>

	<div class="content">
		{#if tab === 'profile'}
			<Card title="Profile" description="Shown on your public profile page." padded={true}>
				{#if profileError}
					<div class="alert" role="alert">
						<span class="glyph" aria-hidden="true"><Icon name="alert" size={15} /></span>
						<p>{profileError}</p>
					</div>
				{/if}
				<form onsubmit={saveProfile} novalidate>
					<Input
						label="Display name"
						name="display_name"
						bind:value={displayName}
						error={profileFields.display_name ?? null}
					/>
					<Textarea
						label="Bio"
						name="bio"
						rows={4}
						bind:value={bio}
						hint="Plain text. Shown on your profile page."
						error={profileFields.bio ?? null}
					/>
					<div class="readonly">
						<span class="ro-label">Username</span>
						<span class="mono">{session.user?.username ?? '—'}</span>
					</div>
					<div class="readonly">
						<span class="ro-label">Email</span>
						<span class="mono">{session.user?.email ?? '—'}</span>
					</div>
					<div class="actions">
						<Button type="submit" loading={profileSaving}>Save profile</Button>
					</div>
				</form>
			</Card>
		{:else if tab === 'tokens'}
			<div class="stack">
				<Card
					title="Create a token"
					description="Tokens authenticate the API and git pushes. The secret is shown once."
					padded={true}
				>
					{#if newPlaintext}
						<div class="plaintext" role="status">
							<p class="warn-line">
								<Icon name="alert" size={15} />
								Copy this token now — it is not shown again.
							</p>
							<CopyField value={newPlaintext} label="Copy token" />
						</div>
					{/if}

					{#if tokenFormError}
						<div class="alert" role="alert">
							<span class="glyph" aria-hidden="true"><Icon name="alert" size={15} /></span>
							<p>{tokenFormError}</p>
						</div>
					{/if}

					<form onsubmit={createToken} novalidate>
						<Input
							label="Token name"
							name="name"
							bind:value={tokenName}
							required
							hint="A label so you can tell tokens apart later."
							error={tokenFields.name ?? null}
						/>
						<fieldset class="scopes">
							<legend>Scopes</legend>
							<label class="check">
								<input type="checkbox" bind:checked={scopeRepo} />
								<span>repo</span>
							</label>
							<label class="check">
								<input type="checkbox" bind:checked={scopeUser} />
								<span>user</span>
							</label>
							<p class="scope-note">
								Scopes are recorded and displayed but not enforced in v0.
							</p>
						</fieldset>
						<Select label="Expiry" name="expires_in_days" bind:value={expires} options={expiryOptions} />
						<div class="actions">
							<Button type="submit" loading={tokenCreating} disabled={!tokenName.trim()}>
								Create token
							</Button>
						</div>
					</form>
				</Card>

				<Card title="Active tokens" padded={false}>
					<StatePanel
						loading={tokensLoading}
						error={tokensError}
						onretry={loadTokens}
						skeletonRows={3}
					>
						{#if tokens.length === 0}
							<EmptyState
								icon="key"
								title="No tokens yet"
								description="Create one above, or from a shell:"
							>
								<code class="cmd mono">curl -X POST -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/v1/user/tokens</code>
							</EmptyState>
						{:else}
							<ul class="rows">
								{#each tokens as token (token.id)}
									<li class="row">
										<div class="row-main">
											<span class="row-title">{token.name}</span>
											<span class="row-sub mono">
												{token.prefix}…{token.expires_at
													? ` · expires ${shortDate(token.expires_at)}`
													: ' · no expiry'}
											</span>
										</div>
										<IconButton
											name="trash"
											label="Revoke {token.name}"
											variant="danger"
											onclick={() => (tokenToDelete = token)}
										/>
									</li>
								{/each}
							</ul>
						{/if}
					</StatePanel>
				</Card>
			</div>
		{:else}
			<div class="stack">
				<Card
					title="Add an SSH key"
					description="Fingerprints are computed as SHA256:<base64>."
					padded={true}
				>
					{#if keyFormError}
						<div class="alert" role="alert">
							<span class="glyph" aria-hidden="true"><Icon name="alert" size={15} /></span>
							<p>{keyFormError}</p>
						</div>
					{/if}
					<form onsubmit={addKey} novalidate>
						<Input
							label="Title"
							name="title"
							bind:value={keyTitle}
							required
							hint="For example: laptop."
							error={keyFields.title ?? null}
						/>
						<Textarea
							label="Public key"
							name="key"
							rows={4}
							mono
							bind:value={keyValue}
							hint="The contents of your .pub file, starting with ssh-ed25519 or ssh-rsa."
							error={keyFields.key ?? null}
						/>
						<div class="actions">
							<Button
								type="submit"
								loading={keyCreating}
								disabled={!keyTitle.trim() || !keyValue.trim()}
							>
								Add key
							</Button>
						</div>
					</form>
				</Card>

				<Card title="Your keys" padded={false}>
					<StatePanel loading={keysLoading} error={keysError} onretry={loadKeys} skeletonRows={3}>
						{#if keys.length === 0}
							<EmptyState
								icon="key"
								title="No SSH keys yet"
								description="Generate one, then paste the public half above:"
							>
								<code class="cmd mono">ssh-keygen -t ed25519 -C "you@your-host"</code>
							</EmptyState>
						{:else}
							<ul class="rows">
								{#each keys as key (key.id)}
									<li class="row">
										<div class="row-main">
											<span class="row-title">{key.title}</span>
											<span class="row-sub mono">{key.fingerprint}</span>
										</div>
										<IconButton
											name="trash"
											label="Remove {key.title}"
											variant="danger"
											onclick={() => (keyToDelete = key)}
										/>
									</li>
								{/each}
							</ul>
						{/if}
					</StatePanel>
				</Card>
			</div>
		{/if}
	</div>
</div>

<Modal
	open={tokenToDelete !== null}
	title="Revoke token"
	description={tokenToDelete
		? `Any script still using “${tokenToDelete.name}” will start receiving 401 responses.`
		: ''}
	confirmLabel="Revoke token"
	danger
	loading={deletingToken}
	onconfirm={confirmDeleteToken}
	onclose={() => (tokenToDelete = null)}
/>

<Modal
	open={keyToDelete !== null}
	title="Remove SSH key"
	description={keyToDelete
		? `Clones and pushes using “${keyToDelete.title}” will stop working.`
		: ''}
	confirmLabel="Remove key"
	danger
	loading={deletingKey}
	onconfirm={confirmDeleteKey}
	onclose={() => (keyToDelete = null)}
/>

<style>
	.page-head {
		margin-bottom: var(--s-6);
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
		gap: var(--s-6);
		align-items: start;
	}

	.tabs ul {
		display: flex;
		gap: var(--s-2);
		list-style: none;
		margin: 0;
		padding: 0;
		flex-wrap: wrap;
	}

	.tabs button {
		border: 1px solid var(--line);
		background: var(--surface-1);
		color: var(--text-muted);
		font-family: inherit;
		font-size: var(--fs-secondary);
		padding: 6px 12px;
		border-radius: var(--r-pill);
		cursor: pointer;
	}

	.tabs button:hover {
		color: var(--text);
		background: var(--surface-2);
	}

	.tabs button.on {
		background: var(--purple-bg);
		border-color: var(--line-strong);
		color: var(--lilac);
	}

	.content,
	.stack {
		display: flex;
		flex-direction: column;
		gap: var(--s-6);
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

	.plaintext {
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
		padding: var(--s-4);
		margin-bottom: var(--s-4);
		background: var(--warn-bg);
		border: 1px solid var(--warn);
		border-radius: var(--r-sm);
	}

	.warn-line {
		display: flex;
		align-items: center;
		gap: var(--s-2);
		font-size: var(--fs-secondary);
		color: var(--warn);
	}

	.scopes {
		display: flex;
		align-items: center;
		gap: var(--s-4);
		flex-wrap: wrap;
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		padding: var(--s-3);
		margin: 0;
	}

	legend {
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
		padding: 0 4px;
	}

	.check {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-family: var(--font-mono);
		font-size: var(--fs-secondary);
		color: var(--text);
		cursor: pointer;
	}

	.check input {
		accent-color: var(--purple);
	}

	.scope-note {
		flex: 1 1 100%;
		font-size: var(--fs-meta);
		color: var(--text-faint);
	}

	.readonly {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 11px;
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		color: var(--text-muted);
	}

	.ro-label {
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
	}

	.rows {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-4);
		padding: 10px var(--s-4);
	}

	.row + .row {
		border-top: 1px solid var(--line);
	}

	.row-main {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}

	.row-title {
		color: var(--text);
		font-size: var(--fs-body);
	}

	.row-sub {
		color: var(--text-faint);
		font-size: var(--fs-meta);
		overflow-wrap: anywhere;
	}

	.cmd {
		display: block;
		margin-top: var(--s-2);
		padding: var(--s-3);
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		color: var(--lilac);
		font-size: var(--fs-meta);
		text-align: left;
		white-space: pre-wrap;
	}

	@media (min-width: 768px) {
		.layout {
			grid-template-columns: 200px minmax(0, 1fr);
			gap: var(--s-8);
		}

		.tabs ul {
			flex-direction: column;
			gap: 2px;
		}

		.tabs button {
			width: 100%;
			text-align: left;
			border-radius: var(--r-sm);
			border-color: transparent;
			background: none;
			padding: 7px 10px;
		}

		.tabs button.on {
			border-color: transparent;
			background: var(--purple-bg);
			color: var(--lilac);
		}
	}
</style>
