---
name: agm-multimodal-audio-video
description: Specialized skill for managing the Antigravity-Manager multimodal audio and video streaming pipeline, Whisper-compatible transcription endpoints, MIME normalization, payload conversions, and inline media size constraints in src-tauri/src/proxy/audio/ and video/.
---

# AGM Multimodal Audio & Video Pipeline

This skill provides comprehensive architectural guidance, data transformation procedures, MIME normalization standards, and payload constraint enforcement for the multimodal Audio and Video subsystems in **Antigravity-Manager (AGM)**.

---

## 1. Subsystem Architecture Overview

AGM enables seamless multimedia handling across disparate client protocols (OpenAI, Claude, Gemini) by normalizing multimodal payloads into Google Gemini Canonical IR:

```
+----------------------------------------------------------------------------------------------------+
|                                    Inbound Multimodal Requests                                     |
|    - Audio Transcription: POST /v1/audio/transcriptions (OpenAI Whisper API compatible)            |
|    - Chat Audio Parts: input_audio { data, format } or data:audio/wav;base64,...                   |
|    - Chat Video Parts: data:video/mp4;base64,... or http(s)://... or local file:///...             |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                 Processors & Size Validation Gates                                 |
|    - AudioProcessor (src-tauri/src/proxy/audio/mod.rs): 15MB hard limit, MIME inference           |
|    - VideoProcessor (src-tauri/src/proxy/video/mod.rs): 20MB hard limit, 8 container formats       |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                   Normalization to Gemini Canonical                                |
|    - Audio Part -> { inlineData: { mimeType: "audio/...", data: "<base64>" } }                     |
|    - Video Part -> { inlineData: { mimeType: "video/...", data: "<base64>" } }                     |
|    - URL / Cloud Storage Reference -> { fileData: { mimeType: "...", fileUri: "<url>" } }          |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                   Upstream Gemini Execution                                        |
|    - Transcriptions: Formulated as Gemini prompt ("Generate a transcript of the speech.")          |
|    - Response: JSON envelope {"text": "..."} compliant with OpenAI Whisper client expectations     |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/proxy/audio/mod.rs` | `AudioProcessor` struct, MIME detection (`mp3`, `wav`, `m4a`/`aac`, `ogg`/`opus`, `flac`, `aiff`), 15MB limit check (`MAX_SIZE = 15 * 1024 * 1024`), base64 encoding, and `audio_part_from_source()` mapper. |
| `src-tauri/src/proxy/video/mod.rs` | `VideoProcessor` struct, container detection (`mp4`, `webm`, `mov`, `avi`, `wmv`, `flv`, `mkv`, `3gp`), 20MB limit check (`MAX_SIZE = 20 * 1024 * 1024`), and `normalize_video_mime()`. |
| `src-tauri/src/proxy/handlers/audio.rs` | Axum handler for `POST /v1/audio/transcriptions`. Parses `multipart/form-data`, validates audio bytes, builds Gemini transcription prompt, calls upstream client, and returns OpenAI-compatible JSON `{ "text": "..." }`. |
| `src-tauri/src/proxy/mappers/openai/` | Inbound mapper extracting `input_audio` objects and data URLs from OpenAI Chat Completion messages and passing them to `audio_part_from_source()`. |

---

## 3. Core Invariants & Rules

1. **Strict Inline Payload Ceilings**:
   - Audio files must NOT exceed **15MB** (`AudioProcessor::exceeds_size_limit`).
   - Video files must NOT exceed **20MB** (`VideoProcessor::exceeds_size_limit`).
   - Requests exceeding these limits must return HTTP `400 BAD_REQUEST` with clear error messages before contacting upstream endpoints, preventing token starvation or upstream 413 errors.

2. **Source Resolution Hierarchy**:
   `audio_part_from_source(src, declared_mime)` and video handlers must support 4 distinct input formats:
   - `data:audio/<mime>;base64,...` -> decoded into `inlineData`.
   - `http://` or `https://` -> mapped to `fileData { fileUri }`.
   - `file:///path` or local filesystem path -> read asynchronously from disk and base64 encoded into `inlineData`.
   - Raw Base64 string -> wrapped in `inlineData` using `declared_mime` or default `audio/mp3`.

3. **MIME Normalization**:
   - Aliases must map to standard IANA media types (e.g. `wave`, `x-wav` -> `audio/wav`; `opus`, `oga` -> `audio/ogg`; `m4a` -> `audio/aac`).
   - Video container extensions (`m4v` -> `video/mp4`; `quicktime` -> `video/quicktime`).

4. **Whisper API Wire Compatibility**:
   - The `/v1/audio/transcriptions` endpoint must faithfully emulate OpenAI Whisper responses, returning `{ "text": "<transcribed content>" }`.
   - If the upstream Gemini model outputs structured thoughts, thoughts must be stripped, leaving only the pure transcription text.

---

## 4. Verification & Testing

When making changes to audio or video handlers:

```bash
# 1. Check Rust formatting and clippy
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features

# 2. Test Audio Transcription endpoint via curl
curl -X POST http://127.0.0.1:8045/v1/audio/transcriptions \
  -H "Authorization: Bearer test" \
  -F "file=@test_audio.wav" \
  -F "model=gemini-2.0-flash-exp"
```
