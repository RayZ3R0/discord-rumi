/// Shared application state injected into every command and event handler.
///
/// `AppData` is constructed once in `main` and stored behind an `Arc` by poise.
/// Every command receives a `&AppData` reference through `ctx.data()`.
///
/// ## Extending this struct
///
/// 1. Add your field here with a brief doc-comment.
/// 2. Initialise it in [`AppData::new`].
/// 3. Access it anywhere via `ctx.data().<field>`.
///
/// The poise `Context` type carries `AppData` by reference, so adding fields
/// never requires changes to command signatures.
pub struct AppData {
    /// Generic HTTP client for outbound requests (webhooks, external APIs, etc.).
    ///
    /// `reqwest::Client` pools connections internally; a single instance shared
    /// across all handlers is more efficient than creating one per request.
    #[allow(dead_code)]
    pub http: reqwest::Client,

    /// Path to the file that stores the SHA-256 hash of the last registered
    /// command set. Read and written by [`crate::registration::sync_commands`].
    pub command_hash_path: String,
    // ── Future fields ──────────────────────────────────────────────────────
    // Uncomment and initialise the field below when you are ready to add a
    // SQLite database. The sqlx pool is Send + Sync and cheap to clone.
    //
    // pub db: sqlx::SqlitePool,
}

impl AppData {
    /// Construct a new [`AppData`] instance.
    ///
    /// Called once during the poise `setup` callback in `main`. Any async
    /// resource that needs to be initialised before the bot is ready (e.g. a
    /// database connection pool) should be created here.
    ///
    /// # Errors
    ///
    /// Returns an error if any resource fails to initialise.
    pub fn new(command_hash_path: String) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;

        // ── Uncomment to initialise a SQLite pool ──────────────────────────
        // let db = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?)
        //     .await?;

        Ok(Self {
            http,
            command_hash_path,
        })
    }
}
