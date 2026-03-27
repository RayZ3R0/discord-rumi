//! AniList command group — user profile and list statistics.
//!
//! Uses the `anilist_moe` crate which provides a typed Rust wrapper around
//! the AniList GraphQL API. Each command in this module creates its own
//! short-lived `AniListClient` — the client is cheap to construct and holds
//! no persistent connection state between invocations.

use crate::data::AppData;
use crate::error::Error;

pub type Context<'a> = poise::Context<'a, AppData, Error>;

pub mod user;
pub mod utils;

pub use user::user;

// ── Parent command group ───────────────────────────────────────────────────────

/// AniList user profile and statistics — lookup and analyze anime/manga lists.
#[poise::command(
    slash_command,
    subcommands("user"),
    subcommand_required
)]
pub async fn anilist(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
