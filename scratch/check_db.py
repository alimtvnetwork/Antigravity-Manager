import sqlite3

conn = sqlite3.connect(r'C:\Users\Administrator\.antigravity_tools\repo_prompts.db')
tables = [r[0] for r in conn.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()]
for t in tables:
    try:
        rows = conn.execute(f"SELECT * FROM {t}").fetchall()
        for r in rows:
            if any('white' in str(col).lower() for col in r):
                print(f"Table {t}:", r)
    except Exception as e:
        print(f"Error {t}:", e)
