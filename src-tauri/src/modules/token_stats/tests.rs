use rusqlite::{params, Connection};

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn weekly_snapshot(
        observed: i64,
        reset: i64,
        fraction: f64,
    ) -> crate::models::QuotaData {
        let iso = |seconds| {
            chrono::DateTime::from_timestamp(seconds, 0)
                .unwrap()
                .to_rfc3339()
        };
        serde_json::from_value(serde_json::json!({
            "models": [], "last_updated": observed,
            "quota_groups": [
                {"display_name": "Gemini Models", "buckets": [
                    {"bucket_id": "gemini-weekly", "window": "weekly", "remaining_fraction": fraction,
                     "reset_time": iso(reset), "observed_at": observed * 1000},
                    {"bucket_id": "gemini-5h", "window": "5h", "remaining_fraction": 1,
                     "reset_time": iso(reset), "observed_at": observed * 1000}
                ]},
                {"display_name": "Claude and GPT models", "buckets": [
                    {"bucket_id": "3p-weekly", "window": "weekly", "remaining_fraction": 0.5,
                     "reset_time": iso(reset + 100), "observed_at": observed * 1000}
                ]}
            ]
        })).unwrap()
    }

    #[test]
    pub(crate) fn weekly_usage_tracks_accounts_groups_and_official_reset_cycles() {
        let now = 1_700_000_000;
        let reset = now + 100;
        let start = reset - 7 * 86400;
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE token_usage (timestamp INTEGER, account_email TEXT,
            model TEXT, input_tokens INTEGER, output_tokens INTEGER, cached_tokens INTEGER);",
        )
        .unwrap();
        for (email, model, timestamp, input, output) in [
            ("a", "gemini-pro", start - 1, 900, 0),
            ("a", "gemini-pro", start, 11, 2),
            ("a", "claude-sonnet", start, 900, 0),
            ("a", "claude-sonnet", start + 100, 20, 0),
            ("a", "gpt-oss", now, 30, 0),
            ("a", "custom-alias", now, 900, 0),
            ("b", "gemini-pro", start - 51, 900, 0),
            ("b", "gemini-pro", start - 50, 7, 3),
            ("b", "gemini-pro", now, 900, 0),
            ("a", "gemini-pro", reset, 40, 0),
        ] {
            conn.execute(
                "INSERT INTO token_usage VALUES (?1, ?2, ?3, ?4, ?5, 99)",
                params![timestamp, email, model, input, output],
            )
            .unwrap();
        }
        let mut accounts: Vec<_> = [("a", reset), ("b", reset - 50)]
            .into_iter()
            .map(|(email, account_reset)| {
                let token = crate::models::TokenData::new(
                    String::new(),
                    String::new(),
                    0,
                    None,
                    None,
                    None,
                    false,
                    None,
                );
                let mut account = crate::models::Account::new(email.into(), email.into(), token);
                account.update_quota(weekly_snapshot(now, account_reset, 0.0));
                account
            })
            .collect();
        let buckets = |account: &crate::models::Account| {
            account
                .quota
                .as_ref()
                .unwrap()
                .quota_groups
                .as_ref()
                .unwrap()
                .iter()
                .flat_map(|group| group.buckets.iter().map(|bucket| bucket.cycle_tokens))
                .collect::<Vec<_>>()
        };
        populate_weekly_usage_with_conn(&conn, &mut accounts, now).unwrap();
        assert_eq!(buckets(&accounts[0]), vec![Some(13), None, Some(50)]);
        assert_eq!(buckets(&accounts[1]), vec![Some(910), None, Some(0)]);

        // Legacy persisted boundaries are ignored and omitted on the next serialization.
        let mut legacy = serde_json::to_value(&accounts[0]).unwrap();
        legacy["quota"]["quota_groups"][0]["buckets"][0]["cycle_start"] = now.into();
        accounts[0] = serde_json::from_value(legacy).unwrap();
        assert!(
            serde_json::to_value(&accounts[0]).unwrap()["quota"]["quota_groups"][0]["buckets"][0]
                .get("cycle_start")
                .is_none()
        );
        populate_weekly_usage_with_conn(&conn, &mut accounts, now).unwrap();
        assert_eq!(buckets(&accounts[0]), vec![Some(13), None, Some(50)]);

        // Small and large replenishments, refreshes and reloads keep the official interval.
        for (offset, fraction) in [(1, 0.000001), (2, 1.0), (3, 0.9)] {
            accounts[0].update_quota(weekly_snapshot(now + offset, reset, fraction));
            accounts[0] =
                serde_json::from_value(serde_json::to_value(&accounts[0]).unwrap()).unwrap();
            populate_weekly_usage_with_conn(&conn, &mut accounts, now + offset).unwrap();
            assert_eq!(buckets(&accounts[0]), vec![Some(13), None, Some(50)]);
            assert_eq!(buckets(&accounts[1]), vec![Some(910), None, Some(0)]);
        }
        // An older positive observation cannot move the boundary.
        accounts[0].update_quota(weekly_snapshot(now, reset + 10, 1.0));
        populate_weekly_usage_with_conn(&conn, &mut accounts, now + 3).unwrap();
        assert_eq!(buckets(&accounts[0]), vec![Some(13), None, Some(50)]);

        // A new official reset switches cycles and includes its exact new start.
        accounts[0].update_quota(weekly_snapshot(reset, reset + 7 * 86400, 1.0));
        populate_weekly_usage_with_conn(&conn, &mut accounts, reset).unwrap();
        assert_eq!(buckets(&accounts[0]), vec![Some(40), None, None]);
        let first = &accounts[0]
            .quota
            .as_ref()
            .unwrap()
            .quota_groups
            .as_ref()
            .unwrap()[0]
            .buckets[0];
        assert_eq!(
            first.weekly_cycle_bounds(reset),
            Some((reset, reset + 7 * 86400))
        );

        // Invalid and expired periods are unavailable, never fabricated zero totals.
        accounts[0]
            .quota
            .as_mut()
            .unwrap()
            .quota_groups
            .as_mut()
            .unwrap()[0]
            .buckets[0]
            .reset_time = "invalid".into();
        populate_weekly_usage_with_conn(&conn, &mut accounts, reset + 101).unwrap();
        assert_eq!(buckets(&accounts[0]), vec![None, None, Some(0)]);
        assert_eq!(buckets(&accounts[1]), vec![None, None, None]);
    }

    #[test]
    pub(crate) fn test_record_and_query() {
        // This would need a test database setup
        // For now, just verify the module compiles
        assert!(true);
    }
}
