//! GitHub Release checking and version comparison.
//!
//! Queries the GitHub API for the latest release and compares it against the
//! current running version using semantic versioning.

use anyhow::{Context, Result};

use super::UpdateConfig;

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// The new version tag (e.g., "v0.2.0").
    pub version: String,

    /// Direct download URL for the release asset.
    pub download_url: String,

    /// The release asset filename.
    pub asset_name: String,
}

/// Check GitHub Releases API for a newer version than the current one.
///
/// # Returns
///
/// - `Ok(Some(UpdateInfo))` if a newer version is available
/// - `Ok(None)` if already on latest version
/// - `Err(_)` if the API request fails or asset cannot be found
///
/// # Errors
///
/// Returns an error if:
/// - Network request to GitHub fails
/// - No matching asset for the current platform is found
/// - Version parsing fails
pub async fn check_for_update(config: &UpdateConfig) -> Result<Option<UpdateInfo>> {
    tracing::debug!(
        repo = format!("{}/{}", config.repo_owner, config.repo_name),
        current_version = config.current_version,
        "querying GitHub Releases API"
    );

    // Query GitHub for the latest release
    let target = crate::updater::target::detect_target()?;

    let release = self_update::backends::github::ReleaseList::configure()
        .repo_owner(&config.repo_owner)
        .repo_name(&config.repo_name)
        .build()
        .context("failed to configure GitHub release list")?
        .fetch()
        .context("failed to fetch release list from GitHub")?;

    // Get the latest release (releases are sorted newest first)
    let latest = release
        .first()
        .context("no releases found in repository")?;

    // Compare versions using semver
    let latest_version = semver::Version::parse(&latest.version)
        .context("failed to parse latest version as semver")?;

    let current_version = semver::Version::parse(&config.current_version)
        .context("failed to parse current version as semver")?;

    tracing::debug!(
        current = %current_version,
        latest = %latest_version,
        "version comparison"
    );

    if latest_version <= current_version {
        return Ok(None);
    }

    // Find the matching asset for this platform
    let asset = latest
        .asset_for(&target, None)
        .context(format!("no asset found for target: {}", target))?;

    tracing::info!(
        current = %current_version,
        new = %latest_version,
        asset = asset.name,
        "update available"
    );

    Ok(Some(UpdateInfo {
        version: latest.version.clone(),
        download_url: asset.download_url.clone(),
        asset_name: asset.name.clone(),
    }))
}
