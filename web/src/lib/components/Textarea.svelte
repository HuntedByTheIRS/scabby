<script lang="ts">
	import { uid } from '$lib/id';

	interface Props {
		label?: string;
		value?: string;
		name?: string;
		id?: string;
		placeholder?: string;
		hint?: string;
		error?: string | null;
		rows?: number;
		required?: boolean;
		disabled?: boolean;
		mono?: boolean;
		maxlength?: number;
		oninput?: (value: string) => void;
	}

	let {
		label,
		value = $bindable(''),
		name,
		id,
		placeholder,
		hint,
		error = null,
		rows = 5,
		required = false,
		disabled = false,
		mono = false,
		maxlength,
		oninput: onInput
	}: Props = $props();

	const generatedId = uid('textarea');
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
	<textarea
		id={fieldId}
		{name}
		{placeholder}
		{rows}
		{disabled}
		{required}
		{maxlength}
		class:mono
		bind:value
		aria-invalid={error ? 'true' : undefined}
		aria-describedby={describedBy}
		oninput={(event) => onInput?.(event.currentTarget.value)}
	></textarea>
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

	textarea {
		width: 100%;
		resize: vertical;
		min-height: 76px;
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		color: var(--text);
		font-family: inherit;
		font-size: var(--fs-body);
		line-height: 1.5;
		padding: 8px 11px;
		transition: border-color var(--dur-state) var(--ease);
	}

	textarea.mono {
		font-family: var(--font-mono);
		font-size: var(--fs-secondary);
	}

	textarea::placeholder {
		color: var(--text-faint);
	}

	textarea:hover:not(:disabled) {
		border-color: var(--line-strong);
	}

	textarea:focus-visible {
		border-color: var(--purple);
		outline: 2px solid var(--focus-ring);
		outline-offset: 1px;
	}

	textarea:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.invalid textarea {
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
