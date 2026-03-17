use std::time::Instant;

use crate::data::AppData;
use crate::error::Error;

type Context<'a> = poise::Context<'a, AppData, Error>;

/// Check whether the bot is responsive and display round-trip latency.
///
/// Sends an ephemeral message, then immediately edits it with the elapsed
/// time between the initial send and the edit response. This measures the
/// true HTTP round-trip to Discord's API and always returns a real value —
/// it never shows "calculating...".
#[poise::command(
    slash_command,
    description_localized("en-US", "Check if the bot is responsive and display latency")
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .content("Pong! Measuring latency...")
                .ephemeral(true),
        )
        .await?;

    let elapsed = start.elapsed();

    reply
        .edit(
            ctx,
            poise::CreateReply::default()
                .content(format!("Pong! Latency: `{}ms`", elapsed.as_millis()))
                .ephemeral(true),
        )
        .await?;

    Ok(())
}
