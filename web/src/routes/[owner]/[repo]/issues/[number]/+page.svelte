<script lang="ts">
	import { page } from '$app/state';
	import {
		ApiError,
		api,
		type Issue,
		type IssueComment,
		type Repository
	} from '$lib/api';
	import Avatar from '$lib/components/Avatar.svelte';
	import Badge from '$lib/components/Badge.svelte';
	import Button from '$lib/components/Button.svelte';
	import Card from '$lib/components/Card.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Skeleton from '$lib/components/Skeleton.svelte';
	import StatePanel, { type PanelError } from '$lib/components/StatePanel.svelte';
	import Textarea from '$lib/components/Textarea.svelte';
	import { relativeTime } from '$lib/format';
	import { session } from '$lib/session.svelte';
	import { toasts } from '$lib/toast.svelte';

	const params = $derived(page.params as Record<string, string>);
	const repo = $derived((page.data as { repo: Repository | null }).repo);
	const number = $derived(Number(params.number ?? '0'));

	let issue = $state<Issue | null>(null);
	let comments = $state<IssueComment[]>([]);
	let loading = $state(true);
	let error = $state<PanelError | null>(null);

	let body = $state('');
	let commenting = $state(false);
	let commentError = $state<string | null>(null);

	let toggling = $state(false);

	async function load() {
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name || !Number.isFinite(number) || number <= 0) return;
		loading = true;
		error = null;
		try {
			const [detail, list] = await Promise.all([
				api.getIssue(owner, name, number),
				api.listComments(owner, name, number, { per_page: 100 })
			]);
			issue = detail;
			comments = list.items;
		} catch (e) {
			const err = e as ApiError;
			issue = null;
			comments = [];
			error = { status: typeof err.status === 'number' ? err.status : 0, message: err.message };
		} finally {
			loading = false;
		}
	}

	let loadedKey = $state('');
	$effect(() => {
		const key = `${params.owner}/${params.repo}#${number}`;
		if (key !== loadedKey) {
			loadedKey = key;
			void load();
		}
	});

	async function submitComment(event: SubmitEvent) {
		event.preventDefault();
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name) return;
		commenting = true;
		commentError = null;
		try {
			const created = await api.createComment(owner, name, number, { body: body.trim() });
			comments = [...comments, created];
			body = '';
			toasts.success('Comment added.');
		} catch (e) {
			commentError = e instanceof ApiError ? e.message : 'Could not post the comment.';
		} finally {
			commenting = false;
		}
	}

	async function toggleState() {
		const owner = params.owner;
		const name = params.repo;
		if (!owner || !name || !issue) return;
		toggling = true;
		const next = issue.state === 'open' ? 'closed' : 'open';
		try {
			const updated = await api.updateIssue(owner, name, number, { state: next });
			issue = updated;
			toasts.success(next === 'closed' ? 'Issue closed.' : 'Issue reopened.');
		} catch (e) {
			toasts.error(e instanceof ApiError ? e.message : 'Could not change the issue state.');
		} finally {
			toggling = false;
		}
	}

	const canComment = $derived(session.isAuthed);
</script>

<StatePanel {loading} {error} onretry={load} skeletonRows={5} skeletonHeight={48}>
	{#if issue}
		<article class="issue">
			<header class="head">
				<div class="title-row">
					<h1>{issue.title}</h1>
					<Badge variant={issue.state === 'open' ? 'ok' : 'danger'}>
						{issue.state}
					</Badge>
				</div>
				<p class="byline">
					<span class="mono number">#{issue.number}</span>
					<Avatar name={issue.author.username} size={24} />
					<span class="mono">{issue.author.username}</span>
					<span class="faint">opened {relativeTime(issue.created_at)}</span>
					{#if issue.closed_at}
						<span class="faint">· closed {relativeTime(issue.closed_at)}</span>
					{/if}
				</p>
			</header>

			<div class="layout">
				<div class="main">
					<Card>
						<div class="comment-head">
							<Avatar name={issue.author.username} size={32} />
							<span class="mono">{issue.author.username}</span>
							<span class="faint">commented {relativeTime(issue.created_at)}</span>
						</div>
						<div class="comment-body">
							{#if issue.body}
								<p>{issue.body}</p>
							{:else}
								<p class="faint">No description was provided.</p>
							{/if}
						</div>
					</Card>

					{#if comments.length === 0}
						<p class="no-comments faint">No comments yet.</p>
					{:else}
						<ul class="comments">
							{#each comments as comment (comment.id)}
								<li>
									<Card>
										<div class="comment-head">
											<Avatar name={comment.author.username} size={32} />
											<span class="mono">{comment.author.username}</span>
											<span class="faint">commented {relativeTime(comment.created_at)}</span>
										</div>
										<div class="comment-body">
											<p>{comment.body}</p>
										</div>
									</Card>
								</li>
							{/each}
						</ul>
					{/if}

					{#if canComment}
						<Card title="Add a comment" padded={true}>
							{#if commentError}
								<div class="alert" role="alert">
									<span class="glyph" aria-hidden="true"><Icon name="alert" size={15} /></span>
									<p>{commentError}</p>
								</div>
							{/if}
							<form onsubmit={submitComment} novalidate>
								<Textarea
									label="Comment"
									name="body"
									rows={5}
									bind:value={body}
									placeholder="Leave a comment"
								/>
								<div class="actions">
									<Button type="submit" loading={commenting} disabled={!body.trim()}>
										Comment
									</Button>
								</div>
							</form>
						</Card>
					{/if}
				</div>

				<aside class="side">
					<Card>
						<div class="side-block">
							<span class="k">State</span>
							<Badge variant={issue.state === 'open' ? 'ok' : 'danger'}>{issue.state}</Badge>
						</div>
						<div class="side-block">
							<span class="k">Comments</span>
							<span class="v mono">{comments.length}</span>
						</div>
						{#if canComment}
							<div class="side-action">
								<Button
									variant="secondary"
									size="sm"
									full
									loading={toggling}
									onclick={toggleState}
									icon={issue.state === 'open' ? 'check' : 'issue'}
								>
									{issue.state === 'open' ? 'Close issue' : 'Reopen issue'}
								</Button>
							</div>
						{/if}
					</Card>
				</aside>
			</div>
		</article>
	{/if}
</StatePanel>

<style>
	.issue {
		padding-top: var(--s-4);
		display: flex;
		flex-direction: column;
		gap: var(--s-6);
	}

	.head {
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
	}

	.title-row {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		flex-wrap: wrap;
	}

	h1 {
		font-size: var(--fs-h1);
		color: var(--text);
	}

	.byline {
		display: flex;
		align-items: center;
		gap: var(--s-2);
		font-size: var(--fs-secondary);
		color: var(--text-muted);
		flex-wrap: wrap;
	}

	.number {
		color: var(--text-faint);
	}

	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--s-6);
		align-items: start;
	}

	.main {
		display: flex;
		flex-direction: column;
		gap: var(--s-4);
		min-width: 0;
	}

	.comment-head {
		display: flex;
		align-items: center;
		gap: var(--s-3);
		padding: var(--s-3) var(--s-4);
		border-bottom: 1px solid var(--line);
		font-size: var(--fs-secondary);
		color: var(--text-muted);
		flex-wrap: wrap;
	}

	.comment-body {
		padding: var(--s-4);
		color: var(--text);
		font-size: var(--fs-body);
		line-height: 1.6;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}

	.comments {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: var(--s-4);
	}

	.no-comments {
		font-size: var(--fs-secondary);
		padding: var(--s-2) 0;
	}

	form {
		display: flex;
		flex-direction: column;
		gap: var(--s-4);
	}

	.actions {
		display: flex;
		gap: var(--s-2);
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

	.side-block {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--s-3);
		padding: 4px 0;
	}

	.side-block + .side-block {
		border-top: 1px solid var(--line);
		padding-top: var(--s-3);
	}

	.k {
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
	}

	.v {
		font-size: var(--fs-secondary);
		color: var(--text);
	}

	.side-action {
		margin-top: var(--s-4);
	}

	@media (min-width: 1024px) {
		.layout {
			grid-template-columns: minmax(0, 1fr) 260px;
		}
	}
</style>
