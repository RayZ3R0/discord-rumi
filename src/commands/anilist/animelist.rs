//! `/animelist` command — display an AniList user's anime and manga statistics.

use anilist_moe::{
    client::AniListClient,
    enums::media_list::MediaListStatus,
    errors::AniListError,
    objects::{stats::UserStatistics, user::User},
};
use poise::serenity_prelude as serenity;
use serde::Serialize;

use crate::data::AppData;
use crate::error::Error;

type Context<'a> = poise::Context<'a, AppData, Error>;

/// Embed accent color — deep crimson, matching the AniList brand palette.
const COLOR: u32 = 0x8B0000;

/// Query to resolve a username to a user ID + profile metadata.
/// Replaces `anilist_moe`'s fetch_one.graphql which hardcodes `$id: Int!`
/// and cannot accept a name lookup.
const USER_BY_NAME_QUERY: &str = r#"
query GetUserByName($name: String) {
    User(name: $name) {
        id
        name
        siteUrl
        avatar { large }
        bannerImage
    }
}
"#;

/// Variables for the name-lookup query.
#[derive(Serialize)]
struct ByNameVars<'a> {
    name: &'a str,
}

/// Clean copy of the user stats GraphQL query.
///
/// The bundled query in `anilist_moe` v0.3.3 contains a corruption on line 83
/// (`}STATS: &s`) which causes AniList to return a 400 syntax error. We own
/// this query string so we are not affected by upstream crate bugs.
const USER_STATS_QUERY: &str = r#"
query GetUserStats($id: Int!) {
    User(id: $id) {
        id
        name
        statistics {
            anime {
                count
                meanScore
                minutesWatched
                episodesWatched
                formats { format count }
                statuses { status minutesWatched count }
                releaseYears { releaseYear count }
                genres { genre count meanScore minutesWatched }
            }
            manga {
                count
                meanScore
                chaptersRead
                volumesRead
                formats { format count }
                statuses { status chaptersRead count }
                releaseYears { releaseYear count }
                genres { genre count meanScore chaptersRead }
            }
        }
    }
}
"#;

/// Variables struct for the stats query — serializes to `{"id": <i32>}`.
#[derive(Serialize)]
struct StatsVars {
    id: i32,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a raw minutes value into a human-readable string like
/// `"12d 3h 45m"`, dropping leading zero components except for the last.
fn fmt_duration(total_minutes: i32) -> String {
    let total = total_minutes.max(0) as u32;
    let days = total / 1440;
    let hours = (total % 1440) / 60;
    let mins = total % 60;

    match (days, hours, mins) {
        (0, 0, m) => format!("{m}m"),
        (0, h, m) => format!("{h}h {m}m"),
        (d, h, m) => format!("{d}d {h}h {m}m"),
    }
}

/// Pull the top N genre names from the (already-sorted) genres vec.
fn top_genres(stats: &UserStatistics, n: usize) -> Vec<String> {
    stats
        .genres
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .take(n)
        .filter_map(|g| g.genre.clone())
        .collect()
}

/// Compute the anime completion percentage, guarded against division by zero
/// and negative results.
///
/// Logic: `floor(100 − (dropped_minutes / completed_minutes) × 100)`.
/// Returns `None` when there is no completed-minutes data to base this on.
fn completion_pct(stats: &UserStatistics) -> Option<u32> {
    let statuses = stats.statuses.as_deref()?;

    let completed_mins = statuses
        .iter()
        .find(|s| matches!(s.status, Some(MediaListStatus::Completed)))
        .and_then(|s| s.minutes_watched)
        .unwrap_or(0);

    let dropped_mins = statuses
        .iter()
        .find(|s| matches!(s.status, Some(MediaListStatus::Dropped)))
        .and_then(|s| s.minutes_watched)
        .unwrap_or(0);

    if completed_mins == 0 {
        return None;
    }

    let pct: f64 = 100.0 - (dropped_mins as f64 / completed_mins as f64) * 100.0;
    Some(pct.floor().clamp(0.0, 100.0) as u32)
}

/// Escape markdown characters in user-provided text to prevent unintended formatting.
/// This escapes *, _, ~, `, and other Discord markdown characters.
fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
}

// ── Command ───────────────────────────────────────────────────────────────────

/// Display AniList anime and manga statistics for a user.
#[poise::command(slash_command)]
pub async fn animelist(
    ctx: Context<'_>,
    #[description = "AniList username"] username: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let client = AniListClient::new();

    // Step 1: resolve username → user ID and profile metadata.
    // anilist_moe's fetch_one.graphql is hardcoded to `$id: Int!` and cannot
    // do name lookups; we use our own USER_BY_NAME_QUERY instead.
    let user: User = match client
        .fetch(USER_BY_NAME_QUERY, Some(&ByNameVars { name: &username }))
        .await
    {
        Ok(u) => {
            tracing::debug!(user = ?u, "animelist: got user response");
            u
        }
        Err(AniListError::NotFound) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("No AniList user found for **{username}**."))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
        Err(e) => return Err(anyhow::anyhow!(e)),
    };

    let user_id = user.id;
    let display_name = user.name.as_deref().unwrap_or(&username);
    let profile_url = user
        .site_url
        .clone()
        .unwrap_or_else(|| format!("https://anilist.co/user/{username}"));
    let avatar_url = user.avatar.as_ref().and_then(|a| a.large.clone());
    let banner_url = user.banner_image.clone();

    // Step 2: fetch full stats using our own clean query string.
    // The bundled stats.graphql in anilist_moe v0.3.3 has a corruption that
    // causes a GraphQL syntax error; USER_STATS_QUERY is our clean replacement.
    let stats_user: User = match client
        .fetch(USER_STATS_QUERY, Some(&StatsVars { id: user_id }))
        .await
    {
        Ok(u) => {
            tracing::debug!(user = ?u, "animelist: got stats response");
            u
        }
        Err(e) => return Err(anyhow::anyhow!(e)),
    };

    let stat_types = stats_user.statistics.as_ref();
    let anime = stat_types.and_then(|s| s.anime.as_ref());
    let manga = stat_types.and_then(|s| s.manga.as_ref());

    // ── Anime fields ──────────────────────────────────────────────────────────

    let anime_count = anime.and_then(|s| s.count).unwrap_or(0);
    let anime_score = anime.and_then(|s| s.mean_score).unwrap_or(0.0);
    let anime_minutes = anime.and_then(|s| s.minutes_watched).unwrap_or(0);
    let anime_episodes = anime.and_then(|s| s.episodes_watched).unwrap_or(0);
    let anime_top_genres = anime.map(|s| top_genres(s, 3)).unwrap_or_default();
    let anime_most_watched_genre = anime
        .and_then(|s| s.genres.as_deref())
        .and_then(|g| g.first())
        .and_then(|g| {
            let name = g.genre.as_deref()?;
            let mins = g.minutes_watched?;
            Some(format!("{name} ({})", fmt_duration(mins)))
        });
    let anime_least_liked_genre = anime
        .and_then(|s| s.genres.as_deref())
        .and_then(|g| g.last())
        .and_then(|g| g.genre.clone());
    let anime_fav_year = anime
        .and_then(|s| s.release_years.as_deref())
        .and_then(|y| y.first())
        .and_then(|y| y.release_year)
        .map(|y| y.to_string());
    let anime_completion = anime.and_then(completion_pct);

    // ── Manga fields ──────────────────────────────────────────────────────────

    let manga_count = manga.and_then(|s| s.count).unwrap_or(0);
    let manga_score = manga.and_then(|s| s.mean_score).unwrap_or(0.0);
    let manga_chapters = manga.and_then(|s| s.chapters_read).unwrap_or(0);
    let manga_volumes = manga.and_then(|s| s.volumes_read).unwrap_or(0);
    let manga_top_genres = manga.map(|s| top_genres(s, 3)).unwrap_or_default();
    let manga_fav_genre = manga
        .and_then(|s| s.genres.as_deref())
        .and_then(|g| g.first())
        .and_then(|g| g.genre.clone());
    let manga_fav_year = manga
        .and_then(|s| s.release_years.as_deref())
        .and_then(|y| y.first())
        .and_then(|y| y.release_year)
        .map(|y| y.to_string());

    // ── Build embed ───────────────────────────────────────────────────────────

    let mut embed = serenity::CreateEmbed::default()
        .title(escape_markdown(display_name))
        .url(&profile_url)
        .color(COLOR);

    if let Some(url) = avatar_url {
        embed = embed.thumbnail(url);
    }
    if let Some(url) = banner_url {
        embed = embed.image(url);
    }

    // Anime stats field
    let anime_body = format!(
        "**{anime_count}** entries · **{anime_score:.1}** avg\n\
         **{}** watched · **{anime_episodes}** eps",
        fmt_duration(anime_minutes),
    );
    embed = embed.field("📺 Anime", anime_body, true);

    // Manga stats field
    let manga_body = format!(
        "**{manga_count}** entries · **{manga_score:.1}** avg\n\
         **{manga_chapters}** chapters · **{manga_volumes}** vols",
    );
    embed = embed.field("📖 Manga", manga_body, true);

    // Weeb Tendencies field
    let mut tendencies: Vec<String> = Vec::new();

    if !anime_top_genres.is_empty() {
        let escaped_genres: Vec<String> = anime_top_genres.iter().map(|g| escape_markdown(g)).collect();
        tendencies.push(format!("**Top anime:** {}", escaped_genres.join(", ")));
    }
    if !manga_top_genres.is_empty() {
        let escaped_genres: Vec<String> = manga_top_genres.iter().map(|g| escape_markdown(g)).collect();
        tendencies.push(format!("**Top manga:** {}", escaped_genres.join(", ")));
    }
    if let Some(g) = anime_most_watched_genre {
        tendencies.push(format!("**Most watched:** {}", escape_markdown(&g)));
    }
    if let Some(g) = anime_least_liked_genre {
        tendencies.push(format!("**Least watched:** {}", escape_markdown(&g)));
    }
    if let Some(g) = manga_fav_genre {
        tendencies.push(format!("**Most read:** {}", escape_markdown(&g)));
    }
    if let Some(pct) = anime_completion {
        tendencies.push(format!("**Completion:** {pct}%"));
    }
    if let Some(y) = anime_fav_year {
        tendencies.push(format!("**Fav anime era:** {y}"));
    }
    if let Some(y) = manga_fav_year {
        tendencies.push(format!("**Fav manga era:** {y}"));
    }

    if !tendencies.is_empty() {
        embed = embed.field("📊 Weeb Tendencies", tendencies.join("\n"), false);
    }

    tracing::debug!(?embed, "animelist: final embed before sending");
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
