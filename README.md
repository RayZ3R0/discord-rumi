# discord-rumi

A production-grade Discord bot built with Rust, [serenity](https://github.com/serenity-rs/serenity) and [poise](https://github.com/serenity-rs/poise). Designed to run efficiently on low-resource hosting — a single shared VPS serving hundreds of servers with a modest memory footprint.

The binary is named `discord-rumi`, released via [RayZ3R0/discord-rumi](https://github.com/RayZ3R0/discord-rumi).

---

## Features

**Slash commands only.** Prefix commands are disabled entirely. All interactions go through Discord's native slash command interface.

**Hash-based command auto-registration.** On every startup the bot computes a SHA-256 fingerprint of its current command definitions and compares it against a stored hash file. Discord's registration API is only called when the definitions have actually changed. This avoids hitting rate limits on routine restarts and eliminates unnecessary API traffic.

**Minimal cache footprint.** The serenity cache is enabled but configured conservatively: per-channel message buffering is disabled (`max_messages = 0`) and only the `GUILDS` intent is requested. At 200–300 servers the bot holds guild metadata, channel lists, and role data in memory — typically under 50 MB at steady state.

**Structured logging.** All log output goes through `tracing` with `tracing-subscriber`. The log level is controlled at runtime via `RUST_LOG` without rebuilding. Log lines carry structured key-value fields (`command=ping user=123456`) for easy filtering in any log aggregation pipeline.

**Graceful shutdown.** The bot listens for `SIGINT` (Ctrl-C) and `SIGTERM` concurrently. On either signal it closes all shard WebSocket connections cleanly before the process exits, preventing mid-interaction drops.

**Global error handling.** Every possible framework error — argument parse failures, missing permissions, cooldown hits, internal panics — is routed through a single handler. Users receive a short ephemeral message; operators get a structured log line. No error silently swallows itself.

**Autosharding.** At startup the bot queries Discord for the recommended shard count and spawns accordingly. At a few hundred servers this will be one shard. The bot scales past 2,500 servers on the next restart with no code changes.

---

## Commands

| Command | Description |
|---|---|
| `/ping` | Checks whether the bot is responsive and displays the HTTP round-trip latency to Discord's API. |

---

## Running from a prebuilt binary

Prebuilt binaries for each release are attached to the [GitHub Releases](../../releases) page. Three targets are provided:

| Filename | Target | Use case |
|---|---|---|
| `discord-rumi-x86_64-linux-gnu` | x86-64, dynamically linked | Standard Linux VPS or server |
| `discord-rumi-x86_64-linux-musl` | x86-64, fully static | Any Linux host, no libc dependency |
| `discord-rumi-aarch64-linux-gnu` | ARM 64-bit | Raspberry Pi, ARM VPS |

If you are unsure which to pick, use the `musl` build — it is fully self-contained and runs on any Linux system without installing additional libraries.

### Step 1 — Download the binary

```bash
# Replace v0.1.0 and the filename with the version and target you want
wget https://github.com/RayZ3R0/discord-rumi/releases/download/v0.1.0/discord-rumi-x86_64-linux-musl
chmod +x discord-rumi-x86_64-linux-musl
```

### Step 2 — Create a `.env` file

The bot reads its configuration from environment variables. The easiest way to provide them is a `.env` file next to the binary:

```bash
cp .env.example .env   # if you have the example, or create the file manually
```

Minimal `.env`:

```
DISCORD_TOKEN=your-token-here
```

Full `.env` with all options:

```
# Required. Your bot token from https://discord.com/developers/applications
DISCORD_TOKEN=your-token-here

# Optional. Comma-separated Discord user IDs granted owner-level access.
# Owners bypass all permission checks and can run owner-only commands.
# Leave empty if you do not need owner commands.
OWNER_IDS=123456789012345678,987654321098765432

# Optional. Controls log verbosity. Default: info for the bot, warn for libs.
# Other useful values:
#   discord_twilight=debug             verbose output for this bot only
#   discord_twilight=debug,serenity=warn
RUST_LOG=info

# Optional. Where to store the command hash file. Defaults to .command_hash
# next to the running binary.
COMMAND_HASH_PATH=.command_hash
```

### Step 3 — Run the bot

```bash
./discord-rumi-x86_64-linux-musl
```

The bot will log its startup sequence, connect to Discord, and register slash commands if the definitions have changed since the last run:

```
INFO configuration loaded
INFO connecting to Discord gateway (autosharded)
INFO shard ready shard=0 username=YourBot guild_count=42
INFO command definitions changed — registering globally hash=... command_count=1
INFO global slash commands registered successfully
```

On subsequent restarts with no command changes:

```
INFO command definitions unchanged — skipping registration hash=...
```

### Running as a systemd service

Create `/etc/systemd/system/discord-rumi.service`:

```ini
[Unit]
Description=discord-rumi bot
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=discord
WorkingDirectory=/opt/discord-rumi
ExecStart=/opt/discord-rumi/discord-rumi-x86_64-linux-musl
Restart=on-failure
RestartSec=5
# Pass secrets via environment directly instead of a .env file if preferred:
# Environment=DISCORD_TOKEN=your-token-here

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now discord-rumi
sudo journalctl -u discord-rumi -f   # follow logs
```

---

## Building from source

### Prerequisites

- Rust 1.74 or later — install via [rustup](https://rustup.rs)
- A bot token from the [Discord Developer Portal](https://discord.com/developers/applications)

### Clone and build

```bash
git clone https://github.com/RayZ3R0/discord-rumi.git
cd discord-rumi
cargo build --release
```

The release binary will be at `target/release/discord-rumi`.

### Development run

```bash
cp .env.example .env
# Edit .env with your token
cargo run
```

`cargo run` uses the debug profile (faster compilation, larger binary, more verbose backtraces). For production always use `--release`.

---

## Getting a bot token

1. Go to [https://discord.com/developers/applications](https://discord.com/developers/applications) and create a new application.
2. Open the **Bot** tab and click **Add Bot**.
3. Under **Token**, click **Reset Token** and copy the value. This is your `DISCORD_TOKEN`. Treat it like a password — never commit it to version control.
4. Under **Privileged Gateway Intents**, leave all intents off unless you specifically need them (the bot only requires `GUILDS` by default).
5. To invite the bot to a server, go to **OAuth2 > URL Generator**, select the `bot` and `applications.commands` scopes, select the permissions your bot needs, and open the generated URL.

---

## Configuration reference

| Variable | Required | Default | Description |
|---|---|---|---|
| `DISCORD_TOKEN` | Yes | — | Bot token from the Discord developer portal |
| `OWNER_IDS` | No | *(empty)* | Comma-separated user IDs with owner-level access |
| `RUST_LOG` | No | `discord_twilight=info,serenity=warn,poise=warn` | Log filter — see [EnvFilter docs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) |
| `COMMAND_HASH_PATH` | No | `.command_hash` | Path to the command registration hash file |

---

## Project structure

```
src/
├── main.rs           Entrypoint. Bootstraps logging, config, framework, and client.
├── config.rs         Typed configuration loaded from environment variables.
├── data.rs           AppData — shared state accessible from every command and event.
├── error.rs          Global error handler and pre/post command hooks.
├── registration.rs   Hash-based slash command auto-registration.
├── events/
│   ├── mod.rs        Event dispatcher — routes gateway events to sub-handlers.
│   └── ready.rs      Ready event handler — logs shard info, triggers registration.
└── commands/
    ├── mod.rs        Central command list — the only place commands are registered.
    └── utility/
        └── ping.rs   /ping implementation.
```

Detailed guides for extending the bot are in the `docs/` directory:

- [`docs/architecture.md`](docs/architecture.md) — design decisions, cache strategy, error policy, sharding
- [`docs/adding-commands.md`](docs/adding-commands.md) — step-by-step guide for adding a new slash command
- [`docs/adding-events.md`](docs/adding-events.md) — step-by-step guide for handling a new gateway event

---

## Releasing a new version

The GitHub Actions workflow builds release binaries automatically when a version tag is pushed:

```bash
git tag v0.2.0
git push --tags
```

The workflow compiles all three targets in parallel, then creates a GitHub Release with the binaries attached and a changelog generated from commits since the previous tag. No bot token or secrets are needed in CI — the binary reads configuration from the host environment at runtime.

---

## License

MIT
