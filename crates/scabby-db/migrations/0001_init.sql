-- Scabby schema, first cut.
--
-- Conventions: identity bigint keys, timestamptz in UTC, and any value that has to be unique
-- ignoring case gets a generated lowercase column with the unique index on it. Generated columns
-- keep the comparison in the database, where a query cannot forget to lower() it.

create table users (
    id             bigint generated always as identity primary key,
    username       text not null,
    username_lower text not null generated always as (lower(username)) stored,
    display_name   text,
    email          text not null,
    email_lower    text not null generated always as (lower(email)) stored,
    password_hash  text not null,
    bio            text,
    avatar_url     text,
    is_admin       boolean not null default false,
    created_at     timestamptz not null default now(),
    updated_at     timestamptz not null default now(),
    constraint users_username_key unique (username_lower),
    constraint users_email_key unique (email_lower)
);

create table sessions (
    id           bigint generated always as identity primary key,
    user_id      bigint not null references users (id) on delete cascade,
    token_hash   bytea not null,
    user_agent   text,
    created_at   timestamptz not null default now(),
    last_seen_at timestamptz not null default now(),
    expires_at   timestamptz not null,
    constraint sessions_token_key unique (token_hash)
);

create index sessions_user_id_idx on sessions (user_id);
create index sessions_expires_at_idx on sessions (expires_at);

-- Personal access tokens. Only the hash and a display prefix are stored, so a leaked database
-- cannot be replayed against the API.
create table api_tokens (
    id           bigint generated always as identity primary key,
    user_id      bigint not null references users (id) on delete cascade,
    name         text not null,
    prefix       text not null,
    token_hash   bytea not null,
    scopes       text[] not null default '{repo,user}',
    created_at   timestamptz not null default now(),
    last_used_at timestamptz,
    expires_at   timestamptz,
    constraint api_tokens_token_key unique (token_hash)
);

create index api_tokens_user_id_idx on api_tokens (user_id);

create table ssh_keys (
    id           bigint generated always as identity primary key,
    user_id      bigint not null references users (id) on delete cascade,
    title        text not null,
    public_key   text not null,
    fingerprint  text not null,
    created_at   timestamptz not null default now(),
    last_used_at timestamptz,
    constraint ssh_keys_fingerprint_key unique (fingerprint)
);

create index ssh_keys_user_id_idx on ssh_keys (user_id);

create table repositories (
    id             bigint generated always as identity primary key,
    owner_id       bigint not null references users (id) on delete cascade,
    name           text not null,
    name_lower     text not null generated always as (lower(name)) stored,
    description    text,
    visibility     text not null default 'public',
    default_branch text not null default 'main',
    size_bytes     bigint not null default 0,
    created_at     timestamptz not null default now(),
    updated_at     timestamptz not null default now(),
    constraint repositories_visibility_check check (visibility in ('public', 'private')),
    constraint repositories_owner_name_key unique (owner_id, name_lower)
);

create index repositories_owner_id_idx on repositories (owner_id);
create index repositories_visibility_idx on repositories (visibility);

-- Issue numbers are per repository, so the unique key is (repo_id, number) and the allocator locks
-- the repository row before reading max(number).
create table issues (
    id         bigint generated always as identity primary key,
    repo_id    bigint not null references repositories (id) on delete cascade,
    number     integer not null,
    title      text not null,
    body       text,
    state      text not null default 'open',
    author_id  bigint not null references users (id) on delete cascade,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    closed_at  timestamptz,
    constraint issues_state_check check (state in ('open', 'closed')),
    constraint issues_repo_number_key unique (repo_id, number)
);

create index issues_repo_state_idx on issues (repo_id, state, number desc);

create table issue_comments (
    id         bigint generated always as identity primary key,
    issue_id   bigint not null references issues (id) on delete cascade,
    author_id  bigint not null references users (id) on delete cascade,
    body       text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index issue_comments_issue_id_idx on issue_comments (issue_id, created_at);

-- Activity feed. payload is reserved for detail the feed does not render yet.
create table events (
    id         bigint generated always as identity primary key,
    actor_id   bigint references users (id) on delete set null,
    repo_id    bigint references repositories (id) on delete cascade,
    kind       text not null,
    payload    jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);

create index events_created_at_idx on events (created_at desc, id desc);
create index events_actor_created_at_idx on events (actor_id, created_at desc);
