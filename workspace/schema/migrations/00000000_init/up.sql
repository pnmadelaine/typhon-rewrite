create table builds (
    id bigint not null primary key,
    drv text not null,
    started_at timestamp not null,
    ended_at timestamp,
    success boolean
);

create table projects (
    id serial primary key,
    name text not null,
    source text,
    path text,
    enabled bool not null default true
);

create table project_locks (
    id serial primary key,
    project_id integer not null references projects (id),
    lock text not null,
    out_path text not null,
    created_at timestamp not null
);

create table secrets (
    id serial primary key,
    project_id integer not null references projects (id),
    key text not null,
    value text not null,
    created_at timestamp not null,
    unique (project_id, key)
);

create table webhooks (
    id integer not null primary key,
    project_lock_id integer not null references project_locks (id),
    created_at timestamp not null,
    started_at timestamp,
    ended_at timestamp,
    success boolean
);

create table jobsets (
    id integer not null primary key,
    project_lock_id integer not null references project_locks (id),
    name text,
    source text not null,
    lock text not null,
    path text,
    created_at timestamp not null,
    evaluation_started_at timestamp,
    evaluation_ended_at timestamp,
    evaluation_success boolean,
    action_started_at timestamp,
    action_ended_at timestamp,
    action_success boolean,
    deployment_started_at timestamp,
    deployment_ended_at timestamp,
    deployment_success boolean,
    webhook_id integer references webhooks (id)
);

create table jobs (
    id serial primary key,
    jobset_id integer not null references jobsets (id),
    name text not null,
    build_id bigint references builds (id),
    success boolean,
    canceled_at timestamp,
    unique (jobset_id, name)
);

create table users (
    id serial primary key,
    username text not null,
    is_super_admin boolean not null default false,
    unique (username)
);

create table permissions (
    id serial primary key,
    user_id integer not null references users (id),
    project_id integer not null references projects (id),
    admin boolean not null default false,
    unique (user_id, project_id)
);
