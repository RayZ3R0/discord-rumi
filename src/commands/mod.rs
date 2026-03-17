use crate::data::AppData;
use crate::error::Error;

pub mod utility;

/// Returns the complete list of poise commands registered with the framework.
///
/// This is the **single place** you touch when adding a new command to the bot.
///
/// ## Workflow
///
/// 1. Create your command module under the appropriate category directory.
/// 2. Expose it via its category's `mod.rs` (see `utility/mod.rs`).
/// 3. Add the top-level command constructor call to the `vec!` below.
///
/// On next startup, the hash-based registration logic in
/// [`crate::registration`] will detect that the command set changed and
/// re-register with Discord automatically.
///
/// Commands are grouped into category sub-modules for maintainability, but
/// from poise's perspective they are a flat list — Discord handles grouping
/// via the command name and subcommand structure.
pub fn all() -> Vec<poise::Command<AppData, Error>> {
    vec![
        utility::ping::ping(),
        // ── Add new commands here ──────────────────────────────────────────
        // utility::help::help(),
        // moderation::ban::ban(),
    ]
}
