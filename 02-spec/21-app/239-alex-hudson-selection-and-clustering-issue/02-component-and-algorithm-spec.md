# Component & Algorithm Specification: Alex Hudson Selection and Clustering

## 1. Algorithm Refinement: 4-Tier Candidate Selection with Fleet Awareness

### 1.1 Exclusion Pipeline
Before any candidate account is evaluated for Fast-Forward or Smart-Switch, the exclusion set $E$ is compiled:
$$E = E_{\text{disabled}} \cup E_{\text{current}} \cup E_{\text{local\_active}} \cup E_{\text{fleet\_active}} \cup E_{\text{weekly\_floor}}$$

Where:
- $E_{\text{disabled}}$: Accounts marked `disabled` or `proxy_disabled`.
- $E_{\text{current}}$: The account currently bound to the target instance being rotated.
- $E_{\text{local\_active}}$: Accounts bound to any other running sibling instance on the local host.
- $E_{\text{fleet\_active}}$: Accounts actively bound to any instance or reporting active usage on any remote node in the cluster (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) via `query_fleet_machines()` or cached fleet summaries.
- $E_{\text{weekly\_floor}}$: Accounts with `weekly_quota_percent < min_weekly_quota_percent` (default 15.0%).

### 1.2 Decision Ladder
1. **Primary Tier (100% 4h Headroom & Clean 4h Usage)**:
   - `rolling_4h_quota_percent >= 99.5`
   - Clean 4h fleet usage: `now - last_used >= 14400` or `last_used == 0`
   - Sorted ascending by `reset_epoch_seconds` (closest time to expire).
   - Tied candidates broken by higher `weekly_quota_percent`, then higher `tier_multiplier`.
2. **Degraded Fallback Tier (Highest Combined Credits)**:
   - Scored via: $S = \text{tier\_multiplier} \times (\text{weekly\_quota\_percent} + \text{rolling\_4h\_quota\_percent})$.
   - Sorted descending by score $S$.
3. **Transparency Telemetry ("Let It Show")**:
   - Return detailed `FfResolutionReport`:
     ```json
     {
       "selected_account": "user@example.com",
       "tier": "PrimaryFullHeadroomClosestExpiry",
       "score": 195.4,
       "reason": "Primary candidate with 100% 4h headroom and earliest reset at 17:30",
       "fleet_excluded_accounts": ["alex.hudson.riseup@gmail.com (active on remote node w1)"],
       "candidates_evaluated_count": 8,
       "eligible_candidates_count": 3
     }
     ```

## 2. Cluster Fleet Node Resolution & W2/W3 Parity

### 2.1 Dynamic Local Node Alias Resolution
In `src-tauri/src/modules/supabase_sync/node.rs`:
1. **Check GitMap Variables**: Inspect `~/.gitmap/variables.json` for `global.machine.alias` or `global.machine.name`. If defined, use it directly.
2. **Cluster-IP Primacy**: If local IP is valid (`!= 127.0.0.1`), match against `GITMAP_CLUSTER_NODES`:
   - `192.168.1.3` $\rightarrow$ `w1`
   - `192.168.1.7` $\rightarrow$ `w2`
   - `192.168.1.12` $\rightarrow$ `w3`
   - `192.168.1.13` $\rightarrow$ `w4`
   - `192.168.1.22` $\rightarrow$ `u1`
   - `192.168.1.20` $\rightarrow$ `main`
3. **Stored SSH Connections**: If not matched above, query `load_ssh_connections()`.
4. **User-Configured Alias**: Inspect `node_alias` in `supabase_config.json`.
5. **Hostname Fallback**: Computer name from OS environment.

### 2.2 Preserving All 6 Nodes in `query.rs`
1. Remove all hardcoded assignments:
   - Replace `local_machine.node_alias = "w1".to_string()` with `local_machine.node_alias = resolve_local_node_alias()`.
   - Remove `ip_address == "192.168.1.3"` from `is_local` determination; only compare against genuine `local_node_id` or `local_ip`.
2. When merging peer connections and `GITMAP_CLUSTER_NODES`:
   - An entry is only considered duplicate of the local machine if BOTH its IP matches `local_ip` AND its alias matches `local_alias`.
   - Ensure all 6 canonical nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) are guaranteed to exist in the output vector.
