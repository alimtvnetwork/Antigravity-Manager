# Subtask 03: Cross-Machine Telegram Dispatching and GitMap SSH Mesh

## Intent
Ensure reliable prompt sending between machines (`192.168.1.3` w1 and `192.168.1.20` main/w2), resolve local machine alias accurately from GitMap configuration, and verify SSH port 22 connectivity.

## Remediation Details
1. IP Resolution: `node_label.rs` checks local IP `192.168.1.3` against `~/.gitmap/gitmap-ssh-nodes.json` and `GITMAP_CLUSTER_NODES`, resolving machine name to `w1 (collected from gitmap)`.
2. Telemetry Card: Telegram reports format `• Instance: <inst>` directly below `• Machine: <name>` with timestamp.
3. SSH Port 22 Relay: `ssh_manager.rs` and `send_flow.rs` execute prompt relays across cluster nodes.
