use std::collections::HashSet;

use anyhow::{Context, Result};
use poise::serenity_prelude::UserId;

/// Validated, typed configuration loaded from environment variables at startup.
///
/// All fields are resolved once during [`Config::load`] and then stored in
/// [`crate::data::AppData`]. If any required variable is missing the process
/// exits with a clear error message before the Discord client is constructed.
pub struct Config {
    /// Bot token as returned by the Discord developer portal.
    pub token: String,

    /// User IDs that are treated as bot owners.
    ///
    /// Owners bypass all command checks and can invoke owner-only commands.
    /// Loaded from the comma-separated `OWNER_IDS` env var. An empty set is
    /// valid — it simply means no user has owner-level access.
    pub owner_ids: HashSet<UserId>,

    /// Filesystem path used to persist the registered-command hash.
    ///
    /// On startup the bot reads this file, computes a fresh hash of the
    /// current command definitions, and only calls the Discord registration
    /// API when the two differ. Defaults to `.command_hash`.
    pub command_hash_path: String,

    /// Enable automatic updates from GitHub Releases.
    ///
    /// When enabled, the bot periodically checks for new releases and
    /// automatically downloads and installs them. Defaults to `false`.
    pub auto_update_enabled: bool,

    /// Hours between automatic update checks.
    ///
    /// Only used when `auto_update_enabled` is true. Defaults to 24 hours.
    pub update_check_interval_hours: u64,

    /// GitHub repository owner for release updates.
    ///
    /// Defaults to `RayZ3R0`. Change this if running a fork.
    pub update_repo_owner: String,

    /// GitHub repository name for release updates.
    ///
    /// Defaults to `discord-rumi`. Change this if running a fork.
    pub update_repo_name: String,
}

impl Config {
    /// Load and validate configuration from environment variables.
    ///
    /// `dotenvy` has already populated the environment from `.env` before
    /// this is called (see `main`), so both `.env` and real env vars work.
    ///
    /// # Errors
    ///
    /// Returns an error if `DISCORD_TOKEN` is missing or if any `OWNER_IDS`
    /// entry cannot be parsed as a `u64`.
    pub fn load() -> Result<Self> {
        let token = std::env::var("DISCORD_TOKEN")
            .context("DISCORD_TOKEN is not set — copy .env.example to .env and fill it in")?;

        let owner_ids = load_owner_ids()?;

        let command_hash_path =
            std::env::var("COMMAND_HASH_PATH").unwrap_or_else(|_| ".command_hash".to_owned());

        let auto_update_enabled = std::env::var("AUTO_UPDATE_ENABLED")
            .unwrap_or_else(|_| "false".to_owned())
            .parse::<bool>()
            .unwrap_or(false);

        let update_check_interval_hours = std::env::var("UPDATE_CHECK_HOURS")
            .unwrap_or_else(|_| "24".to_owned())
            .parse::<u64>()
            .unwrap_or(24);

        let update_repo_owner =
            std::env::var("UPDATE_REPO_OWNER").unwrap_or_else(|_| "RayZ3R0".to_owned());

        let update_repo_name =
            std::env::var("UPDATE_REPO_NAME").unwrap_or_else(|_| "discord-rumi".to_owned());

        Ok(Self {
            token,
            owner_ids,
            command_hash_path,
            auto_update_enabled,
            update_check_interval_hours,
            update_repo_owner,
            update_repo_name,
        })
    }
}

fn load_owner_ids() -> Result<HashSet<UserId>> {
    let raw = std::env::var("OWNER_IDS").unwrap_or_default();

    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<u64>()
                .map(UserId::new)
                .with_context(|| format!("OWNER_IDS: {:?} is not a valid u64 user ID", s))
        })
        .collect()
}
