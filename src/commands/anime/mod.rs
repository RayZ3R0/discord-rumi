//! `/anime` command group — self-expressive reaction GIFs (no mention needed).
//!
//! These are solo actions where the invoker is the subject: crying, laughing,
//! blushing, sleeping, etc. For actions directed at another user see the
//! `/react` command group (`src/commands/react/mod.rs`).
//!
//! All HTTP requests reuse the shared `reqwest::Client` in `AppData`.
//! The nekos.best API is called at `https://nekos.best/api/v2/<category>`.

use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use serde::Deserialize;

use crate::data::AppData;
use crate::error::Error;

type Context<'a> = poise::Context<'a, AppData, Error>;

// ── Pastel color palette ─────────────────────────────────────────────────────

/// Sad / emotional — soft periwinkle blue
const COLOR_SAD: u32 = 0xB4C8F9;
/// Calm / chill — pale mint
const COLOR_CALM: u32 = 0xC8E6C9;
/// Happy / expressive — light butter yellow
const COLOR_HAPPY: u32 = 0xFFF0A5;

// ── nekos.best API ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct NekosResponse {
    results: Vec<NekosResult>,
}

#[derive(Deserialize)]
struct NekosResult {
    url: String,
    anime_name: Option<String>,
}

pub(crate) async fn nekos_gif(
    ctx: Context<'_>,
    category: &str,
) -> Result<(String, Option<String>), Error> {
    let url = format!("https://nekos.best/api/v2/{}", category);

    let resp: NekosResponse = ctx
        .data()
        .http
        .get(&url)
        .send()
        .await
        .with_context(|| format!("HTTP request to nekos.best/{} failed", category))?
        .error_for_status()
        .with_context(|| format!("nekos.best/{} returned an error status", category))?
        .json()
        .await
        .with_context(|| format!("failed to parse nekos.best/{} response", category))?;

    let result = resp
        .results
        .into_iter()
        .next()
        .with_context(|| format!("nekos.best/{} returned an empty results array", category))?;

    Ok((result.url, result.anime_name))
}

pub(crate) async fn send_reaction(
    ctx: Context<'_>,
    gif_url: String,
    anime_name: Option<String>,
    color: u32,
    text: String,
) -> Result<(), Error> {
    let mut embed = serenity::CreateEmbed::default()
        .description(text)
        .image(gif_url)
        .color(color);

    if let Some(name) = anime_name {
        embed = embed.footer(serenity::CreateEmbedFooter::new(name));
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

// ── Parent command ────────────────────────────────────────────────────────────

/// Anime reaction GIFs — express yourself without targeting anyone.
#[poise::command(
    slash_command,
    subcommands(
        // Happy / expressive
        "smile", "laugh", "blush", "happy", "wink", "wave", "salute", "thumbsup", "nod", "dance",
        // Sad / emotional
        "cry", "pout", "bored", "facepalm", "confused", "shocked",
        // Calm / chill
        "sleep", "yawn", "stare", "think", "sip", "shrug", "nope", "smug", "lurk",
    ),
    subcommand_required
)]
pub async fn anime(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════
// HAPPY / EXPRESSIVE  (#FFF0A5 — light butter yellow)
// ════════════════════════════════════════════════════════════════════════════

/// Flash a warm smile.
#[poise::command(slash_command)]
pub async fn smile(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "smile").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** smiles warmly 😊")).await
}

/// Burst out laughing.
#[poise::command(slash_command)]
pub async fn laugh(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "laugh").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** can't stop laughing 😂")).await
}

/// Blush furiously.
#[poise::command(slash_command)]
pub async fn blush(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "blush").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** is absolutely flustered 😳")).await
}

/// Be openly, uncontrollably happy.
#[poise::command(slash_command)]
pub async fn happy(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "happy").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** is so happy right now! 🎉")).await
}

/// Give a cheeky wink.
#[poise::command(slash_command)]
pub async fn wink(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "wink").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** gives a cheeky wink 😉")).await
}

/// Wave hello or goodbye.
#[poise::command(slash_command)]
pub async fn wave(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "wave").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** waves! 👋")).await
}

/// Stand at attention and salute.
#[poise::command(slash_command)]
pub async fn salute(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "salute").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** salutes. o7")).await
}

/// Give a big thumbs up.
#[poise::command(slash_command)]
pub async fn thumbsup(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "thumbsup").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** gives a big thumbs up 👍")).await
}

/// Nod in agreement.
#[poise::command(slash_command)]
pub async fn nod(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "nod").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** nods. Understood.")).await
}

/// Break into a solo dance.
#[poise::command(slash_command)]
pub async fn dance(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "dance").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_HAPPY, format!("**{a}** breaks into a dance 🕺")).await
}

// ════════════════════════════════════════════════════════════════════════════
// SAD / EMOTIONAL  (#B4C8F9 — soft periwinkle blue)
// ════════════════════════════════════════════════════════════════════════════

/// Let it all out.
#[poise::command(slash_command)]
pub async fn cry(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "cry").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_SAD, format!("**{a}** is crying... 😢")).await
}

/// Pout at the world.
#[poise::command(slash_command)]
pub async fn pout(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "pout").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_SAD, format!("**{a}** is pouting 😤")).await
}

/// Express boredom.
#[poise::command(slash_command)]
pub async fn bored(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "bored").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_SAD, format!("**{a}** is bored out of their mind...")).await
}

/// Facepalm at the state of things.
#[poise::command(slash_command)]
pub async fn facepalm(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "facepalm").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_SAD, format!("**{a}** facepalms. Why is everyone like this.")).await
}

/// Express complete confusion.
#[poise::command(slash_command)]
pub async fn confused(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "confused").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_SAD, format!("**{a}** has absolutely no idea what is going on")).await
}

/// React with shock.
#[poise::command(slash_command)]
pub async fn shocked(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "shocked").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_SAD, format!("**{a}** is utterly shocked 😱")).await
}

// ════════════════════════════════════════════════════════════════════════════
// CALM / CHILL  (#C8E6C9 — pale mint)
// ════════════════════════════════════════════════════════════════════════════

/// Take a well-deserved nap.
#[poise::command(slash_command)]
pub async fn sleep(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "sleep").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_CALM, format!("**{a}** has fallen asleep. Do not disturb. 💤")).await
}

/// Let out a big yawn.
#[poise::command(slash_command)]
pub async fn yawn(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "yawn").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_CALM, format!("**{a}** lets out a big yawn. Someone's tired...")).await
}

/// Stare off into the distance.
#[poise::command(slash_command)]
pub async fn stare(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "stare").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_CALM, format!("**{a}** stares off into the distance...")).await
}

/// Think deeply about something.
#[poise::command(slash_command)]
pub async fn think(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "think").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_CALM, format!("**{a}** is deep in thought 🤔")).await
}

/// Take a quiet, peaceful sip.
#[poise::command(slash_command)]
pub async fn sip(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "sip").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_CALM, format!("**{a}** takes a long, peaceful sip ☕")).await
}

/// Shrug it all off.
#[poise::command(slash_command)]
pub async fn shrug(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "shrug").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_CALM, format!("**{a}** shrugs. ¯\\_(ツ)_/¯")).await
}

/// Nope out of the situation.
#[poise::command(slash_command)]
pub async fn nope(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "nope").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_CALM, format!("**{a}** says: nope. Not today.")).await
}

/// Give the smuggest look possible.
#[poise::command(slash_command)]
pub async fn smug(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "smug").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_CALM, format!("**{a}** gives their smuggest look 😏")).await
}

/// Lurk silently in the shadows.
#[poise::command(slash_command)]
pub async fn lurk(ctx: Context<'_>) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "lurk").await?;
    let a = ctx.author().display_name().to_string();
    send_reaction(ctx, url, anime, COLOR_CALM, format!("**{a}** lurks silently...")).await
}
