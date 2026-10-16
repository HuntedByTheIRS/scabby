<script lang="ts">
	import type { FullAutoFill } from 'svelte/elements';
	import { uid } from '$lib/id';

	interface Props {
		label?: string;
		value?: string;
		type?: 'text' | 'email' | 'password' | 'search' | 'url';
		name?: string;
		id?: string;
		placeholder?: string;
		hint?: string;
		error?: string | null;
		required?: boolean;
		disabled?: boolean;
		readonly?: boolean;
		autocomplete?: FullAutoFill;
		oninput?: (value: string) => void;
	}

	let {
		label,
		value = $bindable(''),
		type = 'text',
		name,
		id,
		placeholder,
		hint,
		error = null,
		required = false,
		disabled = false,
		readonly = false,
		autocomplete,
		oninput: onInput
	}: Props = $props();

	const generatedId = uid('input');
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
	<input
		id={fieldId}
		{name}
		{type}
		{placeholder}
		{disabled}
		{required}
		{readonly}
		{autocomplete}
		bind:value
		aria-invalid={error ? 'true' : undefined}
		aria-describedby={describedBy}
		oninput={(event) => onInput?.(event.currentTarget.value)}
	/>
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

	input {
		width: 100%;
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		color: var(--text);
		font-family: inherit;
		font-size: var(--fs-body);
		padding: 8px 11px;
		transition: border-color var(--dur-state) var(--ease), background-color var(--dur-state) var(--ease);
	}

	input::placeholder {
		color: var(--text-faint);
	}

	input:hover:not(:disabled) {
		border-color: var(--line-strong);
	}

	input:focus-visible {
		border-color: var(--purple);
		outline: 2px solid var(--focus-ring);
		outline-offset: 1px;
	}

	input:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.invalid input {
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
