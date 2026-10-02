#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    AddAccount = 1,
    UpdateAccount = 2,
    SwitchAccount = 3,
}

impl AuditAction {
    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn pascal(self) -> &'static str {
        match self {
            Self::AddAccount => "AddAccount",
            Self::UpdateAccount => "UpdateAccount",
            Self::SwitchAccount => "SwitchAccount",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::AddAccount => "Add Account",
            Self::UpdateAccount => "Update Account",
            Self::SwitchAccount => "Switch Account",
        }
    }

    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(Self::AddAccount),
            2 => Some(Self::UpdateAccount),
            3 => Some(Self::SwitchAccount),
            _ => None,
        }
    }

    pub fn from_legacy(raw: &str) -> Option<Self> {
        let folded = raw.trim().to_ascii_lowercase().replace([' ', '_', '-'], "");
        match folded.as_str() {
            "addaccount" | "1" => Some(Self::AddAccount),
            "updateaccount" | "2" => Some(Self::UpdateAccount),
            "switchaccount" | "3" => Some(Self::SwitchAccount),
            _ => None,
        }
    }
}

pub fn resolve_action(code: Option<i32>, legacy: &str) -> (i32, String, String) {
    if let Some(code) = code {
        if let Some(action) = AuditAction::from_code(code) {
            return (
                action.code(),
                action.pascal().to_string(),
                action.title().to_string(),
            );
        }
    }
    if let Some(action) = AuditAction::from_legacy(legacy) {
        return (
            action.code(),
            action.pascal().to_string(),
            action.title().to_string(),
        );
    }
    (0, legacy.to_string(), legacy.replace('_', " "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_add_and_switch_map_to_numeric_codes() {
        assert_eq!(
            AuditAction::from_legacy("add_account"),
            Some(AuditAction::AddAccount)
        );
        assert_eq!(
            AuditAction::from_legacy("switch_account"),
            Some(AuditAction::SwitchAccount)
        );
        let (code, pascal, title) = resolve_action(None, "add_account");
        assert_eq!(code, 1);
        assert_eq!(pascal, "AddAccount");
        assert_eq!(title, "Add Account");
    }
}
