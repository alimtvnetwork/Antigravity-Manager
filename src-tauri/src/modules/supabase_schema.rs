//! Supabase Schema Definitions and Migration Module
//! Declares tables, indices, and stored functions for Root and Secondary databases.

#![allow(dead_code)]

/// SQL schema statements for the Root Database (Node Registry & Workspace Leases)
pub const ROOT_DB_SCHEMA_SQL: &str = r#"
-- 1. Nodes Registry Table
CREATE TABLE IF NOT EXISTS public.nodes (
    id TEXT PRIMARY KEY,
    alias TEXT NOT NULL DEFAULT '',
    ip_address TEXT NOT NULL DEFAULT '',
    uptime_seconds BIGINT NOT NULL DEFAULT 0,
    project_count INT NOT NULL DEFAULT 0,
    last_heartbeat_at BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'online'
);

CREATE INDEX IF NOT EXISTS idx_nodes_heartbeat ON public.nodes (last_heartbeat_at DESC);

-- 2. Instance Profiles Status Table
CREATE TABLE IF NOT EXISTS public.instance_profiles (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    profile_name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    quota_percent INT NOT NULL DEFAULT 100,
    status TEXT NOT NULL DEFAULT 'idle',
    updated_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_profiles_node ON public.instance_profiles (node_id);

-- 3. Distributed Workspace & Account Leases Table (Collision Prevention)
CREATE TABLE IF NOT EXISTS public.workspace_leases (
    account_id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    node_alias TEXT NOT NULL DEFAULT '',
    profile_name TEXT NOT NULL DEFAULT '',
    leased_at BIGINT NOT NULL DEFAULT 0,
    expires_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_leases_expires ON public.workspace_leases (expires_at DESC);
CREATE INDEX IF NOT EXISTS idx_leases_node ON public.workspace_leases (node_id);

-- 4. Atomic Lease Acquisition Stored Function
CREATE OR REPLACE FUNCTION public.acquire_workspace_lease(
    p_account_id TEXT,
    p_node_id TEXT,
    p_node_alias TEXT,
    p_profile_name TEXT,
    p_ttl_seconds INT
) RETURNS JSONB AS $$
DECLARE
    v_now BIGINT := EXTRACT(EPOCH FROM NOW())::BIGINT;
    v_expires BIGINT := v_now + p_ttl_seconds;
    v_current_lease RECORD;
BEGIN
    SELECT * INTO v_current_lease FROM public.workspace_leases WHERE account_id = p_account_id;
    
    IF FOUND THEN
        IF v_current_lease.expires_at > v_now AND v_current_lease.node_id <> p_node_id THEN
            RETURN jsonb_build_object(
                'is_success', false,
                'error', 'Lease currently held by another node',
                'owner_node_id', v_current_lease.node_id,
                'owner_alias', v_current_lease.node_alias,
                'expires_at', v_current_lease.expires_at
            );
        END IF;
    END IF;

    INSERT INTO public.workspace_leases (account_id, node_id, node_alias, profile_name, leased_at, expires_at)
    VALUES (p_account_id, p_node_id, p_node_alias, p_profile_name, v_now, v_expires)
    ON CONFLICT (account_id) DO UPDATE SET
        node_id = EXCLUDED.node_id,
        node_alias = EXCLUDED.node_alias,
        profile_name = EXCLUDED.profile_name,
        leased_at = EXCLUDED.leased_at,
        expires_at = EXCLUDED.expires_at;

    RETURN jsonb_build_object(
        'is_success', true,
        'account_id', p_account_id,
        'expires_at', v_expires
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
"#;

/// SQL schema statements for Secondary Database (Command Queue & Telemetry)
pub const SECONDARY_DB_SCHEMA_SQL: &str = r#"
-- 1. Inbound Command Queue Table
CREATE TABLE IF NOT EXISTS public.command_queue (
    id TEXT PRIMARY KEY,
    target_node_id TEXT NOT NULL DEFAULT '*',
    source TEXT NOT NULL DEFAULT 'email',
    command_text TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    created_at BIGINT NOT NULL DEFAULT 0,
    started_at BIGINT NOT NULL DEFAULT 0,
    completed_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_command_target ON public.command_queue (target_node_id, status);
CREATE INDEX IF NOT EXISTS idx_command_created ON public.command_queue (created_at DESC);

-- 2. Command Telemetry Logs Table
CREATE TABLE IF NOT EXISTS public.command_telemetry (
    id TEXT PRIMARY KEY,
    command_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    stdout TEXT NOT NULL DEFAULT '',
    stderr TEXT NOT NULL DEFAULT '',
    exit_code INT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_telemetry_cmd ON public.command_telemetry (command_id);
CREATE INDEX IF NOT EXISTS idx_telemetry_created ON public.command_telemetry (created_at DESC);

-- 3. Endpoint Health & Storage Tracker Table
CREATE TABLE IF NOT EXISTS public.endpoint_health (
    id TEXT PRIMARY KEY,
    endpoint_url TEXT NOT NULL,
    storage_used_bytes BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_checked_at BIGINT NOT NULL DEFAULT 0
);

-- 4. FIFO Auto-Pruning Stored Function
CREATE OR REPLACE FUNCTION public.prune_old_commands(p_keep_limit INT)
RETURNS INT AS $$
DECLARE
    v_deleted_count INT := 0;
BEGIN
    WITH to_delete AS (
        SELECT id FROM public.command_queue
        WHERE status IN ('completed', 'failed')
        ORDER BY created_at ASC
        OFFSET p_keep_limit
    )
    DELETE FROM public.command_queue
    WHERE id IN (SELECT id FROM to_delete);
    
    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    RETURN v_deleted_count;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
"#;

/// Returns complete schema SQL for a specific database role
pub fn get_schema_sql(role: &str) -> &'static str {
    if role == "root" {
        ROOT_DB_SCHEMA_SQL
    } else {
        SECONDARY_DB_SCHEMA_SQL
    }
}
