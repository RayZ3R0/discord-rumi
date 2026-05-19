use anyhow::Result;
use poise::serenity_prelude as serenity;

use crate::error::Error;

/// Handle prefix-based commands triggered by bot mention.
///
/// Since the bot doesn't have the MESSAGE_CONTENT intent, it can only read
/// messages where it's explicitly mentioned. This handler looks for the
/// pattern: `@bot say <text>` and echoes the text back.
///
/// This allows the dev user to post long content without slash command
/// input field limitations.
pub async fn handle(
    ctx: &serenity::Context,
    msg: &serenity::Message,
) -> Result<(), Error> {
    // Developer user ID constant
    const DEV_USER_ID: u64 = 636598760616624128;

    // Only process messages from the dev user
    if msg.author.id.get() != DEV_USER_ID {
        return Ok(());
    }

    // Check if bot is mentioned at the start of the message
    let bot_id = ctx.cache.current_user().id;
    if !msg.mentions_user_id(bot_id) {
        return Ok(());
    }

    // Extract content after the bot mention
    // The mention format is typically `<@BOT_ID>` at the start
    let mention_str = format!("<@{}>", bot_id);
    let content = match msg.content.strip_prefix(&mention_str) {
        Some(c) => c.trim_start(),
        None => return Ok(()), // Mention might be in a different format, skip
    };

    // Check for the `say` command
    if !content.starts_with("say ") {
        return Ok(());
    }

    // Extract text after "say "
    let text = content.strip_prefix("say ").unwrap_or("").trim();

    // Validate text is not empty
    if text.is_empty() {
        msg.reply(ctx, "Error: text cannot be empty").await?;
        return Ok(());
    }

    // Validate text length (Discord has a 2000 character limit per message)
    if text.len() > 2000 {
        msg.reply(
            ctx,
            format!(
                "Error: text exceeds 2000 character limit (provided: {} characters)",
                text.len()
            ),
        )
        .await?;
        return Ok(());
    }

    // Send the text as-is
    msg.channel_id.say(ctx, text).await?;

    Ok(())
}
