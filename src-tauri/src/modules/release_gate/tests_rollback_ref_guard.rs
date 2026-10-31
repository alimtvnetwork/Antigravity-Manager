#[cfg(test)]
mod tests {
    use super::super::rollback_ref_guard::{
        format_backup_ref_name, is_valid_backup_ref, parse_drift_stat, DriftReport,
    };

    #[test]
    fn test_format_backup_ref_name() {
        let name = format_backup_ref_name("task-191-test", 1792652000);
        assert_eq!(name, "backup/task-191-test-1792652000");
    }

    #[test]
    fn test_is_valid_backup_ref() {
        assert!(is_valid_backup_ref("backup/task-191-1792652000"));
        assert!(is_valid_backup_ref("backup/pre-rewrite-sync"));
        assert!(!is_valid_backup_ref("main"));
        assert!(!is_valid_backup_ref("backup/"));
        assert!(!is_valid_backup_ref("feature/backup/something"));
    }

    #[test]
    fn test_parse_drift_stat_empty() {
        let report = parse_drift_stat("");
        assert!(!report.has_drift);
        assert_eq!(report.files_changed, 0);
    }

    #[test]
    fn test_parse_drift_stat_with_changes() {
        let stat = " src/App.tsx | 4 +++-\n 1 file changed, 3 insertions(+), 1 deletion(-)";
        let report = parse_drift_stat(stat);
        assert!(report.has_drift);
        assert_eq!(report.files_changed, 1);
        assert_eq!(report.insertions, 3);
        assert_eq!(report.deletions, 1);
    }

    #[test]
    fn test_parse_drift_stat_zero_changes() {
        let stat = " 0 files changed";
        let report = parse_drift_stat(stat);
        assert!(!report.has_drift);
    }
}
