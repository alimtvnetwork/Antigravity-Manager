import sqlite3
import os

db_path = r"C:\Users\Administrator\.antigravity_tools\repo_prompts.db"
conn = sqlite3.connect(db_path)
cur = conn.cursor()

# Test 1: Check active_prompts for white-presentation-v1
cur.execute("""
    SELECT count(*) FROM active_prompts 
    WHERE (project_id LIKE '%white-presentation%' OR repo_path LIKE '%white-presentation%')
      AND (instance_id = 'default' OR instance_id = '__default__')
      AND status = 'running'
""")
active_under_default = cur.fetchone()[0]
print(f"Active running prompts under default for white-presentation: {active_under_default}")
assert active_under_default == 0

# Test 2: Check running_projects for any project attached to default with NULL workspace_storage_path
cur.execute("""
    SELECT count(*) FROM running_projects
    WHERE (instance_id = 'default' OR instance_id = '__default__')
      AND workspace_storage_path IS NULL
""")
null_ws_under_default = cur.fetchone()[0]
print(f"Null workspace_storage_path rows under default: {null_ws_under_default}")
assert null_ws_under_default == 0

# Test 3: Check agm_project_sequences and agm_conversation_sequences
cur.execute("SELECT count(*) FROM agm_project_sequences")
p_seq_count = cur.fetchone()[0]
cur.execute("SELECT count(*) FROM agm_conversation_sequences")
c_seq_count = cur.fetchone()[0]
print(f"agm_project_sequences: {p_seq_count}, agm_conversation_sequences: {c_seq_count}")

conn.close()
print("All prompt running logic tests passed successfully!")
