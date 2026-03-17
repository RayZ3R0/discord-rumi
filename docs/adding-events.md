# Adding a new event handler

This document is the authoritative reference for reacting to Discord gateway events. Follow these steps exactly and nothing else needs to change.

---

## Step 1 — Check the required intent

Every Discord event is gated behind a gateway intent. If the intent is not requested, Discord will not send the event to the bot — no amount of code changes will make it arrive.

Common intents:

| Event | Intent required | Privileged? |
|---|---|---|
| `GuildCreate` / `GuildDelete` | `GUILDS` | No (already enabled) |
| `GuildMemberAddition` / `GuildMemberRemoval` | `GUILD_MEMBERS` | **Yes** |
| `Message` | `GUILD_MESSAGES` + `MESSAGE_CONTENT` | `MESSAGE_CONTENT` is privileged |
| `ReactionAdd` / `ReactionRemove` | `GUILD_MESSAGE_REACTIONS` | No |
| `PresenceUpdate` | `GUILD_PRESENCES` | **Yes** |
| `VoiceStateUpdate` | `GUILD_VOICE_STATES` | No |

Privileged intents must be enabled in the [Discord Developer Portal](https://discord.com/developers/applications) under your bot's settings before they can be requested in code.

If the intent is not already enabled, add it in `src/main.rs`:

```rust
let intents = serenity::GatewayIntents::GUILDS
    | serenity::GatewayIntents::GUILD_MEMBERS;  // example
```

---

## Step 2 — Create the handler file

Create `src/events/<name>.rs`. The function signature must accept whatever data the event variant carries.

Template:

```rust
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Handle the <EventName> gateway event.
///
/// Brief description of what this handler does and why.
pub async fn handle(
    ctx: &serenity::Context,
    // Add event-specific parameters here.
    // For example, GuildCreate carries:
    // guild: &serenity::Guild,
) -> Result<()> {
    tracing::info!("event received");

    // Your logic here.

    Ok(())
}
```

The `ctx` parameter gives access to the serenity cache (`ctx.cache`) and HTTP client (`ctx.http`) if needed.

---

## Step 3 — Wire the handler into the dispatcher

In `src/events/mod.rs`:

1. Add `pub mod <name>;` next to the existing module declarations.
2. Add a match arm inside the `match event` block:

```rust
serenity::FullEvent::GuildCreate { guild, is_new } => {
    if let Err(e) = guild_create::handle(ctx, guild).await {
        tracing::error!(error = ?e, "GuildCreate handler failed");
    }
}
```

The `if let Err(e) = ...` pattern is mandatory — it ensures a failure in this handler does not affect other handlers or the shard connection.

---

## Step 4 — Restart the bot

No registration or hash changes are needed for event handlers. Restart the bot and the new handler will begin receiving events immediately.

---

## Finding the correct FullEvent variant

serenity exposes every Discord gateway event as a variant of `serenity::FullEvent`. The full list is in the [serenity docs](https://docs.rs/serenity/latest/serenity/client/enum.FullEvent.html).

Common variants and their data fields:

```rust
serenity::FullEvent::GuildCreate { guild, is_new }
serenity::FullEvent::GuildDelete { incomplete, full }
serenity::FullEvent::GuildMemberAddition { new_member }
serenity::FullEvent::GuildMemberRemoval { guild_id, user, member_data_if_available }
serenity::FullEvent::Message { new_message }
serenity::FullEvent::ReactionAdd { add_reaction }
serenity::FullEvent::ReactionRemove { removed_reaction }
serenity::FullEvent::VoiceStateUpdate { old, new }
serenity::FullEvent::InteractionCreate { interaction }
```

---

## Checklist

- [ ] Intent required by the event is enabled in `src/main.rs` (check the table above)
- [ ] `src/events/<name>.rs` created with a `pub async fn handle(...)` function
- [ ] `pub mod <name>;` added in `src/events/mod.rs`
- [ ] Match arm added in the `match event` block in `src/events/mod.rs`
- [ ] Bot restarted
