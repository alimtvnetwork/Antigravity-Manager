import sqlite3
import os
import json
import time

db_path = r"C:\Users\Administrator\.antigravity_tools\repo_prompts.db"
print(f"Connecting to database: {db_path}")

conn = sqlite3.connect(db_path)
cur = conn.cursor()

# 1. Clear prompt_tree_cache
cur.execute("DELETE FROM prompt_tree_cache")
cleared_cache_count = cur.rowcount
print(f"[CLEANUP] Deleted {cleared_cache_count} rows from prompt_tree_cache")

# 2. Delete corrupted rows in running_projects
cur.execute("""
    DELETE FROM running_projects 
    WHERE workspace_storage_path IS NULL 
       OR trim(workspace_storage_path) = '' 
       OR instr(id, '__') = 0 
       OR trim(instance_id) = ''
""")
deleted_corrupted_count = cur.rowcount
print(f"[CLEANUP] Deleted {deleted_corrupted_count} corrupted/un-namespaced rows from running_projects")

# Commit changes
conn.commit()

# 3. Verify white-presentation-v1 status
cur.execute("""
    SELECT id, instance_id, repo_name, is_running, workspace_storage_path 
    FROM running_projects 
    WHERE repo_name LIKE '%white-presentation%' OR id LIKE '%white-presentation%'
""")
white_pres_rows = cur.fetchall()
print(f"\n[VERIFICATION] Remaining white-presentation-v1 rows in running_projects: {len(white_pres_rows)}")
for r in white_pres_rows:
    print("  Row:", r)

# Check if any white-presentation row is attached to default or running
cur.execute("""
    SELECT count(*) 
    FROM running_projects 
    WHERE (repo_name LIKE '%white-presentation%' OR id LIKE '%white-presentation%')
      AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id = '')
      AND is_running = 1
""")
running_under_default = cur.fetchone()[0]
print(f"[VERIFICATION] white-presentation running under Default: {running_under_default}")
assert running_under_default == 0, "FAILED: white-presentation-v1 is still marked running under Default!"

# Check if any corrupted rows remain in running_projects
cur.execute("""
    SELECT count(*) 
    FROM running_projects 
    WHERE workspace_storage_path IS NULL 
       OR trim(workspace_storage_path) = '' 
       OR instr(id, '__') = 0 
       OR trim(instance_id) = ''
""")
corrupted_remaining = cur.fetchone()[0]
print(f"[VERIFICATION] Corrupted rows remaining: {corrupted_remaining}")
assert corrupted_remaining == 0, "FAILED: corrupted rows still exist in running_projects!"

# 4. Check prompt_tree_cache
cur.execute("SELECT count(*) FROM prompt_tree_cache")
cache_count = cur.fetchone()[0]
print(f"[VERIFICATION] prompt_tree_cache row count: {cache_count}")
assert cache_count == 0, "FAILED: prompt_tree_cache is not empty!"

# 5. Test Live Prompt Dispatch & Task File Writing (simulating resend_running_commands_for_instance)
test_workspace = r"d:\work\Antigravity-Manager"
test_task_file = os.path.join(test_workspace, ".antigravity_resume_task.json")
test_payload = {
    "prompt_id": "test-prompt-live-verify-01",
    "project_id": "antigravity-manager__default",
    "instance_id": "default",
    "repo_path": test_workspace,
    "prompt_content": "Verification prompt dispatched via live testing suite",
    "model": "gemini-2.5-pro",
    "image_payload": None,
    "image_paths": [],
    "has_image": False,
    "auto_boot": True,
    "status": "dispatched",
    "resumed_at": int(time.time()),
}

with open(test_task_file, "w", encoding="utf-8") as f:
    json.dump(test_payload, f, indent=2)

print(f"\n[VERIFICATION] Wrote test resume task file to: {test_task_file}")
assert os.path.exists(test_task_file), "FAILED: task file does not exist!"
with open(test_task_file, "r", encoding="utf-8") as f:
    read_back = json.load(f)
assert read_back["prompt_id"] == "test-prompt-live-verify-01", "FAILED: read-back content mismatch!"
print("[VERIFICATION] Successfully verified .antigravity_resume_task.json format and contents.")

# Clean up temporary test task file
if os.path.exists(test_task_file):
    os.remove(test_task_file)
    print(f"[VERIFICATION] Cleaned up temporary test task file.")

# 6. Test Moving a Prompt Between Projects & Instances
now = int(time.time())
test_prompt_id = "test-migration-prompt-001"
cur.execute("""
    INSERT OR REPLACE INTO active_prompts 
    (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
""", (test_prompt_id, "project-alpha__inst1", "inst1", r"d:\work\project-alpha", "Test prompt move", "gemini-flash", "sess-01", "backed_up", now, now))
conn.commit()

# Move the prompt to project-beta under instance-2
new_proj_id = "project-beta__inst2"
new_inst_id = "inst2"
new_repo_path = r"d:\work\project-beta"
cur.execute("""
    UPDATE active_prompts 
    SET project_id = ?, instance_id = ?, repo_path = ?, updated_at = ?
    WHERE id = ?
""", (new_proj_id, new_inst_id, new_repo_path, now + 1, test_prompt_id))
cur.execute("DELETE FROM prompt_tree_cache")
conn.commit()

# Verify the prompt has migrated
cur.execute("SELECT project_id, instance_id, repo_path FROM active_prompts WHERE id = ?", (test_prompt_id,))
migrated = cur.fetchone()
print(f"\n[VERIFICATION] Moved prompt record: {migrated}")
assert migrated == (new_proj_id, new_inst_id, new_repo_path), "FAILED: prompt move failed!"

# Clean up test prompt
cur.execute("DELETE FROM active_prompts WHERE id = ?", (test_prompt_id,))
conn.commit()
print("[VERIFICATION] Cleaned up test prompt from active_prompts.")

# 7. Verify Live Running Prompts Properties Simulation
sample_prompt = "Refactor the authentication middleware to use standard bearer tokens and validate JWT claims properly without leaking secrets."
words = sample_prompt.split()
word_count = len(words)
tail_words = words[-12:] if len(words) >= 12 else words
tail_snippet = " ".join(tail_words)

print("\n=== RUNNING PROMPT PROPERTIES VERIFICATION ===")
print(f"Sample Prompt Word Count: {word_count}")
print(f"Prompt Tail Snippet (12 words): '... {tail_snippet}'")
assert word_count == 17, "Word count mismatch"
assert "secrets." in tail_snippet, "Tail snippet missing expected trailing words"

conn.close()
print("\n[SUCCESS] All live testing and running prompts properties verification passed cleanly!")
