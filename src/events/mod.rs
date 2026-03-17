use poise::serenity_prelude as serenity;

use crate::data::AppData;
use crate::error::Error;

pub mod ready;

/// Top-level gateway event dispatcher.
///
/// Receives every [`serenity::FullEvent`] that serenity delivers and routes it
/// to the appropriate sub-handler. This function is registered as
/// `FrameworkOptions::event_handler` in `main.rs`.
///
/// ## Adding a new event handler
///
/// 1. Create `src/events/<name>.rs` with a `pub async fn handle(...)` function.
/// 2. Add `pub mod <name>;` below the existing module declarations.
/// 3. Add a match arm in the `match event` block below:
///    ```rust
///    serenity::FullEvent::GuildCreate { guild, .. } => {
///        if let Err(e) = guild_create::handle(ctx, guild).await {
///            tracing::error!(error = ?e, "GuildCreate handler failed");
///        }
///    }
///    ```
/// 4. If the event requires additional gateway intents, add them in `main.rs`.
///
/// That is the complete workflow — no other files need to be touched.
///
/// ## Error policy
///
/// Each handler call is individually guarded with `if let Err(e) = ...`.
/// A failure in one handler does not affect others or the shard connection.
pub async fn handle(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, AppData, Error>,
    _data: &AppData,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            if let Err(e) = ready::handle(ctx, data_about_bot, framework).await {
                tracing::error!(error = ?e, "Ready handler failed");
            }
        }

        serenity::FullEvent::Ratelimit { data } => {
            tracing::warn!(
                timeout_ms = data.timeout.as_millis(),
                limit      = data.limit,
                "Discord rate limit hit"
            );
        }

        // ── Add new event handlers here ────────────────────────────────────
        //
        // serenity::FullEvent::GuildCreate { guild, .. } => {
        //     if let Err(e) = guild_create::handle(ctx, guild).await {
        //         tracing::error!(error = ?e, "GuildCreate handler failed");
        //     }
        // }

        _ => {}
    }

    Ok(())
}
