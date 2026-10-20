# Subtask 03: Composite PNG Clipboard Repair & Send Status Correction

**Subtask ID**: `sub-03-clipboard-send`  
**Parent Plan**: `.ai-memory/plans/telegram-backup-and-fleet-management-enhancements.md`  
**Status**: PENDING  

## Objectives
1. Fix composite PNG image copying:
   - Ensure `read_image_as_base64` is used when compositing local images to prevent canvas tainting (`SecurityError`).
   - In `copyAllImagesToClipboard`, execute both native `copy_image_to_clipboard` AND `navigator.clipboard.write` so both Win32 desktop apps and web views receive the image.
2. Send button queue status evaluation:
   - Correct prompt queue status calculation so idle or completed prompts do not falsely indicate `QUEUED` on the Send button.
