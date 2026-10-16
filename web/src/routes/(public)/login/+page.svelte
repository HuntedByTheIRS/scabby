<script lang="ts">
	import { goto } from '$app/navigation';
	import { ApiError } from '$lib/api';
	import Button from '$lib/components/Button.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Input from '$lib/components/Input.svelte';
	import { session } from '$lib/session.svelte';

	let identifier = $state('');
	let password = $state('');
	let loading = $state(false);
	let formError = $state<string | null>(null);
	let fieldErrors = $state<Record<string, string>>({});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		formError = null;
		fieldErrors = {};
		loading = true;
		try {
			await session.login(identifier.trim(), password);
			await goto('/dashboard');
		} catch (e) {
			if (e instanceof ApiError) {
				if (e.status === 401) {
					formError = 'That username or email and password combination was not recognised.';
				} else if (e.errors.length > 0) {
					fieldErrors = e.fieldErrors();
					formError = e.message;
				} else {
					formError = e.message;
				}
			} else {
				formError = 'Something went wrong signing you in. Try again.';
			}
		} finally {
			loading = false;
		}
	}
</script>

<div class="wrap">
	<div class="card">
		<header>
			<h1>Sign in</h1>
			<p class="muted">Access your repositories, issues and tokens.</p>
		</header>

		{#if formError}
			<div class="alert" role="alert">
				<span class="glyph" aria-hidden="true"><Icon name="alert" size={15} /></span>
				<p>{formError}</p>
			</div>
		{/if}

		<form onsubmit={submit} novalidate>
			<Input
				label="Username or email"
				name="username_or_email"
				autocomplete="username"
				required
				bind:value={identifier}
				error={fieldErrors.username_or_email ?? null}
			/>
			<Input
				label="Password"
				name="password"
				type="password"
				autocomplete="current-password"
				required
				bind:value={password}
				error={fieldErrors.password ?? null}
			/>
			<Button type="submit" full {loading} disabled={!identifier || !password}>
				{loading ? 'Signing in' : 'Sign in'}
			</Button>
		</form>

		<p class="foot">
			No account yet? <a href="/register">Create one</a>.
		</p>
	</div>
</div>

<style>
	.wrap {
		display: flex;
		justify-content: center;
		padding: var(--s-8) 0 var(--s-16);
	}

	.card {
		width: 100%;
		max-width: 400px;
		display: flex;
		flex-direction: column;
		gap: var(--s-6);
		padding: var(--s-6);
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
	}

	header {
		display: flex;
		flex-direction: column;
		gap: var(--s-1);
	}

	h1 {
		font-size: var(--fs-h1);
	}

	.muted {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
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
		background: var(--danger-bg);
		border: 1px solid var(--danger);
		border-radius: var(--r-sm);
		color: var(--text);
		font-size: var(--fs-secondary);
	}

	.alert .glyph {
		color: var(--danger);
		display: inline-flex;
		margin-top: 1px;
	}

	.foot {
		color: var(--text-muted);
		font-size: var(--fs-secondary);
		border-top: 1px solid var(--line);
		padding-top: var(--s-4);
	}

	@media (max-width: 767px) {
		.wrap {
			padding: var(--s-4) 0;
		}

		.card {
			border: none;
			background: none;
			padding: 0;
			max-width: none;
		}
	}
</style>
