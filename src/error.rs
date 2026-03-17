use poise::serenity_prelude as serenity;

use crate::data::AppData;

/// The single error type used throughout the bot.
///
/// Using `anyhow::Error` as the framework error type gives us:
/// - Automatic `?` propagation in command and event handlers
/// - Rich error context via `.context("...")`
/// - A single, consistent type across the entire codebase
///
/// The type alias is defined in `main.rs` and re-exported as `crate::Error`.
pub type Error = anyhow::Error;

/// Global poise error handler.
///
/// Receives every unhandled [`poise::FrameworkError`] and decides how to
/// surface it — either to the user (ephemeral message) or to the operator
/// (structured log line), or both.
///
/// This function is registered as `FrameworkOptions::on_error` in `main.rs`.
/// Adding new error variants here is the only change needed to customise
/// error presentation bot-wide.
pub async fn on_error(error: poise::FrameworkError<'_, AppData, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            tracing::error!(error = ?error, "fatal: framework setup failed");
        }

        poise::FrameworkError::Command { error, ctx, .. } => {
            tracing::error!(
                command = ctx.command().name,
                error  = ?error,
                "command handler returned an error"
            );
            let msg = format!("An internal error occurred. The bot owner has been notified.");
            let _ = ctx.send(poise::CreateReply::default().content(msg).ephemeral(true)).await;
        }

        poise::FrameworkError::ArgumentParse { error, input, ctx, .. } => {
            let msg = match input {
                Some(input) => format!("Could not parse `{}`: {}", input, error),
                None => format!("Argument error: {}", error),
            };
            let _ = ctx.send(poise::CreateReply::default().content(msg).ephemeral(true)).await;
        }

        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            if let Some(error) = error {
                tracing::warn!(
                    command = ctx.command().name,
                    error  = ?error,
                    "command check failed with error"
                );
            }
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("You do not have permission to use this command.")
                        .ephemeral(true),
                )
                .await;
        }

        poise::FrameworkError::MissingBotPermissions { missing_permissions, ctx, .. } => {
            tracing::warn!(
                command     = ctx.command().name,
                missing     = ?missing_permissions,
                "bot is missing required permissions"
            );
            let msg = format!(
                "I am missing the following permissions to run this command: `{}`",
                missing_permissions
            );
            let _ = ctx.send(poise::CreateReply::default().content(msg).ephemeral(true)).await;
        }

        poise::FrameworkError::MissingUserPermissions { missing_permissions, ctx, .. } => {
            let msg = match missing_permissions {
                Some(p) => format!("You are missing the following permissions: `{}`", p),
                None => "You do not have permission to use this command.".to_owned(),
            };
            let _ = ctx.send(poise::CreateReply::default().content(msg).ephemeral(true)).await;
        }

        poise::FrameworkError::NotAnOwner { ctx, .. } => {
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("This command is restricted to bot owners.")
                        .ephemeral(true),
                )
                .await;
        }

        poise::FrameworkError::GuildOnly { ctx, .. } => {
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("This command can only be used inside a server.")
                        .ephemeral(true),
                )
                .await;
        }

        poise::FrameworkError::DmOnly { ctx, .. } => {
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("This command can only be used in direct messages.")
                        .ephemeral(true),
                )
                .await;
        }

        poise::FrameworkError::NsfwOnly { ctx, .. } => {
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("This command can only be used in age-restricted channels.")
                        .ephemeral(true),
                )
                .await;
        }

        poise::FrameworkError::CommandStructureMismatch { description, ctx, .. } => {
            tracing::error!(
                command     = ctx.command().name,
                description = description,
                "command structure mismatch — re-register commands"
            );
        }

        poise::FrameworkError::CooldownHit { remaining_cooldown, ctx, .. } => {
            let secs = remaining_cooldown.as_secs_f32();
            let msg = format!("You are on cooldown. Try again in **{:.1}s**.", secs);
            let _ = ctx.send(poise::CreateReply::default().content(msg).ephemeral(true)).await;
        }

        other => {
            if let Err(e) = poise::builtins::on_error(other).await {
                tracing::error!(error = ?e, "unhandled framework error");
            }
        }
    }
}

/// Log a structured line when a command is about to execute.
///
/// Registered as `FrameworkOptions::pre_command`. Gives visibility into
/// command usage in production logs without modifying individual commands.
pub async fn pre_command(ctx: poise::Context<'_, AppData, Error>) {
    let guild = ctx
        .guild_id()
        .map(|id| id.to_string())
        .unwrap_or_else(|| "DM".to_owned());

    tracing::info!(
        command  = ctx.command().name,
        user     = %ctx.author().id,
        guild    = guild,
        "command invoked"
    );
}

/// Log a structured line after a command finishes executing.
///
/// Registered as `FrameworkOptions::post_command`. Pairs with
/// [`pre_command`] to give a simple audit trail in logs.
pub async fn post_command(ctx: poise::Context<'_, AppData, Error>) {
    tracing::debug!(command = ctx.command().name, "command completed");
}

/// Convert a serenity [`serenity::FullEvent`] into a loggable string label.
///
/// Used in the event handler to emit a consistent log prefix per event type
/// without a massive match arm in the hot path.
#[allow(dead_code)]
pub fn event_name(event: &serenity::FullEvent) -> &'static str {
    match event {
        serenity::FullEvent::Ready { .. } => "Ready",
        serenity::FullEvent::GuildCreate { .. } => "GuildCreate",
        serenity::FullEvent::GuildDelete { .. } => "GuildDelete",
        serenity::FullEvent::GuildMemberAddition { .. } => "GuildMemberAddition",
        serenity::FullEvent::GuildMemberRemoval { .. } => "GuildMemberRemoval",
        serenity::FullEvent::Message { .. } => "Message",
        serenity::FullEvent::ReactionAdd { .. } => "ReactionAdd",
        serenity::FullEvent::ReactionRemove { .. } => "ReactionRemove",
        serenity::FullEvent::InteractionCreate { .. } => "InteractionCreate",
        serenity::FullEvent::Resume { .. } => "Resume",
        serenity::FullEvent::Ratelimit { .. } => "Ratelimit",
        _ => "Unknown",
    }
}
