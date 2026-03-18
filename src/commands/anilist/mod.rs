//! AniList command group — user profile and list statistics.
//!
//! Uses the `anilist_moe` crate which provides a typed Rust wrapper around
//! the AniList GraphQL API. Each command in this module creates its own
//! short-lived `AniListClient` — the client is cheap to construct and holds
//! no persistent connection state between invocations.

pub mod animelist;
