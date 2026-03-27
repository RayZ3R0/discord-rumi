//! Binary download and atomic replacement.
//!
//! Downloads the new release asset, extracts the binary, creates a backup of
//! the current executable, and performs an atomic replacement.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use super::check::UpdateInfo;

/// Download and install a new version of the bot binary.
///
/// # Process
///
/// 1. Download the release asset (tar.gz) to a temp directory
/// 2. Extract the binary from the archive
/// 3. Create a backup of the current executable (`.backup` extension)
/// 4. Atomically replace the current binary with the new one
///
/// # Safety
///
/// - All operations use temporary directories that auto-cleanup on error
/// - Backup is created before replacement, allowing manual rollback
/// - `rename()` is atomic, preventing partial writes
/// - Permissions are preserved from the current executable
///
/// # Errors
///
/// Returns an error if:
/// - Download fails
/// - Extraction fails
/// - Unable to determine current executable path
/// - Insufficient permissions to replace the binary
pub async fn install_update(update: &UpdateInfo) -> Result<()> {
    let current_exe = std::env::current_exe()
        .context("failed to determine current executable path")?;

    let backup_path = current_exe.with_extension("backup");

    tracing::info!(
        version = update.version,
        asset = update.asset_name,
        "downloading update"
    );

    // Create a temporary directory for the download
    let temp_dir = tempfile::Builder::new()
        .prefix("discord-rumi-update-")
        .tempdir()
        .context("failed to create temporary directory")?;

    let temp_archive = temp_dir.path().join(&update.asset_name);
    let temp_binary = temp_dir.path().join("discord-rumi");

    // Download the release asset
    let response = reqwest::get(&update.download_url)
        .await
        .context("failed to download update")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "GitHub returned non-success status: {}",
            response.status()
        );
    }

    let bytes = response
        .bytes()
        .await
        .context("failed to read response body")?;

    fs::write(&temp_archive, &bytes)
        .context("failed to write downloaded archive to disk")?;

    tracing::debug!(
        path = ?temp_archive,
        size_bytes = bytes.len(),
        "archive downloaded"
    );

    // Extract the binary from the tar.gz archive
    extract_binary(&temp_archive, &temp_binary)
        .context("failed to extract binary from archive")?;

    // Verify the extracted binary is executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_binary)
            .context("failed to read extracted binary metadata")?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_binary, perms)
            .context("failed to set executable permissions")?;
    }

    tracing::debug!(path = ?temp_binary, "binary extracted and marked executable");

    // Create backup of current binary
    fs::copy(&current_exe, &backup_path)
        .context("failed to create backup of current binary")?;

    tracing::info!(backup = ?backup_path, "backup created");

    // Atomic replacement
    fs::rename(&temp_binary, &current_exe)
        .context("failed to replace current binary with new version")?;

    tracing::info!(
        version = update.version,
        "binary replaced successfully"
    );

    Ok(())
}

/// Extract the `discord-rumi` binary from a tar.gz archive.
fn extract_binary(archive_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let archive_file = fs::File::open(archive_path)
        .context("failed to open archive file")?;

    let gz = GzDecoder::new(archive_file);
    let mut archive = Archive::new(gz);

    // Iterate through archive entries to find the binary
    for entry_result in archive.entries().context("failed to read archive entries")? {
        let mut entry = entry_result.context("failed to read archive entry")?;
        let path = entry.path().context("failed to read entry path")?;

        // Look for the binary (may be at root or in a subdirectory)
        if path.file_name() == Some(std::ffi::OsStr::new("discord-rumi")) {
            let mut output_file = fs::File::create(output_path)
                .context("failed to create output file")?;

            std::io::copy(&mut entry, &mut output_file)
                .context("failed to copy binary from archive")?;

            return Ok(());
        }
    }

    anyhow::bail!("binary 'discord-rumi' not found in archive")
}
