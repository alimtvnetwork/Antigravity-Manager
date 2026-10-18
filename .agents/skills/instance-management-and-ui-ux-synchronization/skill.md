---
name: instance-management-and-ui-ux-synchronization
description: >-
  Specialized skill for managing Antigravity instance lifecycle, PID termination, switch account synchronization, dark-glass UI/UX, instance audit trails, and CLI self-instance preservation.
---

# Instance Management and UI/UX Synchronization

## Overview
This skill guides the end-to-end resolution of issues across:
1. **List View & Path Tail Alignment**: Displaying the executable path ending and data path ending together on the exact same line, and rendering the process PID clearly as `PID: <pid>`.
2. **Account Switching & Process Termination**: Deterministic PID closure/killing before switching accounts or reloading instances, fixing the "Stop Instance" action.
3. **Switch Account Dialog UI/UX**: Elevating the 4-option / switch account modal to modern dark-glass styling consistent with the design system.
4. **Instance Audit Trail**: Full historical visibility of switch events, start/stop actions, and status changes in the audit modal.
5. **Caller IDE Self-Preservation & CLI Parity**: Auto-detecting caller instances (`detect_caller_instance_id()`) to prevent closing the developer's active session, and supporting transient test sandbox commands (`agm instances test-create` / `test-clean`).
