import { SupabaseSyncApi } from './useSupabaseSync';
import { maskEmail } from '../../utils/maskEmail';

export function LeasesSection(props: SupabaseSyncApi) {
    const { leases, isLoadingLeases, revealedLeaseEmails, setRevealedLeaseEmails, fetchLeases, text } = props;

    return (
            <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-4">
                <div className="flex items-center justify-between">
                    <div>
                        <h4 className="font-semibold text-white flex items-center gap-2">
                            <Layers className="w-4 h-4 text-emerald-400" />
                            Workspace Lease & Cluster Lock Table
                        </h4>
                        <p className="text-xs text-gray-400 mt-0.5">
                            Real-time multi-node account leasing and conflict prevention table synchronized across connected Supabase nodes.
                        </p>
                    </div>
                    <button
                        type="button"
                        onClick={fetchLeases}
                        disabled={isLoadingLeases}
                        className="px-2.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 text-xs flex items-center gap-1.5 transition disabled:opacity-50"
                    >
                        {isLoadingLeases ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        ) : (
                            <RefreshCw className="w-3.5 h-3.5" />
                        )}
                        Refresh Leases
                    </button>
                </div>

                {leases.length === 0 ? (
                    <div className="py-8 text-center text-xs text-gray-500 bg-slate-950/40 rounded-lg border border-dashed border-slate-800">
                        No active workspace leases currently held. Workspaces are acquired on-demand during IDE execution.
                    </div>
                ) : (
                    <div className="overflow-x-auto rounded-lg border border-slate-800 bg-slate-950/40">
                        <table className="w-full text-left text-xs text-gray-300">
                            <thead className="bg-slate-900/80 text-[11px] uppercase tracking-wider text-gray-400 border-b border-slate-800">
                                <tr>
                                    <th className="py-2.5 px-3 font-semibold">Account Email</th>
                                    <th className="py-2.5 px-3 font-semibold">Account ID</th>
                                    <th className="py-2.5 px-3 font-semibold">Node Alias</th>
                                    <th className="py-2.5 px-3 font-semibold">IP Address</th>
                                    <th className="py-2.5 px-3 font-semibold">Instance</th>
                                    <th className="py-2.5 px-3 font-semibold">Expires In</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-slate-800/60 font-mono text-[11px]">
                                {leases.map((lease, idx) => {
                                    const emailKey = `${lease.account_id}_${idx}`;
                                    const isRevealed = !!revealedLeaseEmails[emailKey];
                                    const rawEmail = lease.account_email || 'anonymous';
                                    const displayEmail = isRevealed ? rawEmail : maskEmail(rawEmail);
                                    const nowSec = Math.floor(Date.now() / 1000);
                                    const remainingSec = Math.max(0, lease.expires_at - nowSec);

                                    return (
                                        <tr key={emailKey} className="hover:bg-slate-800/30 transition-colors">
                                            <td className="py-2.5 px-3 font-sans font-medium text-white">
                                                <button
                                                    type="button"
                                                    onClick={() =>
                                                        setRevealedLeaseEmails((prev) => ({
                                                            ...prev,
                                                            [emailKey]: !prev[emailKey],
                                                        }))
                                                    }
                                                    className="hover:underline text-left cursor-pointer"
                                                    title="Click to toggle unmask"
                                                >
                                                    {displayEmail}
                                                </button>
                                            </td>
                                            <td className="py-2.5 px-3 text-gray-400">
                                                {lease.account_id.slice(0, 12)}...
                                            </td>
                                            <td className="py-2.5 px-3">
                                                <span className="px-1.5 py-0.5 rounded bg-blue-500/10 text-blue-400 border border-blue-500/20">
                                                    {lease.node_alias}
                                                </span>
                                            </td>
                                            <td className="py-2.5 px-3 text-gray-400">
                                                {lease.ip_address || '127.0.0.1'}
                                            </td>
                                            <td className="py-2.5 px-3 text-emerald-400">
                                                {lease.profile_name}
                                            </td>
                                            <td className="py-2.5 px-3">
                                                <span
                                                    className={`px-1.5 py-0.5 rounded font-mono ${
                                                        remainingSec > 30
                                                            ? 'bg-emerald-500/10 text-emerald-400'
                                                            : 'bg-amber-500/10 text-amber-400 animate-pulse'
                                                    }`}
                                                >
                                                    {remainingSec}s
                                                </span>
                                            </td>
                                        </tr>
                                    );
                                })}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>

    );
}
