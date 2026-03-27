//! Automatic self-update system.
//!
//! Periodically checks GitHub Releases for new versions and automatically
//! downloads and installs them. When an update is successfully installed, the
//! bot exits cleanly to allow the process supervisor (systemd, etc.) to restart
//! it with the new binary.
//!
//! ## Design
//!
//! - **Conservative by default**: Updates are disabled unless explicitly enabled
//!   in configuration (`AUTO_UPDATE_ENABLED=true`).
//! - **Background task**: Runs independently without blocking bot operation.
//! - **Atomic replacement**: Uses atomic filesystem operations to prevent
//!   corruption. A backup is created before replacement.
//! - **Graceful exit**: After successful update, exits with code 0 to trigger
//!   process supervisor restart.
//! - **Safe failures**: Network errors, missing assets, or permission issues are
//!   logged but don't crash the bot — it continues running on the current version.
//!
//! ## Update Flow
//!
//! 1. Wait for configured check interval (default: 24 hours)
//! 2. Query GitHub Releases API for latest version
//! 3. Compare with current version using semver
//! 4. If newer version found:
//!    - Download matching release asset for current platform
//!    - Extract binary from tar.gz archive
//!    - Create backup of current binary
//!    - Atomically replace current binary
//!    - Exit cleanly (code 0)
//! 5. Process supervisor (systemd) restarts bot with new binary

mod check;
mod install;
mod target;

use std::time::Duration;

/// Configuration for the automatic update system.
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// GitHub repository owner (e.g., "RayZ3R0").
    pub repo_owner: String,

    /// GitHub repository name (e.g., "discord-rumi").
    pub repo_name: String,

    /// Hours between update checks.
    pub check_interval_hours: u64,

    /// Current version of the running binary (from CARGO_PKG_VERSION).
    pub current_version: String,
}

/// Spawn the automatic update background task.
///
/// This function starts an infinite loop that:
/// 1. Waits for the configured check interval
/// 2. Checks for updates
/// 3. Downloads and installs if a new version is available
/// 4. Exits cleanly to trigger restart
///
/// All errors are logged but do not crash the bot. If any step fails, the bot
/// continues running on the current version and will retry at the next interval.
///
/// # Panics
///
/// Does not panic. All errors are caught and logged.
pub async fn spawn_update_task(config: UpdateConfig) {
    let interval = Duration::from_secs(config.check_interval_hours * 3600);

    tracing::info!(
        interval_hours = config.check_interval_hours,
        repo = format!("{}/{}", config.repo_owner, config.repo_name),
        "automatic update task started"
    );

    loop {
        // Wait for the check interval
        tokio::time::sleep(interval).await;

        tracing::info!("checking for updates...");

        // Check for new version
        match check::check_for_update(&config).await {
            Ok(Some(update)) => {
                tracing::info!(
                    current = config.current_version,
                    new = update.version,
                    "update available, beginning installation"
                );

                // Download and install
                match install::install_update(&update).await {
                    Ok(()) => {
                        tracing::info!(
                            version = update.version,
                            "update installed successfully, restarting bot..."
                        );

                        // Exit cleanly — systemd/supervisor will restart us
                        std::process::exit(0);
                    }
                    Err(e) => {
                        tracing::error!(
                            error = ?e,
                            "failed to install update, continuing on current version"
                        );
                    }
                }
            }
            Ok(None) => {
                tracing::debug!("already on latest version");
            }
            Err(e) => {
                tracing::error!(
                    error = ?e,
                    "failed to check for updates, will retry at next interval"
                );
            }
        }
    }
}
