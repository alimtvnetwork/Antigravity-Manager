//! Image-generation concurrency scheduler and its unit tests.
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Default)]
struct ImageAccountState {
    enabled: bool,
    in_use: usize,
}

#[derive(Default)]
struct ImageSchedulerState {
    accounts: HashMap<String, ImageAccountState>,
}

pub struct ImageScheduler {
    per_account_concurrency: usize,
    state: Mutex<ImageSchedulerState>,
    change_tx: watch::Sender<u64>,
}

pub struct ImagePermit {
    account_id: String,
    scheduler: Arc<ImageScheduler>,
}

impl Drop for ImagePermit {
    fn drop(&mut self) {
        let mut state = self
            .scheduler
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let remove_account = if let Some(account) = state.accounts.get_mut(&self.account_id) {
            account.in_use = account.in_use.saturating_sub(1);
            !account.enabled && account.in_use == 0
        } else {
            false
        };
        if remove_account {
            state.accounts.remove(&self.account_id);
        }
        drop(state);
        self.scheduler.notify_change();
    }
}

impl ImageScheduler {
    fn new(account_ids: Vec<String>, per_account_concurrency: usize) -> Arc<Self> {
        let (change_tx, _change_rx) = watch::channel(0);
        let scheduler = Arc::new(Self {
            per_account_concurrency,
            state: Mutex::new(ImageSchedulerState::default()),
            change_tx,
        });
        scheduler.sync_accounts(account_ids);
        scheduler
    }

    fn notify_change(&self) {
        self.change_tx.send_modify(|generation| {
            *generation = generation.wrapping_add(1);
        });
    }

    pub(crate) fn subscribe_changes(&self) -> watch::Receiver<u64> {
        self.change_tx.subscribe()
    }

    pub(crate) fn try_acquire(self: &Arc<Self>, account_id: &str) -> Option<ImagePermit> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let account = state.accounts.get_mut(account_id)?;
        if !account.enabled || account.in_use >= self.per_account_concurrency {
            return None;
        }

        account.in_use += 1;
        Some(ImagePermit {
            account_id: account_id.to_string(),
            scheduler: self.clone(),
        })
    }

    pub(crate) fn sync_accounts(&self, account_ids: Vec<String>) {
        let enabled_accounts: HashSet<String> = account_ids.into_iter().collect();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let old_enabled_count = state
            .accounts
            .values()
            .filter(|account| account.enabled)
            .count();

        for (account_id, account) in &mut state.accounts {
            account.enabled = enabled_accounts.contains(account_id);
        }
        for account_id in &enabled_accounts {
            state
                .accounts
                .entry(account_id.clone())
                .or_default()
                .enabled = true;
        }
        state
            .accounts
            .retain(|_, account| account.enabled || account.in_use > 0);

        let new_enabled_count = enabled_accounts.len();
        drop(state);
        self.notify_change();

        if old_enabled_count != new_enabled_count {
            tracing::info!(
                account_count = new_enabled_count,
                per_account_concurrency = self.per_account_concurrency,
                "Image scheduler capacity updated"
            );
        }
    }

    pub(crate) fn available_slots(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .accounts
            .values()
            .filter(|account| account.enabled)
            .map(|account| self.per_account_concurrency.saturating_sub(account.in_use))
            .sum()
    }

    #[cfg(test)]
    fn in_use_for(&self, account_id: &str) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .accounts
            .get(account_id)
            .map_or(0, |account| account.in_use)
    }
}

pub(crate) fn build_image_scheduler(
    account_ids: Vec<String>,
    per_account_concurrency: usize,
) -> Arc<ImageScheduler> {
    ImageScheduler::new(account_ids, per_account_concurrency)
}

#[cfg(test)]
mod image_scheduler_tests {
    use super::{build_image_scheduler, ImagePermit, ImageScheduler};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::{mpsc, oneshot, Semaphore};
    use tokio::task::JoinSet;

    async fn acquire(scheduler: &Arc<ImageScheduler>, account_ids: &[String]) -> ImagePermit {
        let mut changes = scheduler.subscribe_changes();
        loop {
            changes.borrow_and_update();
            for account_id in account_ids {
                if let Some(permit) = scheduler.try_acquire(account_id) {
                    return permit;
                }
            }
            changes.changed().await.unwrap();
        }
    }

    #[tokio::test]
    async fn task_image_scheduler_enforces_each_account_capacity_and_queues_excess() {
        let account_ids: Vec<String> = (0..3).map(|index| format!("account-{index}")).collect();
        let scheduler = build_image_scheduler(account_ids.clone(), 2);
        let release_gate = std::sync::Arc::new(Semaphore::new(0));
        let (started_tx, mut started_rx) = mpsc::channel(8);
        let mut tasks = JoinSet::new();

        for task_index in 0..8 {
            let scheduler = scheduler.clone();
            let account_ids = account_ids.clone();
            let release_gate = release_gate.clone();
            let started_tx = started_tx.clone();
            tasks.spawn(async move {
                let permit = acquire(&scheduler, &account_ids).await;
                started_tx
                    .send((task_index, permit.account_id.clone()))
                    .await
                    .unwrap();
                let release = release_gate.acquire().await.unwrap();
                release.forget();
            });
        }
        drop(started_tx);

        let mut per_account = HashMap::new();
        for _ in 0..6 {
            let (_, account_id) = tokio::time::timeout(Duration::from_secs(1), started_rx.recv())
                .await
                .unwrap()
                .unwrap();
            *per_account.entry(account_id).or_insert(0) += 1;
        }
        assert_eq!(per_account.len(), 3);
        assert!(per_account.values().all(|count| *count == 2));
        assert_eq!(scheduler.available_slots(), 0);
        assert!(
            tokio::time::timeout(Duration::from_millis(50), started_rx.recv())
                .await
                .is_err()
        );

        release_gate.add_permits(1);
        tasks.join_next().await.unwrap().unwrap();
        tokio::time::timeout(Duration::from_secs(1), started_rx.recv())
            .await
            .unwrap()
            .unwrap();
        release_gate.add_permits(8);
        while let Some(task) = tasks.join_next().await {
            task.unwrap();
        }
    }

    #[tokio::test]
    async fn task_image_permits_release_and_account_sync_preserves_inflight_usage() {
        let account_ids = vec!["account-1".to_string()];
        let scheduler = build_image_scheduler(account_ids.clone(), 1);
        let mut tasks = JoinSet::new();

        let success_scheduler = scheduler.clone();
        let success_accounts = account_ids.clone();
        tasks.spawn(async move {
            let _permit = acquire(&success_scheduler, &success_accounts).await;
            Ok::<(), ()>(())
        });
        assert!(tasks.join_next().await.unwrap().unwrap().is_ok());
        assert_eq!(scheduler.available_slots(), 1);

        let failure_scheduler = scheduler.clone();
        let failure_accounts = account_ids.clone();
        tasks.spawn(async move {
            let _permit = acquire(&failure_scheduler, &failure_accounts).await;
            Err::<(), ()>(())
        });
        assert!(tasks.join_next().await.unwrap().unwrap().is_err());
        assert_eq!(scheduler.available_slots(), 1);

        let (acquired_tx, acquired_rx) = oneshot::channel();
        let cancelled_scheduler = scheduler.clone();
        let cancelled_accounts = account_ids.clone();
        let cancelled = tokio::spawn(async move {
            let _permit = acquire(&cancelled_scheduler, &cancelled_accounts).await;
            // Justification: oneshot::Sender::send returns Result<(), ()> — the unit error carries no information to log; a dropped receiver is benign here.
            let _ = acquired_tx.send(());
            std::future::pending::<()>().await;
        });
        acquired_rx.await.unwrap();
        assert_eq!(scheduler.in_use_for("account-1"), 1);
        cancelled.abort();
        assert!(cancelled.await.unwrap_err().is_cancelled());
        assert_eq!(scheduler.available_slots(), 1);

        let permit = acquire(&scheduler, &account_ids).await;
        scheduler.sync_accounts(Vec::new());
        scheduler.sync_accounts(account_ids.clone());
        assert!(scheduler.try_acquire("account-1").is_none());
        drop(permit);
        assert_eq!(scheduler.available_slots(), 1);
    }

    #[test]
    fn account_priority_web_response_preserves_saved_and_default_values() {
        let token = crate::models::TokenData::new(
            "test".into(),
            "test".into(),
            3600,
            None,
            None,
            None,
            false,
            None,
        );
        let mut account = crate::models::Account::new(
            "test-priority".into(),
            "test-priority@test.invalid".into(),
            token,
        );

        for priority in [7, 50] {
            account.priority = priority;
            let payload = serde_json::to_value(crate::proxy::server::dto::to_account_response(
                &account, &None,
            ))
            .unwrap();
            assert_eq!(payload.get("priority"), Some(&serde_json::json!(priority)));
        }
    }

    #[test]
    fn test_switch_request_deserialization_with_and_without_target_ide() {
        use crate::proxy::server::admin_accounts::SwitchRequest;
        let with_ide: SwitchRequest =
            serde_json::from_str(r#"{"accountId": "acc_1", "targetIde": "agy"}"#).unwrap();
        assert_eq!(with_ide.account_id, "acc_1");
        assert_eq!(with_ide.target_ide.as_deref(), Some("agy"));

        let without_ide: SwitchRequest = serde_json::from_str(r#"{"accountId": "acc_2"}"#).unwrap();
        assert_eq!(without_ide.account_id, "acc_2");
        assert_eq!(without_ide.target_ide, None);
    }

    #[tokio::test]
    async fn test_bind_tcp_listener_and_reuse() {
        let port = 18099;
        let listener1 = crate::proxy::server::bind::bind_tcp_listener("127.0.0.1", port)
            .expect("first bind should succeed");
        drop(listener1);
        let listener2 = crate::proxy::server::bind::bind_tcp_listener("127.0.0.1", port)
            .expect("immediate re-bind must succeed with SO_REUSEADDR");
        drop(listener2);
    }

    #[tokio::test]
    async fn test_bind_tcp_listener_wildcard_dual_stack() {
        let port = 18100;
        let listener = crate::proxy::server::bind::bind_tcp_listener("0.0.0.0", port)
            .expect("wildcard dual-stack bind should succeed");
        drop(listener);
        let listener_v6 = crate::proxy::server::bind::bind_tcp_listener("::", port)
            .expect("wildcard v6 dual-stack bind should succeed");
        drop(listener_v6);
    }
}
