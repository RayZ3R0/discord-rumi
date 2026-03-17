use anyhow::Result;
use poise::serenity_prelude as serenity;

use crate::data::AppData;
use crate::error::Error;
use crate::registration;

/// Handle the Discord `Ready` event.
///
/// Fired once per shard immediately after it finishes identifying with the
/// gateway and receiving the initial guild state. This is the earliest point
/// at which the bot is operational on a given shard.
///
/// ## Responsibilities
///
/// 1. Log the bot username, discriminator, and guild count for this shard.
/// 2. On shard 0, trigger hash-based command auto-registration.
///
/// ## Why shard 0 only for registration
///
/// When running multiple shards all of them fire `Ready` concurrently.
/// Global command registration only needs to happen once — serenity's
/// `start_autosharded` always includes shard 0, so we gate on that.
pub async fn handle(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    framework: poise::FrameworkContext<'_, AppData, Error>,
) -> Result<()> {
    let shard_id = ctx.shard_id;

    tracing::info!(
        shard       = shard_id.0,
        username    = %ready.user.name,
        guild_count = ready.guilds.len(),
        "shard ready"
    );

    if shard_id.0 != 0 {
        return Ok(());
    }

    let hash_path = framework.user_data.command_hash_path.clone();

    // `registration::sync_commands` compares the SHA-256 hash of the current
    // command definitions against the stored hash and only calls Discord's
    // API when the two differ.
    if let Err(e) = registration::sync_commands(
        ctx,
        &framework.options().commands,
        &hash_path,
    )
    .await
    {
        tracing::error!(error = ?e, "command registration failed");
    }

    Ok(())
}
