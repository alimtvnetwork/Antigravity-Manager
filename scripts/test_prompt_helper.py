import sys
import os
import sqlite3
import time

db_path = os.path.expanduser(r'~/.antigravity_tools/repo_prompts.db')

if len(sys.argv) < 2:
    sys.exit(1)

action = sys.argv[1]

if action == "seed":
    prompt_id = sys.argv[2]
    inst_id = sys.argv[3]
    repo_path = sys.argv[4]
    content = sys.argv[5]
    now = int(time.time())

    conn = sqlite3.connect(db_path)
    c = conn.cursor()
    c.execute("""
        INSERT OR REPLACE INTO active_prompts 
        (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 'running', ?, ?)
    """, (prompt_id, 'proj-test', inst_id, repo_path, content, 'gemini-pro', 'sess-e2e', now, now))
    conn.commit()
    conn.close()
    print("SEEDED")

elif action == "check":
    prompt_id = sys.argv[2]
    conn = sqlite3.connect(db_path)
    c = conn.cursor()
    c.execute("SELECT status, prompt_content FROM active_prompts WHERE id = ?", (prompt_id,))
    row = c.fetchone()
    if row:
        print(f"{row[0]}|{row[1]}")
    else:
        print("NOT_FOUND")
    conn.close()
