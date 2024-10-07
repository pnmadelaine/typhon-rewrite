// @generated automatically by Diesel CLI.

diesel::table! {
    builds (id) {
        id -> Int8,
        drv -> Text,
        started_at -> Timestamp,
        ended_at -> Nullable<Timestamp>,
        success -> Nullable<Bool>,
    }
}

diesel::table! {
    jobs (id) {
        id -> Int4,
        jobset_id -> Int4,
        name -> Text,
        build_id -> Nullable<Int8>,
        success -> Nullable<Bool>,
        canceled_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    jobsets (id) {
        id -> Int4,
        project_lock_id -> Int4,
        name -> Nullable<Text>,
        source -> Text,
        lock -> Text,
        path -> Nullable<Text>,
        created_at -> Timestamp,
        evaluation_started_at -> Nullable<Timestamp>,
        evaluation_ended_at -> Nullable<Timestamp>,
        evaluation_success -> Nullable<Bool>,
        action_started_at -> Nullable<Timestamp>,
        action_ended_at -> Nullable<Timestamp>,
        action_success -> Nullable<Bool>,
        deployment_started_at -> Nullable<Timestamp>,
        deployment_ended_at -> Nullable<Timestamp>,
        deployment_success -> Nullable<Bool>,
        webhook_id -> Nullable<Int4>,
    }
}

diesel::table! {
    permissions (id) {
        id -> Int4,
        user_id -> Int4,
        project_id -> Int4,
        admin -> Bool,
    }
}

diesel::table! {
    project_locks (id) {
        id -> Int4,
        project_id -> Int4,
        lock -> Text,
        out_path -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    projects (id) {
        id -> Int4,
        name -> Text,
        source -> Nullable<Text>,
        path -> Nullable<Text>,
        enabled -> Bool,
    }
}

diesel::table! {
    secrets (id) {
        id -> Int4,
        project_id -> Int4,
        key -> Text,
        value -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Text,
        is_super_admin -> Bool,
    }
}

diesel::table! {
    webhooks (id) {
        id -> Int4,
        project_lock_id -> Int4,
        created_at -> Timestamp,
        started_at -> Nullable<Timestamp>,
        ended_at -> Nullable<Timestamp>,
        success -> Nullable<Bool>,
    }
}

diesel::joinable!(jobs -> builds (build_id));
diesel::joinable!(jobs -> jobsets (jobset_id));
diesel::joinable!(jobsets -> project_locks (project_lock_id));
diesel::joinable!(jobsets -> webhooks (webhook_id));
diesel::joinable!(permissions -> projects (project_id));
diesel::joinable!(permissions -> users (user_id));
diesel::joinable!(project_locks -> projects (project_id));
diesel::joinable!(secrets -> projects (project_id));
diesel::joinable!(webhooks -> project_locks (project_lock_id));

diesel::allow_tables_to_appear_in_same_query!(
    builds,
    jobs,
    jobsets,
    permissions,
    project_locks,
    projects,
    secrets,
    users,
    webhooks,
);
