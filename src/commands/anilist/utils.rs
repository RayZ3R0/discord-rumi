use anilist_moe::{
    enums::media_list::MediaListStatus,
    objects::stats::UserStatistics,
};

/// Embed accent color — deep crimson, matching the AniList brand palette.
pub const COLOR: u32 = 0x8B0000;

/// Convert a raw minutes value into a human-readable string like
/// `"12d 3h 45m"`, dropping leading zero components except for the last.
pub fn fmt_duration(total_minutes: i32) -> String {
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
pub fn top_genres(stats: &UserStatistics, n: usize) -> Vec<String> {
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
pub fn completion_pct(stats: &UserStatistics) -> Option<u32> {
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
pub fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
}
