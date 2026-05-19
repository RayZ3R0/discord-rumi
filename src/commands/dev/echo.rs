use crate::data::AppData;
use crate::error::Error;

type Context<'a> = poise::Context<'a, AppData, Error>;

/// Developer-only command to echo back text exactly as provided.
///
/// This command is restricted to a specific developer user ID for testing and
/// debugging purposes. It safely echoes back the provided text without any
/// modifications, preserving all content character-by-character.
#[poise::command(
    slash_command,
    description_localized("en-US", "Echo back text (dev only)")
)]
pub async fn echo(
    ctx: Context<'_>,
    #[description = "Text to echo back"] text: String,
) -> Result<(), Error> {
    // Developer user ID constant
    const DEV_USER_ID: u64 = 636598760616624128;

    // Check if the command invoker is the authorized developer
    let author_id = ctx.author().id.get();
    if author_id != DEV_USER_ID {
        return Err(anyhow::anyhow!(
            "Access denied: this is a developer-only command"
        ));
    }

    // Validate text is not empty
    if text.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content("Error: text cannot be empty")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Validate text length (Discord has a 2000 character limit per message)
    if text.len() > 2000 {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "Error: text exceeds 2000 character limit (provided: {} characters)",
                    text.len()
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    // Echo the text back exactly as provided
    ctx.send(
        poise::CreateReply::default()
            .content(text)
            .ephemeral(false), // Public response so dev can verify output
    )
    .await?;

    Ok(())
}
