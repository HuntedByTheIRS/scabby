<script lang="ts">
	import { uid } from '$lib/id';
	import Icon from './Icon.svelte';

	interface Option {
		value: string;
		label: string;
	}

	interface Props {
		label?: string;
		value?: string;
		options: Option[];
		name?: string;
		id?: string;
		hint?: string;
		error?: string | null;
		required?: boolean;
		disabled?: boolean;
	}

	let {
		label,
		value = $bindable(''),
		options,
		name,
		id,
		hint,
		error = null,
		required = false,
		disabled = false
	}: Props = $props();

	const generatedId = uid('select');
	const fieldId = $derived(id ?? generatedId);
	const hintId = $derived(`${fieldId}-hint`);
	const errorId = $derived(`${fieldId}-error`);
	const describedBy = $derived(
		[error ? errorId : null, !error && hint ? hintId : null].filter(Boolean).join(' ') || undefined
	);
</script>

<div class="field" class:invalid={!!error}>
	{#if label}
		<label for={fieldId}>
			{label}{#if required}<span class="req" aria-hidden="true"> *</span>{/if}
		</label>
	{/if}
	<div class="wrap">
		<select
			id={fieldId}
			{name}
			{disabled}
			{required}
			bind:value
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={describedBy}
		>
			{#each options as option (option.value)}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
		<span class="caret" aria-hidden="true"><Icon name="chevron-down" size={14} /></span>
	</div>
	{#if error}
		<p class="msg error" id={errorId}>{error}</p>
	{:else if hint}
		<p class="msg hint" id={hintId}>{hint}</p>
	{/if}
</div>

<style>
	.field {
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
	}

	label {
		font-size: var(--fs-label);
		color: var(--text);
		font-weight: 500;
	}

	.req {
		color: var(--danger);
	}

	.wrap {
		position: relative;
		display: block;
	}

	select {
		width: 100%;
		appearance: none;
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		color: var(--text);
		font-family: inherit;
		font-size: var(--fs-body);
		padding: 8px 32px 8px 11px;
		cursor: pointer;
		transition: border-color var(--dur-state) var(--ease);
	}

	select:hover:not(:disabled) {
		border-color: var(--line-strong);
	}

	select:focus-visible {
		border-color: var(--purple);
		outline: 2px solid var(--focus-ring);
		outline-offset: 1px;
	}

	select:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.caret {
		position: absolute;
		right: 10px;
		top: 50%;
		transform: translateY(-50%);
		color: var(--text-muted);
		pointer-events: none;
		display: inline-flex;
	}

	.invalid select {
		border-color: var(--danger);
	}

	.msg {
		font-size: var(--fs-meta);
		margin: 0;
	}

	.error {
		color: var(--danger);
	}

	.hint {
		color: var(--text-muted);
	}
</style>
