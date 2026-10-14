# Telegram Voice → Prompt Dispatch: Feasibility Analysis

## Status
Analysis only. No implementation, no dependencies added. Owned by Task-09.

## Purpose
Determine whether the Telegram bot in this project can accept a **voice
message** from the user, turn it into text, and feed that text into the
planned `/send` flow so the transcribed prompt is dispatched to the IDE —
exactly as if the user had typed it.

## Background: how the telegram_inbound module works today
All paths below are relative to the repo root and all file names are
lowercase, matching the source tree.

- The bot ingress lives in `src-tauri/src/modules/telegram_inbound/`.
- `daemon.rs` runs a polling loop (`poll_telegram_updates_once`) against
  the Telegram Bot API `getUpdates` endpoint. For each incoming message it
  reads `message.text` (falling back to `message.caption`) and hands it to
  the command processor in `processing.rs`.
- `media.rs` already implements the full "download a file from Telegram"
  pattern: call `getFile` with the message's `file_id`, build the
  `https://api.telegram.org/file/bot<token>/<file_path>` URL, download the
  bytes, and index them in a local SQLite `media_vault` table
  (`download_and_store_telegram_photo`). A voice-message variant of this
  helper would reuse the same proven path.
- Voice is **not handled today**: voice messages arrive as
  `message.voice.file_id` (Opus-in-Ogg `.oga`, typically 8–64 kbps), which
  the current daemon ignores — `ui_remote.rs` explicitly tells the user
  "Voice notes stay audio. Telegram does not turn them into text. Type the
  prompt after `/target`." This analysis proposes changing exactly that.
- The `/send` flow (Task-07, pending) is a two-step design: destination
  first (user issues `/send`, picks machine → instance → conversation from
  a picker, bot confirms the destination), then prompt text, then dispatch
  through the existing prompt-dispatch path, then delivery confirmation
  with a screenshot.

## Proposed flow (any STT option)
1. User records a voice message and sends it to the bot.
2. The daemon detects `message.voice` in the update.
3. The bot acknowledges ("🎙 Transcribing your voice message…") so the
   user is never left staring at silence.
4. The voice file is downloaded (Bot API file limit is 20 MB for
   `getFile`-retrieved files — a minute of spoken prompt at Telegram's
   default bitrate is well under 1 MB, so the limit is not a practical
   constraint).
5. Speech-to-text converts the audio to text.
6. The transcript is fed into step 3 of the `/send` flow — **after** the
   destination has already been chosen — and dispatched to the IDE like any
   typed prompt.
7. The bot replies with the transcript plus delivery confirmation (and the
   screenshot proof required by Task-07).

## Option A: Telegram's native voice transcription
Telegram's official clients offer one-tap transcription of voice messages,
but that feature is a **client-side, account-bound Premium perk**:
the Bot API does not expose it. An incoming `Update` object contains no
transcript field for `message.voice`, and bots receive the audio only. The
only "integration" possible is manual: the user taps transcribe in the
Telegram app (Premium account), copies the resulting text, and pastes it
after `/send` — i.e. no automation at all.

- Latency: N/A (manual).
- Accuracy: decent on Telegram's side, but irrelevant — the bot never sees it.
- Cost: user pays Telegram Premium themselves.
- Privacy: fine (never leaves Telegram).
- Platform support: not available to bots on any platform.
- Implementation complexity: zero, but also zero automation — it is not a
  feature, it is a workaround.

**Verdict: option A is not automatable.** It is worth exactly one line in
the help text ("Premium users can transcribe in the app and paste"), and
nothing more.

## Option B1: bot downloads the file and runs local Whisper
The bot downloads the `.oga` via the existing `getFile` pattern, then
runs transcription locally with an open-weight Whisper-family model
(whisper.cpp-style binary or a Python runner behind a small sidecar
process). No audio ever leaves the user's machines.

- Latency: best possible steady-state — no network upload; on a modern
  laptop, a `base`/`small` model transcribes a 60-second note in roughly
  real time or faster. Cold-start cost is one model load (~1 s).
- Accuracy: Whisper `small`/`medium` is excellent for English and strong
  for the other languages this project deals with; slightly behind the
  best cloud engines on heavy accents and jargon, but the gap is small at
  `small` and above.
- Cost: zero per-message after the one-time model download (~100 MB–1 GB
  depending on model size, cached once in the app data dir).
- Privacy: strongest option — voice never leaves the machine, consistent
  with this project's on-premises, owner-controlled posture.
- Platform support: whisper.cpp-class binaries build for Windows, macOS,
  and Linux (the platforms AGM ships to), but **shipping one binary per
  platform** is real release-pipeline work (signing, bundling, AV
  heuristics on Windows). CPU-only inference needs no GPU. ARM vs x86
  doubles the matrix if both are shipped.
- Implementation complexity: medium-high. Needs: a voice-aware
  `download_and_store_telegram_voice` helper (clone of the photo path),
  audio conversion (Whisper's native decoder expects 16 kHz WAV; Telegram
  ships Opus-in-Ogg, so either bundle a decoder or shell out to an ffmpeg
  binary — another bundling concern), process spawn + timeout + error
  routing into the project's error module (`AppError`, no silent errors —
  transcription failure must surface as a visible bot error, never a
  swallowed empty transcript), model download + cache validation, and a
  config toggle plus model-size selector in settings.

## Option B2: bot downloads the file and calls a cloud STT API
The bot downloads the voice file and POSTs it to a speech-to-text HTTP
endpoint (OpenAI Whisper API, Google Speech-to-Text, Deepgram, etc.), then
continues as in B1.

- Latency: typically 2–8 s end-to-end for a short note (upload + inference
  + response), dominated by network. Acceptable for an async bot
  interaction, worse than local on a slow uplink.
- Accuracy: state of the art, especially with cloud models that handle
  code-switching and domain jargon well; generally a small notch above
  local Whisper `base`/`small`.
- Cost: small but real and metered — e.g. roughly $0.006 per audio minute
  on OpenAI's Whisper API. A heavy user (dozens of voice prompts daily)
  stays well under a few dollars a month; a fleet of staff members
  multiplies it.
- Privacy: the weakest option — spoken prompts (which can contain
  proprietary code, credentials read aloud, internal plans) are sent to a
  third-party service. Requires the user's explicit consent and should be
  off by default.
- Platform support: trivial — it is one HTTPS call with the existing
  HTTP client, identical on every OS, zero bundling.
- Implementation complexity: low. Needs: the voice download helper, one
  HTTP POST function, API-key storage in the existing config (encrypted at
  rest if the project's config layer supports it), provider URL in
  settings, and the same error routing + timeouts as B1.

## Trade-off summary
| Dimension            | A: native (manual)     | B1: local Whisper            | B2: cloud STT API            |
|----------------------|------------------------|------------------------------|------------------------------|
| Automation           | None                   | Full                         | Full                         |
| Latency (60 s note)  | Manual                 | ~1–3 s steady-state          | ~2–8 s (network-bound)       |
| Accuracy             | N/A to the bot         | Very good (small/medium)     | Best (SOTA cloud models)     |
| Cost                 | Premium subscription   | $0 after model download      | Metered, ~$0.006/min         |
| Privacy              | Never leaves Telegram  | Audio never leaves machine   | Audio leaves to a 3rd party  |
| Platform support     | Bot-inaccessible       | Needs per-OS binary bundling | Identical on all platforms   |
| Implementation cost  | Zero / not a feature   | Medium-high (bundling work)  | Low (one HTTP call + config) |

## Recommended path
**B2 (cloud STT API) first, B1 (local Whisper) as the follow-up.**

Reasoning:
1. B2 is the only option that ships soon: it reuses the existing,
   already-proven `getFile` download pattern in `media.rs` and adds one
   HTTP call — days of work, no release-pipeline changes, no per-OS
   binary bundling.
2. B1 is the right long-term default (zero cost, strongest privacy, no
   network dependency), but its cost is concentrated in release
   engineering (per-platform model binaries + audio decoder bundling +
   signing), not in the bot logic. That work is real and should not block
   the feature.
3. Both options converge on the same bot-side interface: "voice bytes in,
   transcript string out." Building B2 first with a clean
   `transcribe_voice(audio_path) -> Result<String>` boundary means B1
   later swaps the engine behind that boundary with no changes to the
   daemon, the `/send` integration, or the UX.
4. Privacy stance: cloud STT must be **opt-in with an explicit consent
   toggle** and off by default, since spoken prompts can contain sensitive
   material. The settings UI should name the provider and link its data
   policy.
5. A transcript confirmation step is strongly recommended before dispatch:
   the bot replies "I heard: `<transcript>` — replying `yes` (or tapping
   ✅) sends it to `<destination>`." STT errors are common with jargon;
   dispatching a misheard prompt to the IDE is the failure mode this
   project must not have. This mirrors the explicit-destination
   confirmation culture of the `/send` design.

## What stays unchanged
- The `/send` **destination-first ordering** applies to transcribed text
  exactly as it applies to typed text. A voice message never bypasses the
  destination picker: the user must have a confirmed destination (from an
  earlier `/send` in the same session, or the voice flow must first ask
  for one) before the transcript is dispatched. Never guess the
  destination.
- The prompt-dispatch path, the delivery confirmation, and the screenshot
  proof are untouched — transcription is a text-source adapter that feeds
  the existing pipeline.
- The no-silent-errors rule: a failed download or a failed transcription
  must produce a visible bot error ("⚠️ I couldn't transcribe that voice
  message: <reason>. Please type it or try again."), never a silent drop.

## Suggested implementation touchpoints (no code written)
- `src-tauri/src/modules/telegram_inbound/media.rs`: add
  `download_and_store_telegram_voice` (voice-aware clone of the photo
  downloader; key off `msg["voice"]["file_id"]`); store duration in
  `media_vault` metadata.
- `src-tauri/src/modules/telegram_inbound/daemon.rs`: branch on
  `msg["voice"]` presence in `poll_telegram_updates_once`; run the
  transcribe step; route the transcript into the same handling as typed
  prompt text for the pending `/send` destination.
- New module (e.g. `src-tauri/src/modules/telegram_inbound/voice.rs` —
  name to be decided at implementation time): the
  `transcribe_voice` boundary, provider config, timeouts, and error
  mapping into `AppError`.
- Settings/UI: STT provider selector, API-key field, consent toggle,
  optional local-model size selector (for the later B1 phase).

## Open questions for the user
1. **Preferred STT provider for the B2 phase** — OpenAI Whisper API,
   Google Speech-to-Text, Deepgram, or another? (Determines auth shape
   and pricing.)
2. **Language** — which spoken language(s) should be the default, and
   should the bot auto-detect or use a fixed language setting?
3. **Transcript confirmation** — must the bot always show the transcript
   and wait for a yes before dispatching, or is an immediate dispatch
   acceptable for trusted chats?
4. **Voice file retention** — delete the downloaded `.oga` after
   transcription, or keep it in `media_vault` like photos (privacy vs
   auditability)?
5. **Length limits** — maximum voice-message duration to accept (e.g.
   refuse notes over 2–5 minutes, or chunk longer ones)?
6. **B1 timing** — is local Whisper a hard requirement before shipping,
   or is opt-in cloud STT acceptable as the first release with local as
   the follow-up?

(Thanks to @aukgit)
