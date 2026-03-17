use anyhow::{Context, Result};
use poise::serenity_prelude as serenity;
use sha2::{Digest, Sha256};

use crate::data::AppData;
use crate::error::Error;

/// Compare the current command set against the stored hash and register
/// globally with Discord only when they differ.
///
/// ## Algorithm
///
/// 1. Serialize the full list of [`serenity::CreateCommand`] builders to JSON.
/// 2. SHA-256 hash the resulting bytes.
/// 3. Read the previously stored hash from `hash_path` (if it exists).
/// 4. If the hashes match → skip registration and return early.
/// 5. If they differ (or no hash file exists) → call `register_globally`,
///    then write the new hash to `hash_path`.
///
/// ## Why this is safe
///
/// `poise::builtins::create_application_commands` produces a deterministic
/// `Vec<CreateCommand>` from the command definitions compiled into the binary.
/// The JSON serialization of that vec is stable across runs of the same binary.
/// The hash therefore changes if and only if command definitions change, which
/// is exactly when re-registration is necessary.
///
/// ## Hash file
///
/// The file contains a single 64-character lowercase hex string with no
/// trailing newline. It is safe to delete manually to force re-registration.
pub async fn sync_commands(
    ctx: &serenity::Context,
    commands: &[poise::Command<AppData, Error>],
    hash_path: &str,
) -> Result<()> {
    let create_commands = poise::builtins::create_application_commands(commands);

    let current_hash = hash_commands(&create_commands)?;
    let stored_hash = read_stored_hash(hash_path);

    if stored_hash.as_deref() == Some(current_hash.as_str()) {
        tracing::info!(
            hash      = %current_hash,
            hash_file = hash_path,
            "command definitions unchanged — skipping registration"
        );
        return Ok(());
    }

    let previous = stored_hash.as_deref().unwrap_or("<none>");
    tracing::info!(
        previous_hash = previous,
        new_hash      = %current_hash,
        command_count = create_commands.len(),
        "command definitions changed — registering globally"
    );

    serenity::Command::set_global_commands(ctx, create_commands)
        .await
        .context("failed to register global slash commands with Discord")?;

    write_hash(hash_path, &current_hash)
        .context("failed to write command hash file after registration")?;

    tracing::info!(
        hash          = %current_hash,
        command_count = commands.len(),
        "global slash commands registered successfully"
    );

    Ok(())
}

fn hash_commands(commands: &[serenity::CreateCommand]) -> Result<String> {
    let json = serde_json::to_vec(commands)
        .context("failed to serialize command definitions to JSON for hashing")?;

    let mut hasher = Sha256::new();
    hasher.update(&json);
    Ok(hex::encode(hasher.finalize()))
}

fn read_stored_hash(path: &str) -> Option<String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let trimmed = contents.trim().to_owned();
            if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                Some(trimmed)
            } else {
                tracing::warn!(path, "command hash file contains unexpected content — ignoring");
                None
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            tracing::warn!(path, error = ?e, "could not read command hash file — will re-register");
            None
        }
    }
}

fn write_hash(path: &str, hash: &str) -> std::io::Result<()> {
    std::fs::write(path, hash)
}
