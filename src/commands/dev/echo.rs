use crate::data::AppData;
use crate::error::Error;

type Context<'a> = poise::Context<'a, AppData, Error>;

/// Developer-only command to post server rules.
///
/// This command is restricted to a specific developer user ID and posts
/// pre-formatted server rules to the channel.
#[poise::command(
    slash_command,
    description_localized("en-US", "Post server rules (dev only)")
)]
pub async fn echo(
    ctx: Context<'_>,
) -> Result<(), Error> {
    // Developer user ID constant
    const DEV_USER_ID: u64 = 636598760616624128;

    // Check if the command invoker is the authorized developer
    let author_id = ctx.author().id.get();
    if author_id != DEV_USER_ID {
        return Err(anyhow::anyhow!(
            "Access denied: this is a developer-only command"
        ));
    }

    let rules = r#"Okay, I've drafted the rules proper now. What y'all think?

1. **Server Theme**
    This server is dedicated to **Kurumi Tokisaki** from Date a Live. While the discussion of the anime and its characters is welcome, attempts to provoke, antagonize, insult or disrupt the community or its interests are not tolerated.

2. **Language of Communication**
    English 🇬🇧/🇺🇲 is the primary language of this server. Non-English conversations belong in <#901338477021499413>. 

3. **Code of Conduct**
    Use common sense and respect others. Harassment, trolling, excessive toxicity, disruptive behavior, attention-seeking, drama-farming, inappropriate conduct, or behavior negatively affecting the server atmosphere may result in moderation action.
    Keep sensitive or divisive topics, excessive personal matters, and off-topic discussions to appropriate channels or avoid them entirely.
    Use channels for their intended purpose.

4. **Appropriate Content**
    Do not engage in or promote behavior violating Discord's Terms of Service. 
    NSFW content is only permitted in designated NSFW channels.
    Gore, exploitative or illegal content is not allowed.

5. **Staff Authority**
    Staff reserve the right to interpret and enforce the rules as necessary to maintain the server environment and community standards. Users may be removed for behavior deemed harmful, disruptive, or incompatible with the server atmosphere, even if not explicitly covered by these rules."#;

    ctx.send(
        poise::CreateReply::default()
            .content(rules)
            .ephemeral(false),
    )
    .await?;

    Ok(())
}

