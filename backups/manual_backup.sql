--
-- PostgreSQL database dump
--

-- Dumped from database version 15.12
-- Dumped by pg_dump version 15.12

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: diesel_manage_updated_at(regclass); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;


ALTER FUNCTION public.diesel_manage_updated_at(_tbl regclass) OWNER TO postgres;

--
-- Name: diesel_set_updated_at(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.diesel_set_updated_at() OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: __diesel_schema_migrations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.__diesel_schema_migrations (
    version character varying(50) NOT NULL,
    run_on timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.__diesel_schema_migrations OWNER TO postgres;

--
-- Name: job_assignments; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.job_assignments (
    id integer NOT NULL,
    job_id integer NOT NULL,
    worker_id integer NOT NULL,
    assigned_at timestamp without time zone DEFAULT now() NOT NULL,
    started_at timestamp without time zone,
    finished_at timestamp without time zone
);


ALTER TABLE public.job_assignments OWNER TO postgres;

--
-- Name: job_assignments_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.job_assignments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.job_assignments_id_seq OWNER TO postgres;

--
-- Name: job_assignments_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.job_assignments_id_seq OWNED BY public.job_assignments.id;


--
-- Name: job_metrics; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.job_metrics (
    id integer NOT NULL,
    job_id integer NOT NULL,
    worker_id integer NOT NULL,
    duration_sec integer,
    cpu_usage_pct real,
    mem_usage_mb real,
    exit_code integer,
    "timestamp" timestamp without time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.job_metrics OWNER TO postgres;

--
-- Name: job_metrics_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.job_metrics_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.job_metrics_id_seq OWNER TO postgres;

--
-- Name: job_metrics_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.job_metrics_id_seq OWNED BY public.job_metrics.id;


--
-- Name: job_results; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.job_results (
    id integer NOT NULL,
    job_id integer NOT NULL,
    stdout text,
    files text[],
    saved_at timestamp without time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.job_results OWNER TO postgres;

--
-- Name: job_results_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.job_results_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.job_results_id_seq OWNER TO postgres;

--
-- Name: job_results_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.job_results_id_seq OWNED BY public.job_results.id;


--
-- Name: job_results_job_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.job_results_job_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.job_results_job_id_seq OWNER TO postgres;

--
-- Name: job_results_job_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.job_results_job_id_seq OWNED BY public.job_results.job_id;


--
-- Name: jobs; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.jobs (
    id integer NOT NULL,
    user_id integer NOT NULL,
    job_name text NOT NULL,
    image_url text NOT NULL,
    image_format character varying(64) NOT NULL,
    docker_flags text[],
    output_type character varying(64) NOT NULL,
    output_paths text[],
    schedule_type character varying(64) NOT NULL,
    cron_expression text,
    notes text,
    state character varying(64) NOT NULL,
    error_message text,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.jobs OWNER TO postgres;

--
-- Name: jobs_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.jobs_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.jobs_id_seq OWNER TO postgres;

--
-- Name: jobs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.jobs_id_seq OWNED BY public.jobs.id;


--
-- Name: jobs_user_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.jobs_user_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.jobs_user_id_seq OWNER TO postgres;

--
-- Name: jobs_user_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.jobs_user_id_seq OWNED BY public.jobs.user_id;


--
-- Name: logs; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.logs (
    id integer NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    level character varying(64) NOT NULL,
    module character varying(64) NOT NULL,
    action character varying(64) NOT NULL,
    expires_at timestamp without time zone NOT NULL,
    client_connected_ip text,
    client_connected_username text,
    job_submitted_job_id integer,
    job_submitted_from_module character varying(64),
    job_submitted_to_module character varying(64),
    job_completed_job_id integer,
    job_completed_success boolean,
    custom_msg text
);


ALTER TABLE public.logs OWNER TO postgres;

--
-- Name: logs_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.logs_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.logs_id_seq OWNER TO postgres;

--
-- Name: logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.logs_id_seq OWNED BY public.logs.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.users (
    id integer NOT NULL,
    username text NOT NULL,
    email text NOT NULL,
    password_hash text NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.users OWNER TO postgres;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.users_id_seq OWNER TO postgres;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: worker_status; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.worker_status (
    id integer NOT NULL,
    worker_id integer NOT NULL,
    status character varying(64) NOT NULL,
    last_heartbeat timestamp without time zone,
    active_job_id integer,
    uptime_sec integer,
    load_avg real[],
    last_error text,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.worker_status OWNER TO postgres;

--
-- Name: worker_status_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.worker_status_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.worker_status_id_seq OWNER TO postgres;

--
-- Name: worker_status_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.worker_status_id_seq OWNED BY public.worker_status.id;


--
-- Name: worker_status_worker_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.worker_status_worker_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.worker_status_worker_id_seq OWNER TO postgres;

--
-- Name: worker_status_worker_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.worker_status_worker_id_seq OWNED BY public.worker_status.worker_id;


--
-- Name: workers; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.workers (
    id integer NOT NULL,
    user_id integer NOT NULL,
    label text NOT NULL,
    ip_address text NOT NULL,
    hostname text NOT NULL,
    ssh_user text NOT NULL,
    ssh_key text NOT NULL,
    docker_version text NOT NULL,
    arch text NOT NULL,
    os character varying(64) NOT NULL,
    tags text[] DEFAULT '{}'::text[],
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    last_seen_at timestamp without time zone
);


ALTER TABLE public.workers OWNER TO postgres;

--
-- Name: workers_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.workers_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.workers_id_seq OWNER TO postgres;

--
-- Name: workers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.workers_id_seq OWNED BY public.workers.id;


--
-- Name: workers_user_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.workers_user_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.workers_user_id_seq OWNER TO postgres;

--
-- Name: workers_user_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.workers_user_id_seq OWNED BY public.workers.user_id;


--
-- Name: job_assignments id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_assignments ALTER COLUMN id SET DEFAULT nextval('public.job_assignments_id_seq'::regclass);


--
-- Name: job_metrics id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_metrics ALTER COLUMN id SET DEFAULT nextval('public.job_metrics_id_seq'::regclass);


--
-- Name: job_results id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_results ALTER COLUMN id SET DEFAULT nextval('public.job_results_id_seq'::regclass);


--
-- Name: job_results job_id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_results ALTER COLUMN job_id SET DEFAULT nextval('public.job_results_job_id_seq'::regclass);


--
-- Name: jobs id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.jobs ALTER COLUMN id SET DEFAULT nextval('public.jobs_id_seq'::regclass);


--
-- Name: jobs user_id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.jobs ALTER COLUMN user_id SET DEFAULT nextval('public.jobs_user_id_seq'::regclass);


--
-- Name: logs id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.logs ALTER COLUMN id SET DEFAULT nextval('public.logs_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: worker_status id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.worker_status ALTER COLUMN id SET DEFAULT nextval('public.worker_status_id_seq'::regclass);


--
-- Name: worker_status worker_id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.worker_status ALTER COLUMN worker_id SET DEFAULT nextval('public.worker_status_worker_id_seq'::regclass);


--
-- Name: workers id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workers ALTER COLUMN id SET DEFAULT nextval('public.workers_id_seq'::regclass);


--
-- Name: workers user_id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workers ALTER COLUMN user_id SET DEFAULT nextval('public.workers_user_id_seq'::regclass);


--
-- Data for Name: __diesel_schema_migrations; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.__diesel_schema_migrations (version, run_on) FROM stdin;
00000000000000	2025-04-27 14:36:40.66996
20250414155632	2025-04-27 14:36:40.672935
20250415175449	2025-04-27 14:36:40.678606
20250415175554	2025-04-27 14:36:40.68302
20250415175715	2025-04-27 14:36:40.690043
20250417140624	2025-04-27 14:36:40.699485
\.


--
-- Data for Name: job_assignments; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.job_assignments (id, job_id, worker_id, assigned_at, started_at, finished_at) FROM stdin;
\.


--
-- Data for Name: job_metrics; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.job_metrics (id, job_id, worker_id, duration_sec, cpu_usage_pct, mem_usage_mb, exit_code, "timestamp") FROM stdin;
\.


--
-- Data for Name: job_results; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.job_results (id, job_id, stdout, files, saved_at) FROM stdin;
\.


--
-- Data for Name: jobs; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.jobs (id, user_id, job_name, image_url, image_format, docker_flags, output_type, output_paths, schedule_type, cron_expression, notes, state, error_message, created_at, updated_at) FROM stdin;
1	1	Helloworld	http:lalal	DockerRegistry	\N	Files	{result,lolalla}	Once	\N	\N	Queued	\N	2025-04-28 11:37:04.567136	2025-04-28 11:37:42.039033
2	1	hejsan	diejde	DockerRegistry	\N	Stdout	\N	Once	\N	\N	Submitted	\N	2025-04-28 11:44:00.768558	2025-04-28 11:44:00.773763
\.


--
-- Data for Name: logs; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.logs (id, created_at, level, module, action, expires_at, client_connected_ip, client_connected_username, job_submitted_job_id, job_submitted_from_module, job_submitted_to_module, job_completed_job_id, job_completed_success, custom_msg) FROM stdin;
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.users (id, username, email, password_hash, created_at) FROM stdin;
1	Admin	Admin@gmail.com	$argon2id$v=19$m=4096,t=3,p=1$a8Y5eexZWMdy/zjFqOSHlQ$EhfBuXrp10d0UTGFscOsASDWHJLvqNEdkVfXjE8alyk	2025-04-27 14:48:02.000192
\.


--
-- Data for Name: worker_status; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.worker_status (id, worker_id, status, last_heartbeat, active_job_id, uptime_sec, load_avg, last_error, updated_at) FROM stdin;
\.


--
-- Data for Name: workers; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.workers (id, user_id, label, ip_address, hostname, ssh_user, ssh_key, docker_version, arch, os, tags, created_at, last_seen_at) FROM stdin;
1	1	LinuxDator	127.0.0.1	localhost	root	~/.ssh/id_rsa	20.10.7	x86_64	Linux	{}	2025-04-28 11:36:05.667064	\N
\.


--
-- Name: job_assignments_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.job_assignments_id_seq', 1, false);


--
-- Name: job_metrics_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.job_metrics_id_seq', 1, false);


--
-- Name: job_results_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.job_results_id_seq', 1, false);


--
-- Name: job_results_job_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.job_results_job_id_seq', 1, false);


--
-- Name: jobs_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.jobs_id_seq', 2, true);


--
-- Name: jobs_user_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.jobs_user_id_seq', 1, false);


--
-- Name: logs_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.logs_id_seq', 1, false);


--
-- Name: users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.users_id_seq', 1, true);


--
-- Name: worker_status_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.worker_status_id_seq', 1, false);


--
-- Name: worker_status_worker_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.worker_status_worker_id_seq', 1, false);


--
-- Name: workers_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.workers_id_seq', 1, true);


--
-- Name: workers_user_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.workers_user_id_seq', 1, false);


--
-- Name: __diesel_schema_migrations __diesel_schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.__diesel_schema_migrations
    ADD CONSTRAINT __diesel_schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: job_assignments job_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_assignments
    ADD CONSTRAINT job_assignments_pkey PRIMARY KEY (id);


--
-- Name: job_metrics job_metrics_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_metrics
    ADD CONSTRAINT job_metrics_pkey PRIMARY KEY (id);


--
-- Name: job_results job_results_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_results
    ADD CONSTRAINT job_results_pkey PRIMARY KEY (id);


--
-- Name: jobs jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.jobs
    ADD CONSTRAINT jobs_pkey PRIMARY KEY (id);


--
-- Name: logs logs_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.logs
    ADD CONSTRAINT logs_pkey PRIMARY KEY (id);


--
-- Name: job_metrics unique_job_worker_pair; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_metrics
    ADD CONSTRAINT unique_job_worker_pair UNIQUE (job_id, worker_id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: worker_status worker_status_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.worker_status
    ADD CONSTRAINT worker_status_pkey PRIMARY KEY (id);


--
-- Name: worker_status worker_status_worker_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.worker_status
    ADD CONSTRAINT worker_status_worker_id_key UNIQUE (worker_id);


--
-- Name: workers workers_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workers
    ADD CONSTRAINT workers_pkey PRIMARY KEY (id);


--
-- Name: logs fk_job_completed; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.logs
    ADD CONSTRAINT fk_job_completed FOREIGN KEY (job_completed_job_id) REFERENCES public.jobs(id) ON DELETE CASCADE;


--
-- Name: logs fk_job_submitted; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.logs
    ADD CONSTRAINT fk_job_submitted FOREIGN KEY (job_submitted_job_id) REFERENCES public.jobs(id) ON DELETE CASCADE;


--
-- Name: job_assignments job_assignments_job_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_assignments
    ADD CONSTRAINT job_assignments_job_id_fkey FOREIGN KEY (job_id) REFERENCES public.jobs(id) ON DELETE CASCADE;


--
-- Name: job_assignments job_assignments_worker_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_assignments
    ADD CONSTRAINT job_assignments_worker_id_fkey FOREIGN KEY (worker_id) REFERENCES public.workers(id) ON DELETE CASCADE;


--
-- Name: job_metrics job_metrics_job_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_metrics
    ADD CONSTRAINT job_metrics_job_id_fkey FOREIGN KEY (job_id) REFERENCES public.jobs(id) ON DELETE CASCADE;


--
-- Name: job_metrics job_metrics_worker_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_metrics
    ADD CONSTRAINT job_metrics_worker_id_fkey FOREIGN KEY (worker_id) REFERENCES public.workers(id) ON DELETE CASCADE;


--
-- Name: job_results job_results_job_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.job_results
    ADD CONSTRAINT job_results_job_id_fkey FOREIGN KEY (job_id) REFERENCES public.jobs(id) ON DELETE CASCADE;


--
-- Name: jobs jobs_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.jobs
    ADD CONSTRAINT jobs_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: worker_status worker_status_active_job_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.worker_status
    ADD CONSTRAINT worker_status_active_job_id_fkey FOREIGN KEY (active_job_id) REFERENCES public.jobs(id) ON DELETE SET NULL;


--
-- Name: worker_status worker_status_worker_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.worker_status
    ADD CONSTRAINT worker_status_worker_id_fkey FOREIGN KEY (worker_id) REFERENCES public.workers(id) ON DELETE CASCADE;


--
-- Name: workers workers_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.workers
    ADD CONSTRAINT workers_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

