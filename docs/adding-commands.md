# Adding a new slash command

This document is the authoritative reference for adding a command to the bot. Follow these steps exactly and nothing else needs to change.

---

## Step 1 — Create the command file

Create `src/commands/<category>/<name>.rs`. Use an existing category (e.g. `utility`) or create a new one.

Copy this template:

```rust
use crate::data::AppData;
use crate::error::Error;

type Context<'a> = poise::Context<'a, AppData, Error>;

/// One-sentence description shown to users in Discord's command picker.
#[poise::command(
    slash_command,
    // guild_only,               // uncomment to restrict to servers
    // owners_only,              // uncomment to restrict to bot owners
    // required_permissions = "MANAGE_MESSAGES",  // uncomment to require perms
)]
pub async fn my_command(
    ctx: Context<'_>,
    // Parameters become slash command options automatically.
    // #[description = "A required string option"]
    // text: String,
    // #[description = "An optional integer"]
    // count: Option<i64>,
) -> Result<(), Error> {
    ctx.send(
        poise::CreateReply::default()
            .content("Your response here.")
            .ephemeral(true),  // remove for public responses
    )
    .await?;

    Ok(())
}
```

### Parameter types

Poise parses slash command options from Rust types automatically:

| Rust type | Discord option type |
|---|---|
| `String` | String |
| `i64` / `u64` | Integer |
| `f64` | Number |
| `bool` | Boolean |
| `serenity::User` | User |
| `serenity::Channel` | Channel |
| `serenity::Role` | Role |
| `Option<T>` | Optional version of T |

---

## Step 2 — Expose the module

In the category's `mod.rs` (e.g. `src/commands/utility/mod.rs`), add:

```rust
pub mod my_command;
```

If you are creating a **new category**, also:

1. Create `src/commands/<new_category>/mod.rs` with `pub mod my_command;`
2. Add `pub mod <new_category>;` in `src/commands/mod.rs`

---

## Step 3 — Register the command

In `src/commands/mod.rs`, add the command constructor to the `vec!` inside `all()`:

```rust
pub fn all() -> Vec<poise::Command<AppData, Error>> {
    vec![
        utility::ping::ping(),
        utility::my_command::my_command(),  // ← add this line
    ]
}
```

---

## Step 4 — Restart the bot

On the next startup, `registration::sync_commands` detects that the command hash has changed and re-registers all global commands with Discord automatically.

Global command propagation takes **up to 1 hour** on Discord's side. During development, you can register guild-scoped commands for instant propagation — see the note below.

---

## Development tip — instant registration during development

Global commands can take up to 1 hour to propagate. For faster iteration during development, temporarily change the registration call in `src/events/ready.rs` from `set_global_commands` to `set_guild_commands`:

```rust
// In src/events/ready.rs, inside register_global_commands:
serenity::GuildId::new(YOUR_TEST_GUILD_ID)
    .set_commands(ctx, create_commands)
    .await?;
```

Revert to `set_global_commands` before deploying to production.

---

## Subcommands

To add a command group with subcommands:

```rust
// src/commands/admin/mod.rs
pub mod settings;
pub mod reset;

// src/commands/admin/admin.rs
use crate::data::AppData;
use crate::error::Error;
type Context<'a> = poise::Context<'a, AppData, Error>;

/// Admin commands
#[poise::command(slash_command, subcommands("settings", "reset"), owners_only)]
pub async fn admin(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
```

Register it in `commands/mod.rs` as a single entry: `admin::admin::admin()`.

---

## Checklist

- [ ] `src/commands/<category>/<name>.rs` created with `#[poise::command(slash_command)]`
- [ ] Module declared with `pub mod <name>;` in the category's `mod.rs`
- [ ] Constructor call added to the `vec!` in `src/commands/mod.rs`
- [ ] Bot restarted — hash auto-registration handles the rest
