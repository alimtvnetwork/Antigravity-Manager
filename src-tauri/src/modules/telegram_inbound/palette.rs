//! Telegram card palettes.
//!
//! Telegram HTML cannot set hex colors. A `<blockquote>` is painted with the
//! chat accent, which is the red or maroon panel with white text. Palettes
//! from `02-spec/07-design-system/tokens/design-tokens.json` are expressed
//! with a chromatic marker set, and cards stay on the plain chat surface.
//! Riseup is the default: Corporate Gold canvas (`#0f172a`) with emerald
//! `#10b981` and gold `#facc15`.

use super::{load_config, save_config};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardPalette {
    pub id: &'static str,
    pub label: &'static str,
    pub mark: &'static str,
    pub section: &'static str,
    pub bullet: &'static str,
    pub rule: &'static str,
    pub blurb: &'static str,
}

pub const RISEUP: CardPalette = CardPalette {
    id: "riseup",
    label: "Riseup",
    mark: "🌿",
    section: "🟢",
    bullet: "▸",
    rule: "🌿  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·",
    blurb: "Emerald #10b981 and gold #facc15 on slate #0f172a",
};

pub const OBSIDIAN: CardPalette = CardPalette {
    id: "obsidian",
    label: "Obsidian Navy",
    mark: "🌌",
    section: "🔵",
    bullet: "▹",
    rule: "🌌  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·",
    blurb: "Violet #8b5cf6 and sky #38bdf8 on obsidian #0b1329",
};

pub const GOLD: CardPalette = CardPalette {
    id: "gold",
    label: "Corporate Gold",
    mark: "✨",
    section: "🟡",
    bullet: "★",
    rule: "✨  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·",
    blurb: "Gold #facc15 and emerald #10b981 on slate #0f172a",
};

pub const INDIGO: CardPalette = CardPalette {
    id: "indigo",
    label: "Electric Indigo",
    mark: "💠",
    section: "🔷",
    bullet: "◇",
    rule: "💠  ·  ·  ·  ·  ·  ·  ·  ·  ·  ·",
    blurb: "Indigo #6366f1 and cyan #06b6d4 on slate #030712",
};

const PALETTES: &[CardPalette] = &[RISEUP, OBSIDIAN, GOLD, INDIGO];

pub fn resolve_palette(raw: &str) -> Option<CardPalette> {
    let key = raw.trim().trim_start_matches('/').to_ascii_lowercase();
    let key = key.as_str();
    PALETTES.iter().copied().find(|p| {
        p.id == key
            || p.label.eq_ignore_ascii_case(key)
            || (p.id == "riseup" && matches!(key, "corporate" | "riseup-asia" | "default"))
            || (p.id == "obsidian" && matches!(key, "navy" | "dark"))
            || (p.id == "gold" && matches!(key, "corporate-gold" | "corp"))
            || (p.id == "indigo" && matches!(key, "electric" | "cyber"))
    })
}

pub fn active_card_palette() -> CardPalette {
    match load_config() {
        Ok(cfg) => resolve_palette(&cfg.card_palette).unwrap_or(RISEUP),
        Err(e) => {
            crate::error::record_ignored(Err::<(), _>(e), "load telegram card palette");
            RISEUP
        }
    }
}

pub fn kv(palette: CardPalette, label: &str, value: &str) -> String {
    format!(
        "{} <b>{}</b>  <code>{}</code>",
        palette.bullet, label, value
    )
}

pub fn sub_line(palette: CardPalette, text: &str) -> String {
    format!("    {} <i>{}</i>", palette.bullet, text)
}

pub fn section(palette: CardPalette, title: &str, lines: &[String]) -> String {
    let mut out = format!("{} <b>{}</b>\n", palette.section, title);
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub fn prompt_note(palette: CardPalette, title: &str, body: &str) -> String {
    if body.trim().is_empty() {
        return String::new();
    }
    format!(
        "\n{} <b>{}</b>\n<pre>{}</pre>\n",
        palette.section, title, body
    )
}

pub fn theme_footer(palette: CardPalette) -> String {
    format!("<i>{} Theme {}</i>", palette.mark, palette.label)
}

/// Plain-text setter used by `agm telegram theme`. Empty name lists palettes.
pub fn set_card_palette(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("list")
        || trimmed.eq_ignore_ascii_case("ls")
    {
        let current = active_card_palette();
        let mut lines = vec![format!("Current Telegram card theme: {}", current.label)];
        for palette in PALETTES {
            lines.push(format!(
                "  {}  {} — {}",
                palette.mark, palette.id, palette.blurb
            ));
        }
        lines.push("Set one with: agm telegram theme riseup".to_string());
        return Ok(lines.join("\n"));
    }
    let Some(palette) = resolve_palette(trimmed) else {
        return Err(format!(
            "Unknown theme '{}'. Choose riseup, obsidian, gold, or indigo.",
            trimmed
        ));
    };
    let mut cfg = load_config().map_err(|e| format!("telegram config unreadable: {}", e))?;
    cfg.card_palette = palette.id.to_string();
    save_config(&cfg).map_err(|e| format!("telegram config not saved: {}", e))?;
    Ok(format!(
        "Telegram card theme is now {} ({}). {}",
        palette.label, palette.id, palette.blurb
    ))
}

/// HTML reply for `/theme` in the bot.
pub fn format_theme_reply(rest: &str) -> String {
    match set_card_palette(rest) {
        Ok(plain) => {
            let palette = active_card_palette();
            let mut body = String::new();
            for line in plain.lines() {
                body.push_str(&format!("{} {}\n", palette.bullet, line));
            }
            format!(
                "{} <b>Telegram card theme</b>\n{}\n{}{}",
                palette.mark,
                palette.rule,
                body,
                theme_footer(palette)
            )
        }
        Err(err) => format!(
            "⚠️ <b>Theme</b>\n<code>{}</code>\nSend <code>/theme riseup</code>, <code>/theme obsidian</code>, <code>/theme gold</code>, or <code>/theme indigo</code>.",
            crate::modules::notification_hub::escape_telegram_html(&err)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn riseup_is_the_default_name() {
        assert_eq!(resolve_palette("riseup").unwrap().id, "riseup");
        assert_eq!(resolve_palette("default").unwrap().id, "riseup");
        assert_eq!(resolve_palette("navy").unwrap().id, "obsidian");
        assert_eq!(resolve_palette("electric").unwrap().id, "indigo");
        assert!(resolve_palette("red").is_none());
    }

    #[test]
    fn sections_do_not_use_accent_blockquotes() {
        let block = section(RISEUP, "Quota", &[kv(RISEUP, "Selected", "lisa")]);
        assert!(block.contains("🟢"));
        assert!(!block.contains("<blockquote"));
        let note = prompt_note(GOLD, "Prompt", "hello");
        assert!(note.contains("<pre>"));
        assert!(!note.contains("<blockquote"));
    }

    #[test]
    fn missing_palette_field_defaults_to_riseup() {
        let raw = r#"{"bot_token":"","allowed_chat_id":null,"is_enabled":false,"poll_interval_secs":5,"notify_on_system_update":true}"#;
        let cfg: crate::modules::telegram_inbound::TelegramConfig =
            serde_json::from_str(raw).unwrap();
        assert_eq!(cfg.card_palette, "riseup");
    }
}
