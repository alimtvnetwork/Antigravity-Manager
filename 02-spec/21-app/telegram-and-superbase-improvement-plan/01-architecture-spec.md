# Telegram and Supabase improvement

Slug: `telegram-and-superbase-improvement-plan`

## What this covers

1. Telegram can open Antigravity Manager on the machine that received the command, photograph that window, and send the picture back to the same chat.
2. A prompt is aimed before it is typed. `/target <instance> <conversation>` remembers the destination. The next plain message is the prompt.
3. `/help` shows those commands in Telegram HTML and puts copy buttons under the message.
4. Supabase Test and `agm supabase status` still fail on this machine. The local config is valid. Both project DNS names do not exist. `supabase.co` still resolves. That is a paused free-tier project, not a bad API key.

## Voice notes

Telegram delivers a voice note as an audio file. The Bot API does not transcribe it. This change does not add speech-to-text. After `/target`, type the prompt. A later change could download the audio and run a local transcriber. That is not part of this plan.

## What a screenshot is

`/shot` photographs the Antigravity Manager window on the machine whose bot received the command. If that process is not running, the command starts it, waits, and then takes the picture. It does not click a conversation inside the IDE. The caption names the machine. A prompt sent after `/target` also attaches that window picture when capture succeeds.

Each machine answers for itself when its own manager bot is running. This change does not fan one `/shot` out over SSH.

## Help

Telegram supports a small HTML set (`b`, `code`, `pre`, `blockquote`, links). It does not run CSS. The manual uses those tags, and the message includes `copy_text` buttons for `/ui`, `/shot`, `/target default C001`, and `/prompt C001 --instance default `.
