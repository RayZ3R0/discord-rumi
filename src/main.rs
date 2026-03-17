//! discord-rumi — production-grade Discord bot
//!
//! ## Startup sequence
//!
//! ```text
//! 1. Install tracing subscriber (structured logging)
//! 2. Load .env (dotenvy), parse and validate Config
//! 3. Build poise Framework with all options
//! 4. Build serenity ClientBuilder with minimal cache settings
//! 5. start_autosharded — Discord determines shard count
//! 6. Block until SIGINT or SIGTERM
//! ```
//!
//! ## Environment variables
//!
//! See `.env.example` for the full list with descriptions.

use std::sync::Arc;

use poise::serenity_prelude as serenity;
use tokio::signal;

mod commands;
mod config;
mod data;
mod error;
mod events;
mod registration;

use config::Config;
use data::AppData;

#[tokio::main]
async fn main() {
    // ── 1. Logging ─────────────────────────────────────────────────────────
    //
    // Install the global tracing subscriber before anything else so that
    // every log line emitted during startup is captured.
    //
    // Log level is controlled entirely by the RUST_LOG environment variable.
    // Default: info for this crate, warn for all dependencies.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new(
                        "discord_rumi=info,serenity=warn,poise=warn",
                    )
                }),
        )
        .with_target(false)
        .with_file(cfg!(debug_assertions))
        .with_line_number(cfg!(debug_assertions))
        .init();

    if let Err(e) = run().await {
        tracing::error!(error = ?e, "fatal error during startup");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    // ── 2. Configuration ────────────────────────────────────────────────────
    //
    // dotenvy loads the .env file (if present) into the process environment.
    // It is not an error if .env does not exist — production deployments are
    // expected to set variables through the host environment directly.
    let _ = dotenvy::dotenv();

    let config = Config::load()?;

    tracing::info!("configuration loaded");

    // ── 3. Poise framework ──────────────────────────────────────────────────
    let owner_ids = config.owner_ids.clone();
    let command_hash_path = config.command_hash_path.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),

            // Disable prefix commands entirely — slash-only bot.
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: None,
                ..Default::default()
            },

            // Owner IDs loaded from OWNER_IDS env var.
            owners: owner_ids,

            // Route all unhandled framework errors through our handler.
            on_error: |err| Box::pin(error::on_error(err)),

            // Emit a structured log line before and after every command.
            pre_command: |ctx| Box::pin(error::pre_command(ctx)),
            post_command: |ctx| Box::pin(error::post_command(ctx)),

            // Owners bypass all command checks.
            skip_checks_for_owners: true,

            // Route all gateway events to our event dispatcher.
            event_handler: |ctx, event, framework, data| {
                Box::pin(events::handle(ctx, event, framework, data))
            },

            ..Default::default()
        })
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move { AppData::new(command_hash_path) })
        })
        .build();

    // ── 4. Serenity client ──────────────────────────────────────────────────
    //
    // Intents: GUILDS is the minimum required for slash commands to function.
    // It provides guild create/delete/update events and channel/role data that
    // feeds the cache. No privileged intents are requested.
    //
    // To add intents later, use the bitwise OR operator:
    //   intents | serenity::GatewayIntents::GUILD_MEMBERS
    //
    // Cache: serenity's cache is enabled with its default settings, which
    // already sets max_messages=0. This caches guild/channel/role metadata
    // (needed for permission checks and command routing) while preventing the
    // unbounded per-channel message buffers that cause RAM growth at scale.
    let intents = serenity::GatewayIntents::GUILDS;

    let mut client = serenity::ClientBuilder::new(&config.token, intents)
        .framework(framework)
        .await
        .map_err(|e| anyhow::anyhow!("failed to build serenity client: {}", e))?;

    // ── 5. Start shards ─────────────────────────────────────────────────────
    //
    // `start_autosharded` asks Discord for the recommended shard count and
    // spawns one shard runner per shard. At 200-300 guilds Discord will
    // recommend 1 shard. The bot scales to 2500+ guilds without code changes.
    tracing::info!("connecting to Discord gateway (autosharded)");

    let shard_manager = Arc::clone(&client.shard_manager);

    // Spawn the client in a separate task so we can concurrently listen for
    // the shutdown signal below.
    let client_task = tokio::spawn(async move {
        if let Err(e) = client.start_autosharded().await {
            tracing::error!(error = ?e, "client error");
        }
    });

    // ── 6. Graceful shutdown ────────────────────────────────────────────────
    wait_for_signal().await;

    tracing::info!("shutdown signal received — closing all shards");

    shard_manager.shutdown_all().await;

    // Give the client task a moment to clean up after the shards close.
    let _ = tokio::time::timeout(std::time::Duration::from_secs(10), client_task).await;

    tracing::info!("shutdown complete");
    Ok(())
}

/// Block until SIGINT (Ctrl-C) or SIGTERM arrives.
async fn wait_for_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install SIGINT handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c    => tracing::debug!("SIGINT received"),
        () = terminate => tracing::debug!("SIGTERM received"),
    }
}
