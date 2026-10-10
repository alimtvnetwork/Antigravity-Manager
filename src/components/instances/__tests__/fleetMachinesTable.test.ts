/**
 * Standalone assertion tests for the Remote Fleet Machines section
 * (src/components/instances/FleetMachinesTable.tsx).
 *
 * Covers the pure, user-visible logic:
 *  - maskEmailAddress: privacy masking of bound account emails
 *  - formatRelativeHeartbeat: "last seen" relative timestamps (sec + ms inputs)
 *  - local-node filtering: the local machine must never appear as a remote node
 *
 * Run: npx tsx src/components/instances/__tests__/fleetMachinesTable.test.ts
 *      npm run test -- fleetMachines
 */
import {
    maskEmailAddress,
    formatRelativeHeartbeat,
    type FleetMachineInfo,
} from '../FleetMachinesTable';

let passed = 0;
let failed = 0;

function test(description: string, fn: () => void): void {
    try {
        fn();
        passed++;
    } catch (e: unknown) {
        failed++;
        const msg = e instanceof Error ? e.message : String(e);
        console.error(`  FAIL: ${description} — ${msg}`);
    }
}

function assertEqual<T>(actual: T, expected: T): void {
    if (actual !== expected) {
        throw new Error(`expected "${expected}", got "${actual}"`);
    }
}

function makeMachine(node_id: string, node_alias: string): FleetMachineInfo {
    return {
        node_id,
        node_alias,
        os_info: 'TestOS',
        ip_address: '10.0.0.2',
        is_online: true,
        last_heartbeat_timestamp: Math.floor(Date.now() / 1000) - 30,
        uptime_seconds: 3600,
        in_flight_prompts_count: 0,
        active_instances: [],
        bound_emails: [],
    };
}

// ---------------------------------------------------------------------------
// maskEmailAddress
// ---------------------------------------------------------------------------

test('maskEmailAddress masks a normal email keeping first/last local chars', () => {
    assertEqual(maskEmailAddress('worker.alpha@example.com'), 'w***a@example.com');
});

test('maskEmailAddress handles short local parts', () => {
    assertEqual(maskEmailAddress('ab@example.com'), 'a***@example.com');
    assertEqual(maskEmailAddress('x@example.com'), 'x***@example.com');
});

test('maskEmailAddress passes through non-email or empty input safely', () => {
    assertEqual(maskEmailAddress('not-an-email'), 'not-an-email');
    assertEqual(maskEmailAddress(''), '—');
});

test('maskEmailAddress never leaks the full local part', () => {
    const masked = maskEmailAddress('supersecretworker@example.com');
    if (masked.includes('supersecretworker')) {
        throw new Error(`full local part leaked: "${masked}"`);
    }
});

// ---------------------------------------------------------------------------
// formatRelativeHeartbeat
// ---------------------------------------------------------------------------

test('formatRelativeHeartbeat returns Never for missing timestamps', () => {
    assertEqual(formatRelativeHeartbeat(null), 'Never');
    assertEqual(formatRelativeHeartbeat(undefined), 'Never');
    assertEqual(formatRelativeHeartbeat(0), 'Never');
    assertEqual(formatRelativeHeartbeat(-5), 'Never');
});

test('formatRelativeHeartbeat handles seconds timestamps', () => {
    const nowSec = Math.floor(Date.now() / 1000);
    assertEqual(formatRelativeHeartbeat(nowSec - 2), 'Just now');
    assertEqual(formatRelativeHeartbeat(nowSec - 45), '45s ago');
    assertEqual(formatRelativeHeartbeat(nowSec - 5 * 60), '5m ago');
    assertEqual(formatRelativeHeartbeat(nowSec - 3 * 3600), '3h ago');
    assertEqual(formatRelativeHeartbeat(nowSec - 2 * 86400), '2d ago');
});

test('formatRelativeHeartbeat handles millisecond timestamps', () => {
    const nowMs = Date.now();
    assertEqual(formatRelativeHeartbeat(nowMs - 30 * 1000), '30s ago');
    assertEqual(formatRelativeHeartbeat(nowMs - 90 * 60 * 1000), '1h ago');
});

test('formatRelativeHeartbeat clamps future timestamps to Just now', () => {
    const futureSec = Math.floor(Date.now() / 1000) + 3600;
    assertEqual(formatRelativeHeartbeat(futureSec), 'Just now');
});

// ---------------------------------------------------------------------------
// Local-node filtering (mirrors FleetMachinesTable fetchMachines/init)
// ---------------------------------------------------------------------------

function filterRemoteMachines(data: FleetMachineInfo[], localNodeId: string): FleetMachineInfo[] {
    return data.filter((m) => m.node_id !== localNodeId);
}

test('local node is excluded from the remote fleet list', () => {
    const local = makeMachine('node-local-123', 'Local');
    const remote = makeMachine('node-remote-456', 'Worker-1');
    const result = filterRemoteMachines([local, remote], 'node-local-123');
    assertEqual(result.length, 1);
    assertEqual(result[0].node_id, 'node-remote-456');
});

test('empty fleet yields the empty state (no rows)', () => {
    const result = filterRemoteMachines([], 'node-local-123');
    assertEqual(result.length, 0);
});

test('all nodes kept when none matches the local id', () => {
    const a = makeMachine('node-a', 'A');
    const b = makeMachine('node-b', 'B');
    const result = filterRemoteMachines([a, b], 'node-local-123');
    assertEqual(result.length, 2);
});

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

console.log(`\nFleetMachinesTable tests: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} FleetMachinesTable test(s) failed`);
}
