use crate::data::AppData;
use crate::error::Error;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use serenity::futures::StreamExt;
use std::time::Duration;

type Context<'a> = poise::Context<'a, AppData, Error>;

/// Urban Dictionary API response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrbanDictionaryResponse {
    pub list: Vec<Definition>,
}

/// A single definition from Urban Dictionary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub word: String,
    pub definition: String,
    pub example: String,
    pub author: String,
}

/// Look up a word in Urban Dictionary and display definitions with pagination.
///
/// Fetches definitions from Urban Dictionary and displays them in a rich embed.
/// Use the navigation buttons to browse through results ranked by popularity.
#[poise::command(
    slash_command,
    description_localized("en-US", "Look up a word in Urban Dictionary")
)]
pub async fn define(
    ctx: Context<'_>,
    #[description = "The word or phrase to define"] word: String,
) -> Result<(), Error> {
    // Defer response since we're making an API call
    ctx.defer().await?;

    // Fetch definitions from Urban Dictionary
    let definitions = match fetch_definitions(ctx.data(), &word).await {
        Ok(defs) => {
            if defs.is_empty() {
                ctx.say(format!("❌ No definitions found for **{}**", word))
                    .await?;
                return Ok(());
            }
            defs
        }
        Err(e) => {
            tracing::error!(error = ?e, word = %word, "Failed to fetch Urban Dictionary");
            ctx.say("❌ Failed to fetch definitions. Please try again later.").await?;
            return Ok(());
        }
    };

    // Definitions are already in order from the API (most popular first)

    tracing::info!(word = %word, count = definitions.len(), "Fetched definitions");

    // Start with the first (highest-scored) definition
    let mut current_index: usize = 0;
    let invoker_id = ctx.author().id;

    // Build and send the initial message with the first definition
    let embed = format_definition_embed(&definitions[current_index], current_index, definitions.len());
    let buttons = build_buttons(current_index, definitions.len());

    let reply = poise::CreateReply::default()
        .embed(embed)
        .components(buttons);

    let handle = ctx.send(reply).await?;

    // Get the message ID before creating the collector
    let message_id = handle.message().await?.id;

    // Listen for button interactions for up to 5 minutes using proper StreamExt pattern
    let mut collector = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .filter(move |mci| mci.message.id == message_id)
        .timeout(Duration::from_secs(300))
        .stream();

    while let Some(interaction) = collector.next().await {
        let button_id = &interaction.data.custom_id;

        match button_id.as_str() {
            "define_prev" => {
                if current_index > 0 {
                    current_index -= 1;

                    let embed = format_definition_embed(&definitions[current_index], current_index, definitions.len());
                    let buttons = build_buttons(current_index, definitions.len());

                    let response = serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .components(buttons),
                    );

                    if let Err(e) = interaction.create_response(&ctx.serenity_context(), response).await {
                        tracing::error!(error = ?e, "Failed to update message on prev button");
                    }
                } else {
                    // Silently defer if already at start
                    let _ = interaction
                        .create_response(
                            &ctx.serenity_context(),
                            serenity::CreateInteractionResponse::Defer(
                                serenity::CreateInteractionResponseMessage::default(),
                            ),
                        )
                        .await;
                }
            }
            "define_next" => {
                if current_index < definitions.len() - 1 {
                    current_index += 1;

                    let embed = format_definition_embed(&definitions[current_index], current_index, definitions.len());
                    let buttons = build_buttons(current_index, definitions.len());

                    let response = serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(embed)
                            .components(buttons),
                    );

                    if let Err(e) = interaction.create_response(&ctx.serenity_context(), response).await {
                        tracing::error!(error = ?e, "Failed to update message on next button");
                    }
                } else {
                    // Silently defer if already at end
                    let _ = interaction
                        .create_response(
                            &ctx.serenity_context(),
                            serenity::CreateInteractionResponse::Defer(
                                serenity::CreateInteractionResponseMessage::default(),
                            ),
                        )
                        .await;
                }
            }
             "define_delete" => {
                 // Check if the user who clicked the button is the original invoker
                 if interaction.user.id == invoker_id {
                     // Acknowledge without showing thinking message
                     let response = serenity::CreateInteractionResponse::Acknowledge;
                     let _ = interaction.create_response(&ctx.serenity_context(), response).await;

                     // Delete the message
                     if let Err(e) = handle.delete(ctx).await {
                         tracing::error!(error = ?e, "Failed to delete message");
                     }
                     break; // Exit the interaction loop
                 } else {
                     // Send ephemeral error message
                     let response = serenity::CreateInteractionResponse::Message(
                         serenity::CreateInteractionResponseMessage::new()
                             .content("❌ Only the user who invoked this command can delete it.")
                             .ephemeral(true),
                     );

                     if let Err(e) = interaction.create_response(&ctx.serenity_context(), response).await {
                         tracing::error!(error = ?e, "Failed to send delete permission error");
                     }
                 }
            }
            _ => {
                tracing::warn!(button_id = %button_id, "Received unknown button interaction");
            }
        }
    }

    // Interaction stream timed out or was closed
    tracing::debug!(word = %word, "Definition interaction session ended");

    Ok(())
}

/// Fetch definitions from the Urban Dictionary API.
async fn fetch_definitions(data: &AppData, word: &str) -> Result<Vec<Definition>, Error> {
    let url = format!(
        "https://api.urbandictionary.com/v0/define?term={}",
        urlencoding::encode(word)
    );

    let response = data.http.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Urban Dictionary API returned {}", response.status()));
    }

    let response_text = response.text().await?;
    let parsed: UrbanDictionaryResponse = serde_json::from_str(&response_text)?;

    Ok(parsed.list)
}

/// Format a definition as a rich Discord embed.
fn format_definition_embed(def: &Definition, index: usize, total: usize) -> serenity::CreateEmbed {
    // Truncate example if it's too long (Discord embed field limit is 1024 chars)
    let example = if def.example.len() > 500 {
        format!("{}...", &def.example[..497])
    } else if def.example.is_empty() {
        "*No example provided*".to_string()
    } else {
        def.example.clone()
    };

    // Truncate definition if it's too long (Discord description limit is 4096 chars)
    let definition_text = if def.definition.len() > 2000 {
        format!("{}...", &def.definition[..1997])
    } else {
        def.definition.clone()
    };

    serenity::CreateEmbed::default()
        .title(format!("📚 {}", def.word))
        .description(definition_text)
        .color(0x7f8dc9) // Neutral slate blue
        .field("📌 Example", example, false)
        .field("👤 Author", &def.author, false)
        .footer(serenity::CreateEmbedFooter::new(
            format!("Definition {} of {} | Urban Dictionary", index + 1, total),
        ))
}

/// Build the action row with navigation and delete buttons.
fn build_buttons(current_index: usize, total: usize) -> Vec<serenity::CreateActionRow> {
    vec![serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new("define_prev")
            .label("Previous")
            .emoji('◀')
            .style(serenity::ButtonStyle::Secondary)
            .disabled(current_index == 0), // Disable if at start
        serenity::CreateButton::new("define_next")
            .label("Next")
            .emoji('▶')
            .style(serenity::ButtonStyle::Secondary)
            .disabled(current_index >= total - 1), // Disable if at end
        serenity::CreateButton::new("define_delete")
            .label("Delete")
            .emoji('🗑')
            .style(serenity::ButtonStyle::Danger),
    ])]
}
