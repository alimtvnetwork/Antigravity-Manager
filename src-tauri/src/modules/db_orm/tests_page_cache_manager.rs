#[cfg(test)]
mod tests {
    use super::super::page_cache_manager::{
        apply_page_cache_size, get_current_cache_size, select_profile_for_ram, MemoryProfile,
    };
    use rusqlite::Connection;

    #[test]
    fn test_select_profile_for_ram_tiers() {
        assert_eq!(select_profile_for_ram(2048), MemoryProfile::LowMemory);
        assert_eq!(select_profile_for_ram(4096), MemoryProfile::LowMemory);
        assert_eq!(select_profile_for_ram(8192), MemoryProfile::Standard);
        assert_eq!(select_profile_for_ram(16384), MemoryProfile::Standard);
        assert_eq!(select_profile_for_ram(32768), MemoryProfile::HighThroughput);
        assert_eq!(select_profile_for_ram(65536), MemoryProfile::HighThroughput);
    }

    #[test]
    fn test_cache_size_kib_negative_values() {
        assert_eq!(MemoryProfile::LowMemory.cache_size_kib(), -8192);
        assert_eq!(MemoryProfile::Standard.cache_size_kib(), -32768);
        assert_eq!(MemoryProfile::HighThroughput.cache_size_kib(), -131072);
    }

    #[test]
    fn test_apply_and_get_page_cache_size() {
        let conn = Connection::open_in_memory().unwrap();

        let applied = apply_page_cache_size(&conn, MemoryProfile::Standard).unwrap();
        assert_eq!(applied, -32768);

        let current = get_current_cache_size(&conn).unwrap();
        assert_eq!(current, -32768);

        let high_applied = apply_page_cache_size(&conn, MemoryProfile::HighThroughput).unwrap();
        assert_eq!(high_applied, -131072);

        let high_current = get_current_cache_size(&conn).unwrap();
        assert_eq!(high_current, -131072);
    }
}
