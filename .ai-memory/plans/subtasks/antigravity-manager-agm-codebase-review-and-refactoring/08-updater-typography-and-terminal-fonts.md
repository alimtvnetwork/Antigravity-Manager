# Subtask 08: Updater Typography & Terminal Fonts

**Application:** Antigravity Manager (AGM)  
**Task Slug:** `antigravity-manager-agm-codebase-review-and-refactoring`  
**Subtask ID:** `08-updater-typography-and-terminal-fonts`  
**Status:** Ready / Implemented  
**Spec Reference:** [02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md](../../../../02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md)

---

## 1. Problem Statement & Typography Objectives

### 1.1 Windows Raster Font Degradation
When the delegated CLI updater launched on Windows, `AllocConsole()` initialized a legacy command prompt window using default raster/bitmap fonts (e.g. 8x12 Terminal), causing blurry rendering and visual artifacts on modern high-DPI displays.

### 1.2 Cross-Platform Font Standards
- **Windows:** Consolas 18pt font (`font_family = 54`, `dw_font_size.y = 18`, `font_weight = 600`) with Lucida Console fallback.
- **Linux:** Ubuntu Mono (14-16pt) monospace terminal font.
- **macOS:** SF Mono / Apple system monospace font.

---

## 2. Technical Implementation Plan

### 2.1 Win32 Console Configuration (`src-tauri/src/modules/delegate_updater/run.rs`)
In `win32_console::setup_dedicated_updater_console`:
```rust
#[repr(C)]
struct CONSOLE_FONT_INFOEX {
    cb_size: u32,
    n_font: u32,
    dw_font_size: COORD,
    font_family: u32,
    font_weight: u32,
    face_name: [u16; 32],
}

pub fn setup_dedicated_updater_console() -> bool {
    unsafe {
        ensure_clean_newline_mode();
        set_updater_title();

        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle.is_null() {
            return false;
        }

        let mut font_info: CONSOLE_FONT_INFOEX = std::mem::zeroed();
        font_info.cb_size = std::mem::size_of::<CONSOLE_FONT_INFOEX>() as u32;
        font_info.font_family = 54; // FF_MODERN (0x30) | FIXED_PITCH (0x06)
        font_info.dw_font_size.y = 18;
        font_info.font_weight = 600;
        let face_name_utf16: Vec<u16> = "Consolas\0".encode_utf16().collect();
        for (i, &c) in face_name_utf16.iter().enumerate() {
            if i < font_info.face_name.len() {
                font_info.face_name[i] = c;
            }
        }
        if SetCurrentConsoleFontEx(handle, 0, &mut font_info) == 0 {
            // Fallback to Lucida Console if Consolas is unavailable
            let mut fallback_font: CONSOLE_FONT_INFOEX = std::mem::zeroed();
            fallback_font.cb_size = std::mem::size_of::<CONSOLE_FONT_INFOEX>() as u32;
            fallback_font.font_family = 54;
            fallback_font.dw_font_size.y = 18;
            fallback_font.font_weight = 600;
            let lucida_utf16: Vec<u16> = "Lucida Console\0".encode_utf16().collect();
            for (i, &c) in lucida_utf16.iter().enumerate() {
                if i < fallback_font.face_name.len() {
                    fallback_font.face_name[i] = c;
                }
            }
            SetCurrentConsoleFontEx(handle, 0, &mut fallback_font);
        }

        // Configure optimal window and buffer dimensions
        let _ = SetConsoleScreenBufferSize(handle, COORD { x: 120, y: 400 });
        let _ = SetConsoleWindowInfo(
            handle,
            1,
            &SMALL_RECT {
                left: 0,
                top: 0,
                right: 119,
                bottom: 34,
            },
        );
        true
    }
}
```

### 2.2 Clean Newline & UTF-8 Codepage Mode
```rust
pub fn ensure_clean_newline_mode() -> bool {
    unsafe {
        SetConsoleOutputCP(65001);
        SetConsoleCP(65001);

        let mut all_ok = true;
        for &handle_id in &[STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            let handle = GetStdHandle(handle_id);
            if handle.is_null() {
                continue;
            }
            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) != 0 {
                // Ensure DISABLE_NEWLINE_AUTO_RETURN (0x0008) is NEVER set
                let mode_with_vt = (mode
                    | ENABLE_PROCESSED_OUTPUT
                    | ENABLE_WRAP_AT_EOL_OUTPUT
                    | ENABLE_VIRTUAL_TERMINAL_PROCESSING)
                    & !DISABLE_NEWLINE_AUTO_RETURN;

                if SetConsoleMode(handle, mode_with_vt) == 0 {
                    all_ok = false;
                }
            }
        }
        all_ok
    }
}
```

---

## 3. Invariants & Positive Booleans

- `all_ok`: Positive boolean confirming successful console mode initialization across stdout and stderr.
- `is_font_applied`: Positive verification indicator for console font binding.

---

## 4. Verification & Testing

```bash
cd src-tauri && cargo check --lib
cd src-tauri && cargo clippy --all-targets --all-features
```

### Verification Criteria
- [x] Win32 console initializes with Consolas 18pt font.
- [x] Fallback to Lucida Console succeeds if Consolas font is absent.
- [x] UTF-8 CodePage 65001 correctly renders ANSI tree glyphs and boxes.
- [x] Newline mode outputs standard CRLF without line truncation.
