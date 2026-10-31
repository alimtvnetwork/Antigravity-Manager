//! Workspace Storage Hash Path Resolver.
//! Computes deterministic FNV-1a directory hashes matching Antigravity IDE's internal `workspaceStorage/<hash>` naming convention.

use std::path::Path;

/// Computes a 64-bit FNV-1a hash of a normalized path string.
pub fn compute_fnv1a_hash_str(input: &str) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{:016x}", hash)
}

/// Generates the deterministic workspace folder identifier for an IDE project path.
pub fn resolve_workspace_storage_id(project_path: &Path) -> String {
    let normalized = project_path
        .to_string_lossy()
        .to_string()
        .replace('\\', "/")
        .to_lowercase();

    let hash_hex = compute_fnv1a_hash_str(&normalized);

    let folder_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    format!("{}-{}", folder_name, &hash_hex[..8])
}
