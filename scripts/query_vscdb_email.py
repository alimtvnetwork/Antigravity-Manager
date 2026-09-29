import sys
import sqlite3
import os
import base64
import re

if len(sys.argv) < 2:
    sys.exit(0)

db_path = sys.argv[1]
if not os.path.exists(db_path):
    sys.exit(0)

def extract_email(db_file):
    try:
        conn = sqlite3.connect(db_file)
        c = conn.cursor()
        c.execute("SELECT key, value FROM ItemTable")
        rows = c.fetchall()
        conn.close()
    except Exception:
        return ""

    for k, v in rows:
        try:
            raw = base64.b64decode(v)
        except Exception:
            raw = v if isinstance(v, bytes) else str(v).encode("utf-8")
        
        # 1. Direct regex match
        em = re.findall(rb"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+", raw)
        if em:
            return em[0].decode("utf-8", errors="ignore")
            
        # 2. Shifted base64 regex match (handles protobuf length header prefix)
        for m in re.finditer(rb"[A-Za-z0-9+/=]{16,}", raw):
            cand = m.group()
            for shift in range(4):
                sub = cand[shift:]
                for pad in [b"", b"=", b"==", b"==="]:
                    try:
                        dec = base64.b64decode(sub + pad)
                        em2 = re.findall(rb"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+", dec)
                        if em2:
                            return em2[0].decode("utf-8", errors="ignore")
                    except Exception:
                        pass
    return ""

print(extract_email(db_path))
