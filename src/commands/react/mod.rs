//! `/react` command group — targeted reaction GIFs directed at another user.
//!
//! Every subcommand requires a `user` parameter — the person you are
//! directing the action at. For self-expressive actions with no target,
//! see the `/anime` command group (`src/commands/anime/mod.rs`).
//!
//! Shared HTTP and embed helpers are re-exported from the anime module so
//! there is no duplicated code between the two groups.

use poise::serenity_prelude as serenity;

use crate::commands::anime::{nekos_gif, send_reaction};
use crate::data::AppData;
use crate::error::Error;

type Context<'a> = poise::Context<'a, AppData, Error>;

/// Escape markdown characters in user-provided text to prevent unintended formatting.
/// This escapes *, _, ~, `, and other Discord markdown characters.
fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
}

// ── Pastel color palette ─────────────────────────────────────────────────────

/// Warm / affectionate — soft rose
const COLOR_WARM: u32 = 0xF4A7B9;
/// Playful / silly — warm peach
const COLOR_PLAYFUL: u32 = 0xFDDCAE;
/// Aggressive / spicy — muted coral
const COLOR_AGGRESSIVE: u32 = 0xF9B4B4;

// ── Parent command ────────────────────────────────────────────────────────────

/// Anime reaction GIFs — direct an action at another user.
#[poise::command(
    slash_command,
    subcommands(
        // Warm / affectionate
        "hug", "cuddle", "kiss", "peck", "blowkiss", "pat", "handhold",
        "lappillow", "carry", "feed", "kabedon", "wag", "highfive", "handshake",
        // Playful / silly
        "tickle", "poke", "bonk", "baka", "yeet", "teehee", "nom",
        "bite", "shake",
        // Aggressive / spicy
        "slap", "punch",
    ),
    subcommand_required
)]
pub async fn react(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════
// WARM / AFFECTIONATE  (#F4A7B9 — soft rose)
// ════════════════════════════════════════════════════════════════════════════

/// Give someone a warm hug.
#[poise::command(slash_command)]
pub async fn hug(
    ctx: Context<'_>,
    #[description = "Who do you want to hug?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "hug").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** wraps their arms around **{b}**")).await
}

/// Cuddle up with someone.
#[poise::command(slash_command)]
pub async fn cuddle(
    ctx: Context<'_>,
    #[description = "Who do you want to cuddle?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "cuddle").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** cuddles up with **{b}**")).await
}

/// Kiss someone.
#[poise::command(slash_command)]
pub async fn kiss(
    ctx: Context<'_>,
    #[description = "Who do you want to kiss?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "kiss").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** kisses **{b}** 💋")).await
}

/// Give someone a quick peck on the cheek.
#[poise::command(slash_command)]
pub async fn peck(
    ctx: Context<'_>,
    #[description = "Who do you want to peck?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "peck").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** gives **{b}** a quick peck on the cheek")).await
}

/// Blow a kiss at someone.
#[poise::command(slash_command)]
pub async fn blowkiss(
    ctx: Context<'_>,
    #[description = "Who are you blowing a kiss at?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "blowkiss").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** blows a kiss at **{b}** 💨💋")).await
}

/// Give someone headpats.
#[poise::command(slash_command)]
pub async fn pat(
    ctx: Context<'_>,
    #[description = "Who do you want to pat?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "pat").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** gives **{b}** headpats")).await
}

/// Hold someone's hand.
#[poise::command(slash_command)]
pub async fn handhold(
    ctx: Context<'_>,
    #[description = "Whose hand do you want to hold?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "handhold").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** holds **{b}**'s hand")).await
}

/// Use someone's lap as a pillow.
#[poise::command(slash_command)]
pub async fn lappillow(
    ctx: Context<'_>,
    #[description = "Whose lap do you want to use as a pillow?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "lappillow").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** uses **{b}**'s lap as a pillow")).await
}

/// Pick someone up and carry them.
#[poise::command(slash_command)]
pub async fn carry(
    ctx: Context<'_>,
    #[description = "Who do you want to carry?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "carry").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** picks up **{b}** and carries them")).await
}

/// Feed someone. Say aah!
#[poise::command(slash_command)]
pub async fn feed(
    ctx: Context<'_>,
    #[description = "Who do you want to feed?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "feed").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** feeds **{b}** — say aah!")).await
}

/// Kabedon someone against the wall.
#[poise::command(slash_command)]
pub async fn kabedon(
    ctx: Context<'_>,
    #[description = "Who are you kabedon-ing?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "kabedon").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** kabedon's **{b}** against the wall")).await
}

/// Wag your tail at someone.
#[poise::command(slash_command)]
pub async fn wag(
    ctx: Context<'_>,
    #[description = "Who are you wagging your tail at?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "wag").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** wags their tail happily at **{b}**")).await
}

/// High-five someone.
#[poise::command(slash_command)]
pub async fn highfive(
    ctx: Context<'_>,
    #[description = "Who do you want to high-five?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "highfive").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** high-fives **{b}** 🙌")).await
}

/// Shake someone's hand.
#[poise::command(slash_command)]
pub async fn handshake(
    ctx: Context<'_>,
    #[description = "Who do you want to shake hands with?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "handshake").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_WARM, format!("**{a}** shakes hands with **{b}**")).await
}

// ════════════════════════════════════════════════════════════════════════════
// PLAYFUL / SILLY  (#FDDCAE — warm peach)
// ════════════════════════════════════════════════════════════════════════════

/// Tickle someone relentlessly.
#[poise::command(slash_command)]
pub async fn tickle(
    ctx: Context<'_>,
    #[description = "Who do you want to tickle?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "tickle").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_PLAYFUL, format!("**{a}** tickles **{b}** — no mercy!")).await
}

/// Poke someone.
#[poise::command(slash_command)]
pub async fn poke(
    ctx: Context<'_>,
    #[description = "Who do you want to poke?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "poke").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_PLAYFUL, format!("**{a}** pokes **{b}** 👉")).await
}

/// Bonk someone on the head.
#[poise::command(slash_command)]
pub async fn bonk(
    ctx: Context<'_>,
    #[description = "Who deserves a bonk?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "bonk").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_PLAYFUL, format!("**{a}** bonks **{b}** on the head. Horny jail.")).await
}

/// Call someone a baka.
#[poise::command(slash_command)]
pub async fn baka(
    ctx: Context<'_>,
    #[description = "Who's the baka?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "baka").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_PLAYFUL, format!("**{a}** calls **{b}** a baka! BAKA!")).await
}

/// Yeet someone into the stratosphere.
#[poise::command(slash_command)]
pub async fn yeet(
    ctx: Context<'_>,
    #[description = "Who are you yeeting?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "yeet").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_PLAYFUL, format!("**{a}** yeets **{b}** into the stratosphere")).await
}

/// Tee-hee at someone.
#[poise::command(slash_command)]
pub async fn teehee(
    ctx: Context<'_>,
    #[description = "Who are you tee-hee-ing at?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "teehee").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_PLAYFUL, format!("**{a}** goes tee-hee at **{b}** ✌️")).await
}

/// Nom (nibble) on someone.
#[poise::command(slash_command)]
pub async fn nom(
    ctx: Context<'_>,
    #[description = "Who do you want to nom on?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "nom").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_PLAYFUL, format!("**{a}** noms on **{b}** nom nom nom")).await
}

/// Bite someone.
#[poise::command(slash_command)]
pub async fn bite(
    ctx: Context<'_>,
    #[description = "Who do you want to bite?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "bite").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_PLAYFUL, format!("**{a}** bites **{b}** — watch out!")).await
}

/// Shake someone vigorously.
#[poise::command(slash_command)]
pub async fn shake(
    ctx: Context<'_>,
    #[description = "Who do you want to shake?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "shake").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_PLAYFUL, format!("**{a}** shakes **{b}** vigorously")).await
}

/// Slap someone. What did they do?!
#[poise::command(slash_command)]
pub async fn slap(
    ctx: Context<'_>,
    #[description = "Who are you slapping?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "slap").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_AGGRESSIVE, format!("**{a}** slaps **{b}** — what did they do?!")).await
}

/// Punch someone.
#[poise::command(slash_command)]
pub async fn punch(
    ctx: Context<'_>,
    #[description = "Who are you punching?"] user: serenity::User,
) -> Result<(), Error> {
    let (url, anime) = nekos_gif(ctx, "punch").await?;
    let a = escape_markdown(&ctx.author().display_name().to_string());
    let b = escape_markdown(&user.display_name().to_string());
    send_reaction(ctx, url, anime, COLOR_AGGRESSIVE, format!("**{a}** punches **{b}** right in the face 🥊")).await
}
