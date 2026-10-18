# UI and Functionality Enhancement Specification

## Overview & Executive Summary

This specification establishes the architectural, structural, and behavioral standards for the comprehensive UI and backend enhancements across Antigravity-Manager:
1. **Per-Instance Prompt State Isolation:** Guaranteeing independent running prompt status per instance and eliminating cross-instance bleed in multi-instance setups.
2. **SSH Cluster & Node W2 Parity with GitMap:** Bringing AGM's SSH node discovery and cluster dispatch to parity with GitMap's Go-based SSH engine.
3. **Prompt Tree View & Context System:** Elevating modal z-indices above `z-[100000]`, providing native right-click menus on project and conversation nodes, differentiated conversation views, and enqueued prompt counter badges.
4. **Markdown View & Multimodal Clipboard:** Compact I/O toolbar controls, reduced button border-radii (`rounded-md`), restored instance Detail modal, and dual clipboard copy options ("Copy includes image" vs "Copy without image").
5. **Machine-Independent Standalone JSON:** Offering two export formats: standalone bundle with embedded base64 assets vs lightweight relative path JSON.
6. **Updater Version Detection & Pinned Release Execution:** Dynamic live version extraction from executable PE headers and unblocking pinned tag URLs in `install.ps1` and `update_cmds.rs`.
7. **Coding Guideline Modernization:** Standardizing positive boolean prefixes, eliminating double negatives, and encapsulating magic literals.
