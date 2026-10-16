<script lang="ts">
	import CodeSample, { type Sample } from '$lib/components/CodeSample.svelte';
	import CopyField from '$lib/components/CopyField.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import type { IconName } from '$lib/icons';
	import { SITE } from '$lib/site';

	const cloneCommand = `git clone ${SITE.repo}.git`;

	const features: { icon: IconName; title: string; body: string }[] = [
		{
			icon: 'branch',
			title: 'Async Rust on tokio and axum',
			body: 'Requests are served by an async tokio runtime, routed by axum. Git smart HTTP streams upload-pack and receive-pack through the same process.'
		},
		{
			icon: 'repo',
			title: 'One binary serves everything',
			body: 'The interface is built to static files and shipped inside the server binary. Any route it does not recognise falls through to the app shell.'
		},
		{
			icon: 'filter',
			title: 'Postgres for state, disk for repos',
			body: 'Users, repositories, issues, tokens and sessions live in Postgres. Repository contents stay on disk as bare git repos.'
		},
		{
			icon: 'code',
			title: 'Every screen is an endpoint',
			body: 'The UI calls /api/v1 for everything it does. Personal access tokens let a script or a git client do the same work with the same API.'
		}
	];

	const samples: Sample[] = [
		{
			language: 'curl',
			code: `curl -H "Authorization: Bearer $SCABBY_TOKEN" \\
  "http://localhost:3000/api/v1/repos/ada/scabby/commits?ref=main&page=1&per_page=30"`
		},
		{
			language: 'javascript',
			code: `const res = await fetch('/api/v1/repos/ada/scabby/commits?ref=main&per_page=30', {
  credentials: 'include'
});
if (!res.ok) throw new Error(\`\${res.status} \${res.statusText}\`);
const { items, page, per_page, total, total_pages } = await res.json();`
		},
		{
			language: 'json',
			code: `{
  "items": [],
  "page": 1,
  "per_page": 30,
  "total": 0,
  "total_pages": 0
}`
		}
	];

	const steps = [
		{ label: 'Clone the source', code: cloneCommand },
		{ label: 'Create an account', code: 'POST /api/v1/auth/register' },
		{ label: 'Mint a token for scripts', code: 'POST /api/v1/user/tokens' }
	];
</script>

<section class="hero">
	<p class="eyebrow mono">v0.1.0 · {SITE.license}</p>
	<h1>Git hosting you run yourself.</h1>
	<p class="statement">
		Repositories, issues and a scriptable REST API, served by one async Rust binary with Postgres
		behind it.
	</p>

	<div class="clone">
		<span class="clone-label">Get the source</span>
		<CopyField value={cloneCommand} label="Copy clone command" display={cloneCommand} />
	</div>

	<div class="cta">
		<a class="btn" href="/register">Create an account</a>
		<a class="btn secondary" href={SITE.apiReference}>Read the API reference</a>
	</div>
</section>

<section id="features" class="features" aria-labelledby="features-title">
	<h2 id="features-title" class="section-title">What it is</h2>
	<ul class="grid">
		{#each features as feature (feature.title)}
			<li class="feature">
				<span class="glyph" aria-hidden="true"><Icon name={feature.icon} size={16} /></span>
				<h3>{feature.title}</h3>
				<p>{feature.body}</p>
			</li>
		{/each}
	</ul>
</section>

<section id="quickstart" class="quickstart" aria-labelledby="quickstart-title">
	<h2 id="quickstart-title" class="section-title">Running it</h2>
	<ol class="steps">
		{#each steps as step (step.label)}
			<li>
				<span class="step-label">{step.label}</span>
				<code class="mono">{step.code}</code>
			</li>
		{/each}
	</ol>
	<p class="note">
		The server needs <code class="mono">git</code> on <code class="mono">PATH</code> and a
		Postgres connection string. It serves the API under <code class="mono">/api/v1</code>.
	</p>
</section>

<section id="api" class="api" aria-labelledby="api-title">
	<h2 id="api-title" class="section-title">The API the UI uses</h2>
	<p class="lead">
		Collections come back in an envelope — <code class="mono">items</code>,
		<code class="mono">page</code>, <code class="mono">per_page</code>,
		<code class="mono">total</code>, <code class="mono">total_pages</code> — with snake_case
		fields and a <code class="mono">Link</code> header for pagination.
	</p>
	<CodeSample {samples} title="List commits" />
</section>

<style>
	.hero {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--s-4);
		padding: var(--s-12) 0 var(--s-16);
		max-width: 760px;
	}

	.eyebrow {
		font-size: var(--fs-meta);
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-faint);
	}

	h1 {
		font-size: var(--fs-display);
		font-weight: 700;
		letter-spacing: -0.02em;
		line-height: 1.05;
		color: var(--text);
	}

	.statement {
		font-size: var(--fs-subhead);
		line-height: 1.35;
		color: var(--text-muted);
		max-width: 46ch;
	}

	.clone {
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
		width: 100%;
		max-width: 520px;
		margin-top: var(--s-2);
	}

	.clone-label {
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
	}

	.cta {
		display: flex;
		gap: var(--s-3);
		flex-wrap: wrap;
		margin-top: var(--s-2);
	}

	.btn {
		display: inline-flex;
		align-items: center;
		padding: 9px 16px;
		border-radius: var(--r-sm);
		background: var(--purple);
		color: var(--void);
		font-size: var(--fs-body);
		font-weight: 500;
		text-decoration: none;
		transition: background-color var(--dur-state) var(--ease);
	}

	.btn:hover {
		background: var(--purple-hi);
		text-decoration: none;
	}

	.btn.secondary {
		background: var(--surface-2);
		border: 1px solid var(--line);
		color: var(--text);
	}

	.btn.secondary:hover {
		background: var(--surface-3);
		border-color: var(--line-strong);
	}

	section {
		padding: var(--s-12) 0;
		border-top: 1px solid var(--line);
	}

	.section-title {
		font-size: var(--fs-h2);
		color: var(--text);
		margin-bottom: var(--s-6);
	}

	.grid {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: 1fr;
		gap: var(--s-4);
	}

	.feature {
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
		padding: var(--s-4);
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-md);
	}

	.glyph {
		display: inline-grid;
		place-items: center;
		width: 30px;
		height: 30px;
		border-radius: var(--r-sm);
		background: var(--surface-3);
		border: 1px solid var(--line);
		color: var(--purple-hi);
	}

	.feature h3 {
		font-size: var(--fs-label);
		font-weight: 600;
		color: var(--text);
	}

	.feature p {
		color: var(--text-muted);
		font-size: var(--fs-body);
	}

	.steps {
		list-style: none;
		margin: 0 0 var(--s-4);
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: var(--s-2);
	}

	.steps li {
		display: flex;
		align-items: baseline;
		gap: var(--s-4);
		padding: 10px var(--s-4);
		background: var(--surface-1);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		flex-wrap: wrap;
	}

	.step-label {
		flex: 0 0 auto;
		font-size: var(--fs-meta);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-faint);
		min-width: 180px;
	}

	.steps code {
		color: var(--lilac);
		font-size: var(--fs-secondary);
		flex: 1 1 auto;
		overflow-wrap: anywhere;
	}

	.note,
	.lead {
		color: var(--text-muted);
		font-size: var(--fs-body);
		max-width: var(--prose-max);
	}

	.lead {
		margin-bottom: var(--s-6);
	}

	code {
		background: var(--surface-3);
		border: 1px solid var(--line);
		border-radius: var(--r-sm);
		padding: 1px 5px;
		font-size: 0.8125em;
	}

	.steps code {
		background: none;
		border: none;
		padding: 0;
	}

	@media (min-width: 768px) {
		.grid {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}

	@media (min-width: 1280px) {
		.grid {
			grid-template-columns: repeat(3, minmax(0, 1fr));
		}

		.api {
			display: grid;
			grid-template-columns: minmax(0, 1fr) minmax(0, 620px);
			gap: var(--s-8);
			align-items: start;
		}

		.api .section-title {
			grid-column: 1 / -1;
		}
	}
</style>
