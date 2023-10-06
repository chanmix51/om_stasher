--
-- PostgreSQL database dump
--

-- Dumped from database version 15.4 (Ubuntu 15.4-1.pgdg22.04+1)
-- Dumped by pg_dump version 15.3 (Debian 15.3-0+deb12u1)

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

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: thought; Type: TABLE; Schema: thought; Owner: greg
--

CREATE TABLE thought.thought (
    thought_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    parent_thought_id uuid,
    keywords text[] DEFAULT ARRAY[]::text[] NOT NULL,
    categories public.ltree[] DEFAULT ARRAY[]::public.ltree[] NOT NULL,
    sources jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    content jsonb NOT NULL
);


ALTER TABLE thought.thought OWNER TO greg;

--
-- Data for Name: thought; Type: TABLE DATA; Schema: thought; Owner: greg
--

COPY thought.thought (thought_id, parent_thought_id, keywords, categories, sources, created_at, content) FROM stdin;
\.


--
-- Name: thought thought_pkey; Type: CONSTRAINT; Schema: thought; Owner: greg
--

ALTER TABLE ONLY thought.thought
    ADD CONSTRAINT thought_pkey PRIMARY KEY (thought_id);


--
-- Name: thought thought_parent_thought_id_fkey; Type: FK CONSTRAINT; Schema: thought; Owner: greg
--

ALTER TABLE ONLY thought.thought
    ADD CONSTRAINT thought_parent_thought_id_fkey FOREIGN KEY (parent_thought_id) REFERENCES thought.thought(thought_id);


--
-- PostgreSQL database dump complete
--

