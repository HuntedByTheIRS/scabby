// Typed client for the Scabby REST API v1 (docs/api.md).
// Every call goes through request(): credentials: 'include' (session cookie),
// JSON in/out, and a thrown ApiError carrying status/message/errors for the UI.

const BASE = '/api/v1';

/* ---------------------------------------------------------------- types --- */

export interface ApiFieldError {
	resource: string;
	field: string;
	code: string;
}

export interface ApiErrorBody {
	message: string;
	status: number;
	errors?: ApiFieldError[];
}

export class ApiError extends Error {
	readonly status: number;
	readonly errors: ApiFieldError[];

	constructor(status: number, message: string, errors: ApiFieldError[] = []) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
		this.errors = errors;
	}

	/** First validation message for a field, if the API reported one. */
	fieldError(field: string): string | undefined {
		const hit = this.errors.find((e) => e.field === field);
		return hit ? humanize(hit) : undefined;
	}

	/** field -> message map for form rendering. */
	fieldErrors(): Record<string, string> {
		const out: Record<string, string> = {};
		for (const e of this.errors) out[e.field] = humanize(e);
		return out;
	}

	get isAuth() {
		return this.status === 401;
	}
	get isForbidden() {
		return this.status === 403;
	}
	get isNotFound() {
		return this.status === 404;
	}
	get isConflict() {
		return this.status === 409;
	}
	get isValidation() {
		return this.status === 422;
	}
	get isNetwork() {
		return this.status === 0;
	}
}

function humanize(e: ApiFieldError): string {
	const code = e.code.replace(/_/g, ' ');
	if (e.field) return `${e.field} ${code}`;
	return `${e.resource} ${code}`;
}

export interface Envelope<T> {
	items: T[];
	page: number;
	per_page: number;
	/** `null` when the server did not count the collection. The commit list is the case:
	 *  walking a large history for an exact total is work nobody asked for, so it sends `null`
	 *  and reports only whether a further page exists. */
	total: number | null;
	/** `null` whenever `total` is. */
	total_pages: number | null;
}

export interface User {
	id: number;
	username: string;
	display_name: string | null;
	/** Present only on your own user object. */
	email?: string | null;
	bio: string | null;
	avatar_url: string | null;
	is_admin: boolean;
	created_at: string;
}

export interface RepoOwner {
	id: number;
	username: string;
	display_name: string | null;
	avatar_url?: string | null;
}

export interface Repository {
	id: number;
	owner: RepoOwner;
	name: string;
	full_name: string;
	description: string | null;
	visibility: 'public' | 'private';
	default_branch: string;
	size_bytes: number;
	open_issue_count: number;
	clone_url: string;
	html_url: string;
	created_at: string;
	updated_at: string;
	/** Present when the on-disk git repo exists and the caller asked for `?stats=true`
	 *  on a list, or `GET /repos/{owner}/{repo}` found it. */
	git?: GitStats | null;
}

export interface CommitPerson {
	name: string;
	email: string;
	when: string;
}

export interface CommitSummary {
	sha: string;
	short_sha: string;
	message: string;
	author: CommitPerson;
	committer: CommitPerson;
	parents: string[];
	when: string;
}

/** What `?stats=true` measures about a repository on disk (rayon pool, one entry per repo). */
export interface GitStats {
	/** Number of local branches. */
	branches: number;
	/** Number of tags. */
	tags: number;
	/** Bytes the bare repository occupies. */
	disk_bytes: number;
	/** Commits reachable from the default branch, when it could be walked. */
	commit_count: number | null;
	/** Tip of the default branch. */
	head_commit: CommitSummary | null;
}

export type CommitFileStatus = 'added' | 'modified' | 'deleted' | 'renamed';

export interface CommitFile {
	path: string;
	status: CommitFileStatus;
	additions: number;
	deletions: number;
	binary: boolean;
	/** Not part of api.md v1; tolerated so DiffView can render when a server does send it. */
	patch?: string | null;
}

export interface CommitStats {
	additions: number;
	deletions: number;
	/** Files touched. Sent as `files`, not `files_changed`. */
	files: number;
}

export interface CommitDetail extends CommitSummary {
	/** Everything after the first line of the message; empty when there is none. */
	body: string;
	files: CommitFile[];
	/** Totals for `files`. Sent as `files`, the count of files touched. */
	stats: CommitStats;
}

export type TreeEntryType = 'file' | 'dir' | 'symlink' | 'submodule';

export interface TreeEntry {
	type: TreeEntryType;
	name: string;
	path: string;
	/** Size in bytes; `null` for directories, which have none. */
	size: number | null;
}

export interface ContentsDirectory {
	type: 'dir';
	/** Path listed, empty for the repository root. */
	path: string;
	entries: TreeEntry[];
}

export interface ContentsFile {
	type: 'file';
	name: string;
	path: string;
	size: number;
	/** Object id of the blob. */
	sha: string;
	/** base64, always present. */
	content: string;
	/** decoded UTF-8, null when the file is binary. */
	text: string | null;
	binary: boolean;
}

export type ContentsResponse = ContentsDirectory | ContentsFile;

export function isDirectory(c: ContentsResponse): c is ContentsDirectory {
	return Array.isArray((c as ContentsDirectory).entries);
}

export interface ReadmeResponse {
	path: string;
	text: string;
}

export interface IssueAuthor {
	id: number;
	username: string;
	display_name?: string | null;
	avatar_url?: string | null;
}

export type IssueState = 'open' | 'closed';

export interface Issue {
	id: number;
	number: number;
	title: string;
	body: string | null;
	state: IssueState;
	author: IssueAuthor;
	comment_count: number;
	html_url: string;
	created_at: string;
	updated_at: string;
	closed_at: string | null;
}

export interface IssueComment {
	id: number;
	issue_number?: number;
	body: string;
	author: IssueAuthor;
	created_at: string;
	updated_at?: string;
}

export interface ApiToken {
	id: number;
	name: string;
	prefix: string;
	scopes?: string[];
	created_at: string;
	expires_at?: string | null;
	last_used_at?: string | null;
}

export interface ApiTokenCreated {
	token: ApiToken;
	plaintext: string;
}

export interface SshKey {
	id: number;
	title: string;
	/** The key as pasted. The response field is `public_key`; the request field is `key`. */
	public_key: string;
	fingerprint: string;
	created_at: string;
	last_used_at?: string | null;
}

export interface Branch {
	name: string;
	sha: string;
	is_default?: boolean;
	/** The commit it points at, when it could be read. */
	commit?: CommitSummary | null;
}

export interface Tag {
	name: string;
	sha: string;
	commit?: CommitSummary | null;
}

export interface VersionInfo {
	name: string;
	version: string;
	api_version: string;
}

export interface HealthInfo {
	status: string;
	database: string;
	uptime_seconds: number;
}

/* -------------------------------------------------------------- request --- */

export interface RequestOptions {
	method?: 'GET' | 'POST' | 'PATCH' | 'DELETE';
	body?: unknown;
	query?: Record<string, string | number | boolean | null | undefined>;
	signal?: AbortSignal;
}

export async function request<T>(path: string, opts: RequestOptions = {}): Promise<T> {
	let url = BASE + path;
	if (opts.query) {
		const sp = new URLSearchParams();
		for (const [k, v] of Object.entries(opts.query)) {
			if (v === undefined || v === null || v === '') continue;
			sp.set(k, String(v));
		}
		const qs = sp.toString();
		if (qs) url += `?${qs}`;
	}

	const headers: Record<string, string> = { Accept: 'application/json' };
	let body: string | undefined;
	if (opts.body !== undefined) {
		headers['Content-Type'] = 'application/json';
		body = JSON.stringify(opts.body);
	}

	let res: Response;
	try {
		res = await fetch(url, {
			method: opts.method ?? 'GET',
			headers,
			body,
			credentials: 'include',
			signal: opts.signal
		});
	} catch {
		throw new ApiError(0, 'Network error — the server is unreachable.', []);
	}

	if (res.status === 204) return undefined as T;

	const raw = await res.text();
	let data: unknown = null;
	if (raw) {
		try {
			data = JSON.parse(raw);
		} catch {
			data = null;
		}
	}

	if (!res.ok) {
		const err = (data ?? {}) as Partial<ApiErrorBody>;
		throw new ApiError(
			res.status,
			err.message || res.statusText || 'Request failed',
			err.errors ?? []
		);
	}

	return data as T;
}

/* ------------------------------------------------------------ endpoints --- */

/** The query every collection accepts. `stats` is the repository lists' opt-in measurement:
 *  it adds a `git` block (branches, tags, disk bytes, commit count, head commit) to each repo. */
export type PageQuery = { page?: number; per_page?: number; stats?: boolean };

/** Normalize a collection response. The API answers with an envelope everywhere except
 *  `/user/tokens` and `/user/keys`, which currently return a bare array; `.items` is
 *  meaningful for callers either way. */
export function unwrapList<T>(body: Envelope<T> | T[] | null | undefined): Envelope<T> {
	if (Array.isArray(body)) {
		return {
			items: body,
			page: 1,
			per_page: body.length,
			total: body.length,
			total_pages: body.length > 0 ? 1 : 0
		};
	}
	return {
		items: body?.items ?? [],
		page: body?.page ?? 1,
		per_page: body?.per_page ?? 0,
		total: body?.total ?? null,
		total_pages: body?.total_pages ?? null
	};
}

export function register(input: {
	username: string;
	email: string;
	password: string;
}): Promise<{ user: User }> {
	return request<{ user: User }>('/auth/register', { method: 'POST', body: input });
}

export function login(input: {
	username_or_email: string;
	password: string;
}): Promise<{ user: User }> {
	return request<{ user: User }>('/auth/login', { method: 'POST', body: input });
}

export function logout(): Promise<void> {
	return request<void>('/auth/logout', { method: 'POST' });
}

export function getSession(): Promise<{ user: User }> {
	return request<{ user: User }>('/auth/session');
}

export function getUser(): Promise<User> {
	return request<User>('/user');
}

export function updateUser(input: {
	display_name?: string | null;
	bio?: string | null;
}): Promise<User> {
	return request<User>('/user', { method: 'PATCH', body: input });
}

export function getUserByName(username: string): Promise<User> {
	return request<User>(`/users/${encodeURIComponent(username)}`);
}

export function getUserRepos(username: string, query?: PageQuery): Promise<Envelope<Repository>> {
	return request<Envelope<Repository>>(`/users/${encodeURIComponent(username)}/repos`, { query });
}

export async function listTokens(): Promise<Envelope<ApiToken>> {
	return unwrapList(await request<Envelope<ApiToken> | ApiToken[]>('/user/tokens'));
}

export function createToken(input: {
	name: string;
	scopes?: string[];
	expires_in_days?: number;
}): Promise<ApiTokenCreated> {
	return request<ApiTokenCreated>('/user/tokens', { method: 'POST', body: input });
}

export function deleteToken(id: number): Promise<void> {
	return request<void>(`/user/tokens/${id}`, { method: 'DELETE' });
}

export async function listKeys(): Promise<Envelope<SshKey>> {
	return unwrapList(await request<Envelope<SshKey> | SshKey[]>('/user/keys'));
}

export function addKey(input: { title: string; key: string }): Promise<SshKey> {
	return request<SshKey>('/user/keys', { method: 'POST', body: input });
}

export function deleteKey(id: number): Promise<void> {
	return request<void>(`/user/keys/${id}`, { method: 'DELETE' });
}

export function listRepos(query?: PageQuery): Promise<Envelope<Repository>> {
	return request<Envelope<Repository>>('/repos', { query });
}

export function createRepo(input: {
	name: string;
	description?: string;
	private?: boolean;
}): Promise<Repository> {
	return request<Repository>('/user/repos', { method: 'POST', body: input });
}

export function getRepo(owner: string, repo: string): Promise<Repository> {
	return request<Repository>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}`
	);
}

export function updateRepo(
	owner: string,
	repo: string,
	input: { description?: string | null; private?: boolean; default_branch?: string }
): Promise<Repository> {
	return request<Repository>(`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}`, {
		method: 'PATCH',
		body: input
	});
}

export function deleteRepo(owner: string, repo: string): Promise<void> {
	return request<void>(`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}`, {
		method: 'DELETE'
	});
}

export function listBranches(owner: string, repo: string): Promise<{ items: Branch[] }> {
	return request<{ items: Branch[] }>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/branches`
	);
}

export function listTags(owner: string, repo: string): Promise<{ items: Tag[] }> {
	return request<{ items: Tag[] }>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/tags`
	);
}

export function getStats(owner: string, repo: string): Promise<GitStats> {
	return request<GitStats>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/stats`
	);
}

export function listCommits(
	owner: string,
	repo: string,
	query?: { ref?: string; page?: number; per_page?: number }
): Promise<Envelope<CommitSummary>> {
	return request<Envelope<CommitSummary>>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/commits`,
		{ query }
	);
}

export function getCommit(owner: string, repo: string, sha: string): Promise<CommitDetail> {
	return request<CommitDetail>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/commits/${encodeURIComponent(sha)}`
	);
}

export function getContents(
	owner: string,
	repo: string,
	path: string,
	query?: { ref?: string }
): Promise<ContentsResponse> {
	const clean = path.replace(/^\/+/, '');
	return request<ContentsResponse>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/contents/${clean}`,
		{ query }
	);
}

export function rawUrl(owner: string, repo: string, ref: string, path: string): string {
	const clean = path.replace(/^\/+/, '');
	return `${BASE}/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/raw/${encodeURIComponent(ref)}/${clean}`;
}

export function getReadme(
	owner: string,
	repo: string,
	query?: { ref?: string }
): Promise<ReadmeResponse> {
	return request<ReadmeResponse>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/readme`,
		{ query }
	);
}

export function listIssues(
	owner: string,
	repo: string,
	query?: { state?: IssueState | 'all'; page?: number; per_page?: number }
): Promise<Envelope<Issue>> {
	return request<Envelope<Issue>>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues`,
		{ query }
	);
}

export function createIssue(
	owner: string,
	repo: string,
	input: { title: string; body?: string }
): Promise<Issue> {
	return request<Issue>(`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues`, {
		method: 'POST',
		body: input
	});
}

export function getIssue(owner: string, repo: string, number: number): Promise<Issue> {
	return request<Issue>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${number}`
	);
}

export function updateIssue(
	owner: string,
	repo: string,
	number: number,
	input: { title?: string; body?: string; state?: IssueState }
): Promise<Issue> {
	return request<Issue>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${number}`,
		{ method: 'PATCH', body: input }
	);
}

export function listComments(
	owner: string,
	repo: string,
	number: number,
	query?: { page?: number; per_page?: number }
): Promise<Envelope<IssueComment>> {
	return request<Envelope<IssueComment>>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${number}/comments`,
		{ query }
	);
}

export function createComment(
	owner: string,
	repo: string,
	number: number,
	input: { body: string }
): Promise<IssueComment> {
	return request<IssueComment>(
		`/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/issues/${number}/comments`,
		{ method: 'POST', body: input }
	);
}

export const api = {
	unwrapList,
	register,
	login,
	logout,
	getSession,
	getUser,
	updateUser,
	getUserByName,
	getUserRepos,
	listTokens,
	createToken,
	deleteToken,
	listKeys,
	addKey,
	deleteKey,
	listRepos,
	createRepo,
	getRepo,
	updateRepo,
	deleteRepo,
	listBranches,
	listTags,
	getStats,
	listCommits,
	getCommit,
	getContents,
	rawUrl,
	getReadme,
	listIssues,
	createIssue,
	getIssue,
	updateIssue,
	listComments,
	createComment
};
