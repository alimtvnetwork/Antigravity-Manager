#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    Unknown = 0,
    AddAccount = 1,
    UpdateAccount = 2,
    SwitchAccount = 3,
    SchedulePrompt = 4,
    RequeueConversation = 5,
    LaunchInstance = 6,
    StopInstance = 7,
    RestartInstance = 8,
    KillInstance = 9,
    CreateInstance = 10,
    DeleteInstance = 11,
    CloneInstance = 12,
    InstanceError = 13,
}

impl AuditAction {
    #[allow(non_upper_case_globals)]
    pub const StartInstance: Self = Self::LaunchInstance;
    #[allow(non_upper_case_globals)]
    pub const InstanceLifecycle: Self = Self::LaunchInstance;

    pub fn code(self) -> i32 {
        self as i32
    }

    pub fn pascal(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::AddAccount => "AddAccount",
            Self::UpdateAccount => "UpdateAccount",
            Self::SwitchAccount => "SwitchAccount",
            Self::SchedulePrompt => "SchedulePrompt",
            Self::RequeueConversation => "RequeueConversation",
            Self::LaunchInstance => "LaunchInstance",
            Self::StopInstance => "StopInstance",
            Self::RestartInstance => "RestartInstance",
            Self::KillInstance => "KillInstance",
            Self::CreateInstance => "CreateInstance",
            Self::DeleteInstance => "DeleteInstance",
            Self::CloneInstance => "CloneInstance",
            Self::InstanceError => "InstanceError",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::AddAccount => "Add Account",
            Self::UpdateAccount => "Update Account",
            Self::SwitchAccount => "Switch Account",
            Self::SchedulePrompt => "Schedule Prompt",
            Self::RequeueConversation => "Requeue Conversation",
            Self::LaunchInstance => "Launch Instance",
            Self::StopInstance => "Stop Instance",
            Self::RestartInstance => "Restart Instance",
            Self::KillInstance => "Kill Instance",
            Self::CreateInstance => "Create Instance",
            Self::DeleteInstance => "Delete Instance",
            Self::CloneInstance => "Clone Instance",
            Self::InstanceError => "Instance Error",
        }
    }

    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Unknown),
            1 => Some(Self::AddAccount),
            2 => Some(Self::UpdateAccount),
            3 => Some(Self::SwitchAccount),
            4 => Some(Self::SchedulePrompt),
            5 => Some(Self::RequeueConversation),
            6 => Some(Self::LaunchInstance),
            7 => Some(Self::StopInstance),
            8 => Some(Self::RestartInstance),
            9 => Some(Self::KillInstance),
            10 => Some(Self::CreateInstance),
            11 => Some(Self::DeleteInstance),
            12 => Some(Self::CloneInstance),
            13 => Some(Self::InstanceError),
            _ => None,
        }
    }

    pub fn from_legacy(raw: &str) -> Option<Self> {
        let folded = raw.trim().to_ascii_lowercase().replace([' ', '_', '-'], "");
        match folded.as_str() {
            "unknown" | "0" => Some(Self::Unknown),
            "addaccount" | "1" => Some(Self::AddAccount),
            "updateaccount" | "2" => Some(Self::UpdateAccount),
            "switchaccount" | "3" => Some(Self::SwitchAccount),
            "scheduleprompt" | "promptscheduler" | "4" => Some(Self::SchedulePrompt),
            "requeueconversation" | "requeue" | "5" => Some(Self::RequeueConversation),
            "startinstance" | "launchinstance" | "instancelifecycle" | "lifecycle" | "launch"
            | "start" | "6" => Some(Self::LaunchInstance),
            "stopinstance" | "closeinstance" | "stop" | "close" | "7" => Some(Self::StopInstance),
            "restartinstance" | "restart" | "8" => Some(Self::RestartInstance),
            "killinstance" | "forcekill" | "kill" | "9" => Some(Self::KillInstance),
            "createinstance" | "newinstance" | "create" | "10" => Some(Self::CreateInstance),
            "deleteinstance" | "removeinstance" | "delete" | "remove" | "11" => {
                Some(Self::DeleteInstance)
            }
            "cloneinstance" | "copyinstance" | "clone" | "copy" | "12" => Some(Self::CloneInstance),
            "instanceerror" | "error" | "13" => Some(Self::InstanceError),
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

        assert_eq!(AuditAction::from_code(6), Some(AuditAction::LaunchInstance));
        assert_eq!(AuditAction::from_code(7), Some(AuditAction::StopInstance));
        assert_eq!(
            AuditAction::from_code(8),
            Some(AuditAction::RestartInstance)
        );
        assert_eq!(AuditAction::from_code(9), Some(AuditAction::KillInstance));
        assert_eq!(
            AuditAction::from_code(10),
            Some(AuditAction::CreateInstance)
        );
        assert_eq!(
            AuditAction::from_code(11),
            Some(AuditAction::DeleteInstance)
        );
        assert_eq!(AuditAction::from_code(12), Some(AuditAction::CloneInstance));
        assert_eq!(AuditAction::from_code(13), Some(AuditAction::InstanceError));

        assert_eq!(
            AuditAction::from_legacy("start_instance"),
            Some(AuditAction::LaunchInstance)
        );
        assert_eq!(
            AuditAction::from_legacy("launch_instance"),
            Some(AuditAction::LaunchInstance)
        );
        assert_eq!(
            AuditAction::from_legacy("stop_instance"),
            Some(AuditAction::StopInstance)
        );
        assert_eq!(
            AuditAction::from_legacy("restart_instance"),
            Some(AuditAction::RestartInstance)
        );
        assert_eq!(
            AuditAction::from_legacy("kill_instance"),
            Some(AuditAction::KillInstance)
        );
        assert_eq!(
            AuditAction::from_legacy("create_instance"),
            Some(AuditAction::CreateInstance)
        );
        assert_eq!(
            AuditAction::from_legacy("delete_instance"),
            Some(AuditAction::DeleteInstance)
        );
        assert_eq!(
            AuditAction::from_legacy("clone_instance"),
            Some(AuditAction::CloneInstance)
        );
    }
}
