export interface FleetInstanceSummary {
    profile_id: string;
    profile_name: string;
    is_running: boolean;
    bound_account_id?: string;
    bound_account_email?: string;
}

export interface FleetMachineInfo {
    node_id: string;
    node_alias: string;
    os_info?: string;
    ip_address: string;
    is_online: boolean;
    last_heartbeat_timestamp: number;
    uptime_seconds: number;
    in_flight_prompts_count: number;
    active_instances: FleetInstanceSummary[];
    bound_emails: string[];
}
