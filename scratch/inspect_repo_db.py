import sqlite3
import os

db_path = r"C:\Users\Administrator\.antigravity_tools\repo_prompts.db"
if not os.path.exists(db_path):
    print(f"Database not found at {db_path}")
    exit(0)

conn = sqlite3.connect(db_path)
cur = conn.cursor()

print("=== TABLES IN DB ===")
for row in cur.execute("SELECT name FROM sqlite_master WHERE type='table'"):
    print(row[0])

print("\n=== TOTAL ROWS IN running_projects ===")
cur.execute("SELECT count(*) FROM running_projects")
print("Total running_projects:", cur.fetchone()[0])

print("\n=== CORRUPTED ROWS (workspace_storage_path IS NULL OR instr(id, '__') == 0) ===")
cur.execute("SELECT count(*) FROM running_projects WHERE workspace_storage_path IS NULL OR instr(id, '__') = 0")
print("Corrupted rows:", cur.fetchone()[0])

print("\n=== WHITE-PRESENTATION ROWS IN running_projects ===")
for r in cur.execute("SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running FROM running_projects WHERE repo_name LIKE '%white-presentation%' OR id LIKE '%white-presentation%'"):
    print(r)

print("\n=== PROMPT_TREE_CACHE ROWS ===")
for r in cur.execute("SELECT cache_key, instance_id, project_count, conversation_count, updated_at FROM prompt_tree_cache"):
    print(r)

print("\n=== ACTIVE_PROMPTS COUNT ===")
cur.execute("SELECT count(*) FROM active_prompts")
print("Total active_prompts:", cur.fetchone()[0])

print("\n=== RUNNING PROMPTS IN active_prompts ===")
for r in cur.execute("SELECT id, project_id, instance_id, status, updated_at FROM active_prompts WHERE status = 'running'"):
    print(r)

conn.close()
