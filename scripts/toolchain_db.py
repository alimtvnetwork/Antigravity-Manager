#!/usr/bin/env python3
"""Toolchain SQLite tracking and verification engine for Antigravity-Manager.

Maintains toolchain metadata in a local SQLite database (toolchain.db),
enabling sub-5ms fast verification and eliminating redundant installer executions.

Usage:
    python scripts/toolchain_db.py init [--db <path>]
    python scripts/toolchain_db.py upsert --name <name> --version <ver> --path <path> --drive <drive> [--verified] [--db <path>]
    python scripts/toolchain_db.py verify --name <name> [--file <filename>] [--db <path>]
    python scripts/toolchain_db.py status [--json] [--db <path>]
    python scripts/toolchain_db.py check-all [--tools rust,msvc,llvm,nasm] [--json] [--db <path>]
"""

import argparse
import datetime
import json
import os
import sqlite3
import sys
from pathlib import Path


def get_default_db_path() -> str:
    """Resolve the default toolchain database path.
    
    Priority:
    1. AGM_TOOLCHAIN_DB environment variable
    2. %LOCALAPPDATA%/agm-toolchain/toolchain.db on Windows
    3. ~/.local/share/agm-toolchain/toolchain.db on Unix
    4. Fallback to .ai-memory/toolchain.db in current working directory
    """
    env_path = os.environ.get("AGM_TOOLCHAIN_DB")
    if env_path:
        return env_path

    if os.name == "nt":
        local_app = os.environ.get("LOCALAPPDATA")
        if local_app:
            target_dir = os.path.join(local_app, "agm-toolchain")
            os.makedirs(target_dir, exist_ok=True)
            return os.path.join(target_dir, "toolchain.db")

    home = os.path.expanduser("~")
    unix_dir = os.path.join(home, ".local", "share", "agm-toolchain")
    if os.path.isdir(os.path.dirname(unix_dir)):
        os.makedirs(unix_dir, exist_ok=True)
        return os.path.join(unix_dir, "toolchain.db")

    # Workspace fallback
    ai_mem_dir = os.path.join(os.getcwd(), ".ai-memory")
    if os.path.exists(ai_mem_dir):
        return os.path.join(ai_mem_dir, "toolchain.db")

    return os.path.join(os.getcwd(), "toolchain.db")


def get_connection(db_path: str) -> sqlite3.Connection:
    """Connect to SQLite database with WAL mode and busy timeout."""
    parent = os.path.dirname(os.path.abspath(db_path))
    if parent and not os.path.exists(parent):
        os.makedirs(parent, exist_ok=True)

    conn = sqlite3.connect(db_path, timeout=5.0)
    conn.execute("PRAGMA journal_mode = WAL;")
    conn.execute("PRAGMA busy_timeout = 5000;")
    conn.execute("PRAGMA synchronous = NORMAL;")
    return conn


def init_db(db_path: str) -> None:
    """Initialize SQLite database schema and views."""
    conn = get_connection(db_path)
    with conn:
        conn.execute("""
            CREATE TABLE IF NOT EXISTS ToolchainInstallation (
                ToolchainInstallationId INTEGER PRIMARY KEY AUTOINCREMENT,
                ToolchainName           TEXT NOT NULL,
                Version                 TEXT NOT NULL,
                InstalledPath           TEXT NOT NULL,
                InstallDrive            TEXT NOT NULL,
                InstalledAt             TEXT NOT NULL,
                IsVerified              INTEGER NOT NULL DEFAULT 0,
                LastVerifiedAt          TEXT NOT NULL,
                CONSTRAINT UQ_ToolchainInstallation_Name_Path UNIQUE (ToolchainName, InstalledPath)
            );
        """)
        conn.execute("""
            CREATE INDEX IF NOT EXISTS IX_ToolchainInstallation_Name
                ON ToolchainInstallation (ToolchainName);
        """)
        conn.execute("""
            CREATE INDEX IF NOT EXISTS IX_ToolchainInstallation_Drive
                ON ToolchainInstallation (InstallDrive);
        """)
        conn.execute("""
            CREATE VIEW IF NOT EXISTS V_ActiveToolchains AS
            SELECT
                ToolchainInstallationId,
                ToolchainName,
                Version,
                InstalledPath,
                InstallDrive,
                InstalledAt,
                IsVerified,
                LastVerifiedAt
            FROM ToolchainInstallation
            ORDER BY InstalledAt ASC, ToolchainInstallationId ASC;
        """)
    conn.close()


def normalize_drive(path_or_drive: str) -> str:
    """Normalize drive letter into format 'C:' or 'D:'."""
    if not path_or_drive:
        return "C:"
    s = str(path_or_drive).strip()
    if len(s) >= 2 and s[1] == ":":
        return s[:2].upper()
    if len(s) == 1 and s.isalpha():
        return f"{s.upper()}:"
    return "C:"


def upsert_toolchain(
    db_path: str,
    name: str,
    version: str,
    path: str,
    drive: str = "",
    is_verified: bool = True,
) -> dict:
    """Insert or update toolchain record."""
    init_db(db_path)
    now_iso = datetime.datetime.now(datetime.timezone.utc).isoformat()
    norm_drive = normalize_drive(drive if drive else path)
    norm_path = os.path.abspath(path)

    conn = get_connection(db_path)
    with conn:
        cursor = conn.cursor()
        cursor.execute(
            """
            INSERT INTO ToolchainInstallation (
                ToolchainName, Version, InstalledPath, InstallDrive, InstalledAt, IsVerified, LastVerifiedAt
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(ToolchainName, InstalledPath) DO UPDATE SET
                Version = excluded.Version,
                InstallDrive = excluded.InstallDrive,
                IsVerified = excluded.IsVerified,
                LastVerifiedAt = excluded.LastVerifiedAt;
            """,
            (
                name.lower().strip(),
                version.strip(),
                norm_path,
                norm_drive,
                now_iso,
                1 if is_verified else 0,
                now_iso,
            ),
        )
    conn.close()

    result = {
        "status": "RECORDED",
        "name": name.lower().strip(),
        "version": version.strip(),
        "path": norm_path,
        "drive": norm_drive,
        "is_verified": is_verified,
        "last_verified_at": now_iso,
    }

    # Best-effort mirror to .ai-memory/toolchain.db if workspace folder exists and not already writing to it
    workspace_mem_db = os.path.join(os.getcwd(), ".ai-memory", "toolchain.db")
    if os.path.exists(os.path.join(os.getcwd(), ".ai-memory")) and os.path.abspath(db_path) != os.path.abspath(workspace_mem_db):
        try:
            init_db(workspace_mem_db)
            m_conn = get_connection(workspace_mem_db)
            with m_conn:
                m_conn.execute(
                    """
                    INSERT INTO ToolchainInstallation (
                        ToolchainName, Version, InstalledPath, InstallDrive, InstalledAt, IsVerified, LastVerifiedAt
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(ToolchainName, InstalledPath) DO UPDATE SET
                        Version = excluded.Version,
                        InstallDrive = excluded.InstallDrive,
                        IsVerified = excluded.IsVerified,
                        LastVerifiedAt = excluded.LastVerifiedAt;
                    """,
                    (
                        name.lower().strip(),
                        version.strip(),
                        norm_path,
                        norm_drive,
                        now_iso,
                        1 if is_verified else 0,
                        now_iso,
                    ),
                )
            m_conn.close()
        except Exception:
            pass

    return result


def verify_toolchain(db_path: str, name: str, expected_file: str = "") -> dict:
    """Verify toolchain state in database and validate disk presence."""
    if not os.path.exists(db_path):
        return {"status": "NOT_RECORDED", "name": name, "is_verified": False}

    init_db(db_path)
    conn = get_connection(db_path)
    cursor = conn.cursor()
    cursor.execute(
        """
        SELECT ToolchainInstallationId, ToolchainName, Version, InstalledPath, InstallDrive, IsVerified, LastVerifiedAt
        FROM ToolchainInstallation
        WHERE ToolchainName = ? AND IsVerified = 1
        ORDER BY ToolchainInstallationId DESC
        LIMIT 1;
        """,
        (name.lower().strip(),),
    )
    row = cursor.fetchone()
    conn.close()

    if not row:
        return {"status": "UNVERIFIED", "name": name, "is_verified": False}

    inst_id, tool_name, version, installed_path, install_drive, is_verified, last_verified = row

    # Check path existence on disk
    path_exists = os.path.exists(installed_path)
    if expected_file and path_exists:
        if os.path.isdir(installed_path):
            probe_path = os.path.join(installed_path, expected_file)
            path_exists = os.path.exists(probe_path)
        else:
            path_exists = os.path.basename(installed_path).lower() == expected_file.lower()

    if not path_exists:
        return {
            "status": "MISSING_ON_DISK",
            "name": tool_name,
            "version": version,
            "path": installed_path,
            "drive": install_drive,
            "is_verified": False,
        }

    return {
        "status": "VERIFIED",
        "name": tool_name,
        "version": version,
        "path": installed_path,
        "drive": install_drive,
        "is_verified": True,
        "last_verified_at": last_verified,
    }


def list_status(db_path: str) -> list[dict]:
    """Retrieve all active toolchain records ordered by FIFO sequence."""
    if not os.path.exists(db_path):
        return []

    init_db(db_path)
    conn = get_connection(db_path)
    cursor = conn.cursor()
    cursor.execute(
        """
        SELECT ToolchainInstallationId, ToolchainName, Version, InstalledPath, InstallDrive, InstalledAt, IsVerified, LastVerifiedAt
        FROM V_ActiveToolchains;
        """
    )
    rows = cursor.fetchall()
    conn.close()

    records = []
    for r in rows:
        records.append({
            "id": r[0],
            "name": r[1],
            "version": r[2],
            "path": r[3],
            "drive": r[4],
            "installed_at": r[5],
            "is_verified": bool(r[6]),
            "last_verified_at": r[7],
        })
    return records


def check_all(db_path: str, required_tools: list[str]) -> tuple[bool, dict]:
    """Check whether all specified tools are verified and present on disk."""
    results = {}
    all_ok = True

    tool_files = {
        "llvm": "libclang.dll",
        "rust": "rustc.exe" if os.name == "nt" else "rustc",
        "msvc": "link.exe" if os.name == "nt" else "",
        "nasm": "nasm.exe" if os.name == "nt" else "nasm",
        "nodejs": "node.exe" if os.name == "nt" else "node",
        "pnpm": "pnpm.cmd" if os.name == "nt" else "pnpm",
    }

    for tool in required_tools:
        tool_name = tool.lower().strip()
        expected = tool_files.get(tool_name, "")
        res = verify_toolchain(db_path, tool_name, expected_file=expected)
        results[tool_name] = res
        if not res.get("is_verified"):
            all_ok = False

    return all_ok, results


def main() -> int:
    parser = argparse.ArgumentParser(description="Toolchain SQLite Tracker & Fast Verifier")
    parser.add_argument("--db", default=None, help="Path to toolchain.db (defaults to standard location)")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # Subcommand: init
    subparsers.add_parser("init", help="Initialize toolchain.db schema")

    # Subcommand: upsert
    p_upsert = subparsers.add_parser("upsert", help="Insert or update toolchain record")
    p_upsert.add_argument("--name", required=True, help="Toolchain name (e.g. llvm, rust, msvc, nasm)")
    p_upsert.add_argument("--version", required=True, help="Installed version string")
    p_upsert.add_argument("--path", required=True, help="Installed path on disk")
    p_upsert.add_argument("--drive", default="", help="Drive letter (e.g. C:, D:)")
    p_upsert.add_argument("--verified", action="store_true", default=True, help="Mark as verified (default: true)")
    p_upsert.add_argument("--unverified", action="store_true", help="Mark as unverified")

    # Subcommand: verify
    p_verify = subparsers.add_parser("verify", help="Verify single toolchain component")
    p_verify.add_argument("--name", required=True, help="Toolchain name")
    p_verify.add_argument("--file", default="", help="Optional executable/dll filename to test inside path")

    # Subcommand: status
    p_status = subparsers.add_parser("status", help="List all tracked toolchains")
    p_status.add_argument("--json", action="store_true", help="Output results in JSON format")

    # Subcommand: check-all
    p_check = subparsers.add_parser("check-all", help="Verify all essential toolchain components")
    p_check.add_argument("--tools", default="rust,msvc,llvm,nasm", help="Comma-separated toolchain names")
    p_check.add_argument("--json", action="store_true", help="Output results in JSON format")

    args = parser.parse_args()
    db_path = args.db if args.db else get_default_db_path()

    if args.command == "init":
        init_db(db_path)
        print(f"Toolchain DB initialized at: {db_path}")
        return 0

    elif args.command == "upsert":
        is_ver = not args.unverified if args.unverified else args.verified
        res = upsert_toolchain(
            db_path=db_path,
            name=args.name,
            version=args.version,
            path=args.path,
            drive=args.drive,
            is_verified=is_ver,
        )
        print(json.dumps(res, indent=2))
        return 0

    elif args.command == "verify":
        res = verify_toolchain(db_path, args.name, expected_file=args.file)
        print(json.dumps(res, indent=2))
        return 0 if res.get("is_verified") else 1

    elif args.command == "status":
        records = list_status(db_path)
        if args.json:
            print(json.dumps(records, indent=2))
        else:
            if not records:
                print(f"No toolchains recorded in {db_path}")
            else:
                print(f"Toolchain Registry ({db_path}):")
                print(f"{'Name':<12} {'Version':<14} {'Drive':<6} {'Verified':<10} {'Path'}")
                print("-" * 75)
                for r in records:
                    ver_str = "YES" if r["is_verified"] else "NO"
                    print(f"{r['name']:<12} {r['version']:<14} {r['drive']:<6} {ver_str:<10} {r['path']}")
        return 0

    elif args.command == "check-all":
        tools = [t.strip() for t in args.tools.split(",") if t.strip()]
        all_ok, results = check_all(db_path, tools)
        if args.json:
            print(json.dumps({"all_verified": all_ok, "tools": results}, indent=2))
        else:
            for tool, info in results.items():
                ver_str = "[OK]" if info.get("is_verified") else "[FAIL]"
                status = info.get("status", "UNKNOWN")
                path = info.get("path", "N/A")
                print(f"  {ver_str} {tool:<10} : {status} ({path})")
            if all_ok:
                print("\nAll required toolchain components are VERIFIED and present on disk.")
            else:
                print("\nOne or more toolchain components are missing or unverified.")
        return 0 if all_ok else 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
