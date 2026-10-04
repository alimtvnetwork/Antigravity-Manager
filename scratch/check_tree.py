import sqlite3, json

conn = sqlite3.connect(r'C:\Users\Administrator\.antigravity_tools\repo_prompts.db')
tree_str = conn.execute("SELECT tree_json FROM prompt_tree_cache WHERE cache_key = 'tree:all:50:false'").fetchone()[0]
tree = json.loads(tree_str)
white_nodes = [node for node in tree if 'white' in node.get('repo_name', '').lower()]
for node in white_nodes:
    print("Project:", node.get('project_id'), "repo:", node.get('repo_name'), "inst:", node.get('instance_id'), "running:", node.get('is_running'))
    for c in node.get('conversations', []):
        print("   Conv:", c.get('conversation_id'), "title:", c.get('title'), "running:", c.get('is_running'), "status:", c.get('status'))
