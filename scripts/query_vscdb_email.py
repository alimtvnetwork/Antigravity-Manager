import sys
import sqlite3
import json

if len(sys.argv) < 2:
    sys.exit(0)

db_path = sys.argv[1]
try:
    conn = sqlite3.connect(db_path)
    c = conn.cursor()
    c.execute("SELECT value FROM ItemTable WHERE key = 'antigravityAuth.token'")
    r = c.fetchone()
    if r and r[0]:
        data = json.loads(r[0])
        email = data.get('email') or data.get('username') or ''
        print(email)
    conn.close()
except Exception:
    pass
