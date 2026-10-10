function formatAuditTime(ts?: number | null): string {
    if (!ts || ts <= 0) return '—';
    const ms = ts < 1e11 ? ts * 1000 : ts;
    const d = new Date(ms);
    if (isNaN(d.getTime())) return String(ts);
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    const day = String(d.getDate()).padStart(2, '0');
    const month = months[d.getMonth()];
    const year = d.getFullYear();
    const hours = String(d.getHours()).padStart(2, '0');
    const mins = String(d.getMinutes()).padStart(2, '0');
    const secs = String(d.getSeconds()).padStart(2, '0');
    return `${day}-${month}-${year} ${hours}:${mins}:${secs}`;
}
