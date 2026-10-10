//! Tauri IPC Commands for Email Dispatch and Mailbox Management
//! Exposes email accounts, credentials, import/export, and watcher commands with AppError.

#![allow(dead_code)]

use crate::error::{AppError, AppResult};
use crate::modules::email_inbound;
use crate::modules::email_io::{self, ImportSummary};
use crate::modules::email_sender;
use crate::modules::email_vault_db::{
    self, EmailAccount, EmailAccountInput, EmailNotificationSettings, NotifyRecipient,
    NotifyRecipientInput,
};
use crate::modules::email_watcher::{self, WatcherStatus};
use crate::utils::command::CommandExtWrapper;
use serde::{Deserialize, Serialize};

mod accounts;
mod cli;
mod conn_test;
mod io;
mod settings;
mod watcher;

pub use accounts::*;
pub use cli::*;
pub use conn_test::*;
pub use io::*;
pub use settings::*;
pub use watcher::*;
