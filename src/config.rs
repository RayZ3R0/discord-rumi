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

        Ok(Self {
            token,
            owner_ids,
            command_hash_path,
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
