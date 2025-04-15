// @generated automatically by Diesel CLI.

diesel::table! {
    admins (id) {
        id -> Int4,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    job_assignments (id) {
        id -> Int4,
        job_id -> Int4,
        worker_id -> Int4,
        assigned_at -> Timestamp,
        started_at -> Nullable<Timestamp>,
        finished_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    job_metrics (id) {
        id -> Int4,
        job_id -> Int4,
        worker_id -> Int4,
        duration_sec -> Nullable<Int4>,
        cpu_usage_pct -> Nullable<Float4>,
        mem_usage_mb -> Nullable<Float4>,
        exit_code -> Nullable<Int4>,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    job_results (id) {
        id -> Int4,
        job_id -> Int4,
        stdout -> Nullable<Text>,
        files -> Nullable<Array<Nullable<Text>>>,
        saved_at -> Timestamp,
    }
}

diesel::table! {
    jobs (id) {
        id -> Int4,
        admin_id -> Int4,
        job_name -> Text,
        image_url -> Text,
        #[max_length = 64]
        image_format -> Varchar,
        docker_flags -> Nullable<Array<Nullable<Text>>>,
        #[max_length = 64]
        output_type -> Varchar,
        output_paths -> Nullable<Array<Nullable<Text>>>,
        #[max_length = 64]
        schedule_type -> Varchar,
        cron_expression -> Nullable<Text>,
        notes -> Nullable<Text>,
        #[max_length = 64]
        state -> Varchar,
        error_message -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    worker_status (id) {
        id -> Int4,
        worker_id -> Int4,
        #[max_length = 64]
        status -> Varchar,
        last_heartbeat -> Nullable<Timestamp>,
       active_job_id -> Nullable<Int4>,
        uptime_sec -> Nullable<Int4>,
        load_avg -> Nullable<Array<Nullable<Float4>>>,
        last_error -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    workers (id) {
        id -> Int4,
        admin_id -> Int4,
        label -> Text,
        ip_address -> Text,
        hostname -> Text,
        ssh_user -> Text,
        ssh_key -> Text,
        docker_version -> Text,
        arch -> Text,
        #[max_length = 64]
        os -> Varchar,
        tags -> Nullable<Array<Nullable<Text>>>,
        created_at -> Timestamp,
        last_seen_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(job_assignments -> jobs (job_id));
diesel::joinable!(job_assignments -> workers (worker_id));
diesel::joinable!(job_metrics -> jobs (job_id));
diesel::joinable!(job_metrics -> workers (worker_id));
diesel::joinable!(job_results -> jobs (job_id));
diesel::joinable!(jobs -> admins (admin_id));
diesel::joinable!(worker_status -> jobs (active_job_id));
diesel::joinable!(worker_status -> workers (worker_id));
diesel::joinable!(workers -> admins (admin_id));

diesel::allow_tables_to_appear_in_same_query!(
    admins,
    job_assignments,
    job_metrics,
    job_results,
    jobs,
    worker_status,
    workers,
);
