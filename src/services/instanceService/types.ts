
export interface InstanceConfig {
    id: string;
    name: string;
    data_dir: string;
    executable_path?: string;
    extensions_dir?: string;
    bound_account_id?: string;
    bound_email?: string;
    created_at: number;
    last_used: number;
    is_default: boolean;
    seq_num?: number;
}

export interface InstanceStatus {
    config: InstanceConfig;
    is_running: boolean;
    pid?: number;
    memory_mb?: number;
}
