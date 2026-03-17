# Architecture

## Overview

`discord-twilight` is a production-grade Discord bot built on **serenity** + **poise**. It is designed to run on low-resource hosting (shared VPS, single-core container) while serving 200–300 guilds efficiently.

---

## Technology choices

| Crate | Version | Role | Why |
|---|---|---|---|
| `poise` | 0.6 | Slash command framework | Built on serenity, eliminates boilerplate for argument parsing, checks, error routing, and command registration |
| `serenity` | 0.12 | Discord gateway + REST | Battle-tested, async-first, configurable cache |
| `tokio` | 1 | Async runtime | Standard choice for production Rust services |
| `tracing` + `tracing-subscriber` | 0.1 / 0.3 | Structured logging | Async-aware, integrates with serenity's own instrumentation |
| `dotenvy` | 0.15 | Config loading | Loads `.env` in development, no-op in production |
| `anyhow` | 1 | Error propagation | Single error type across the whole codebase via `?` |
| `sha2` + `hex` | 0.10 / 0.4 | Command hashing | SHA-256 fingerprint of command definitions for smart re-registration |
| `serde_json` | 1 | JSON serialization | Serialize command list for hashing; already a transitive dep of serenity |

---

## Module map

```
src/
├── main.rs           Entrypoint. Bootstraps logging, config, framework, client.
│                     Registers the signal handler and drives autosharded startup.
│
├── config.rs         Typed configuration loaded from environment variables.
│                     Fails fast with a clear error if required vars are missing.
│
├── data.rs           AppData struct — shared state injected into every command
│                     and event handler via poise's context system.
│
├── error.rs          Global on_error handler. Maps every FrameworkError variant
│                     to either a user-facing ephemeral message, a log line, or both.
│                     Also houses pre_command and post_command hooks for audit logging.
│
├── registration.rs   Hash-based command auto-registration. Compares the SHA-256
│                     of the current command definitions against a stored hash file
│                     and only calls Discord's API when the two differ.
│
├── events/
│   ├── mod.rs        Top-level event dispatcher. Receives every FullEvent from
│   │                 serenity and routes it to sub-handlers via a match arm.
│   └── ready.rs      Ready event: logs shard info, triggers command registration
│                     on shard 0 only.
│
└── commands/
    ├── mod.rs        all() function — the single list of all registered commands.
    └── utility/
        ├── mod.rs    Category module declaration.
        └── ping.rs   /ping command implementation.
```

---

## Type aliases

Defined in the files that use them (commands reference `crate::error::Error`):

```rust
type Error = anyhow::Error;
type Context<'a> = poise::Context<'a, AppData, Error>;
```

`Context<'a>` carries a reference to `AppData`, the serenity HTTP client, the cache, and the current invocation metadata. Every command receives it as its first argument.

---

## Shared state — AppData

`AppData` is constructed once during the poise `setup` callback and stored inside the framework. Poise passes an immutable reference to it via `ctx.data()` in every command, and via the `data` parameter in every event handler.

It is **not** wrapped in `Arc` explicitly — poise handles that internally.

```rust
pub struct AppData {
    pub http: reqwest::Client,
    // pub db: sqlx::SqlitePool,   ← slot for future SQLite pool
}
```

To add a field: declare it, initialise it in `AppData::new()`, access it with `ctx.data().your_field`.

---

## Cache strategy (low RAM)

serenity's built-in cache is **enabled** but configured conservatively:

| Setting | Value | Rationale |
|---|---|---|
| `max_messages` | `0` | Disables per-channel message buffering — the #1 source of unbounded RAM growth at scale |
| Intents | `GUILDS` only | No privileged intents, no message content, no presence data |

At 200–300 guilds the cache holds guild metadata, channel lists, and role lists. Typical steady-state RAM usage with this configuration is **under 50 MB**.

To add intents, use the `|` operator in `main.rs`:

```rust
let intents = serenity::GatewayIntents::GUILDS
    | serenity::GatewayIntents::GUILD_MEMBERS;  // requires privileged intent in dev portal
```

---

## Command registration — hash-based auto-registration

On every startup, `registration::sync_commands` is called from the `Ready` event handler (shard 0 only):

1. Serialize the current command list to JSON via `serde_json`.
2. SHA-256 hash the bytes.
3. Read the stored hash from `.command_hash` (configurable via `COMMAND_HASH_PATH`).
4. If hashes match → skip. Log "command definitions unchanged".
5. If they differ → call `serenity::Command::set_global_commands`, write new hash.

**Global command propagation** can take up to 1 hour on Discord's side after registration. This is a Discord limitation, not a bot limitation.

---

## Error handling

All errors flow through `error::on_error`. The policy per variant:

| FrameworkError variant | User message | Log level |
|---|---|---|
| `Command` | "An internal error occurred" (ephemeral) | `error` |
| `ArgumentParse` | Specific parse error (ephemeral) | — |
| `CommandCheckFailed` | "No permission" (ephemeral) | `warn` if error present |
| `MissingBotPermissions` | Lists missing perms (ephemeral) | `warn` |
| `MissingUserPermissions` | Lists missing perms (ephemeral) | — |
| `NotAnOwner` | "Restricted to bot owners" (ephemeral) | — |
| `CooldownHit` | Remaining time (ephemeral) | — |
| Others | Delegated to `poise::builtins::on_error` | varies |

---

## Graceful shutdown

`tokio::signal` listens for `SIGINT` (Ctrl-C) and `SIGTERM` (systemd/Docker stop) concurrently. On signal:

1. `shard_manager.shutdown_all()` — sends WebSocket close frames to all shards.
2. A 10-second timeout waits for the client task to exit.
3. Process exits cleanly.

---

## Sharding

`client.start_autosharded()` queries `GET /gateway/bot` at startup and creates the recommended number of shards. At 200–300 guilds this will be 1 shard. As the bot grows past 2,500 guilds, Discord will recommend more shards on the next restart — no code changes needed.

---

## Logging

Controlled entirely by the `RUST_LOG` environment variable. Recommended values:

| Scenario | Value |
|---|---|
| Production | `discord_twilight=info,serenity=warn,poise=warn` |
| Development | `discord_twilight=debug,serenity=info,poise=debug` |
| Gateway debugging | `discord_twilight=debug,serenity=trace` |

Structured fields (e.g. `command=ping user=123`) are emitted as key=value pairs in the default `fmt` format. Switch to `tracing_subscriber::fmt().json()` in `main.rs` for JSON output in log aggregation pipelines.
