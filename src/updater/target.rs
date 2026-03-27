//! Runtime target platform detection.
//!
//! Determines the correct GitHub Release asset name for the current binary's
//! architecture and libc variant. This allows each deployed binary to download
//! the exact matching replacement during self-update.

use anyhow::Result;

/// Detect the current binary's target triple and return the corresponding
/// GitHub Release asset name.
///
/// # Format
///
/// Returns: `discord-rumi-{arch}-linux-{libc}`
///
/// Where:
/// - `{arch}` is `x86_64` or `aarch64`
/// - `{libc}` is `gnu` or `musl`
///
/// # Examples
///
/// - `discord-rumi-x86_64-linux-musl`
/// - `discord-rumi-x86_64-linux-gnu`
/// - `discord-rumi-aarch64-linux-gnu`
///
/// # Errors
///
/// Returns an error if the libc variant cannot be determined at compile time.
pub fn detect_target() -> Result<String> {
    let arch = std::env::consts::ARCH;
    let libc = detect_libc()?;

    Ok(format!("discord-rumi-{}-linux-{}", arch, libc))
}

/// Detect the libc variant using compile-time target environment.
///
/// This is more reliable than runtime detection because each binary is compiled
/// specifically for one libc. The compile-time cfg flag embeds this information
/// directly into the binary.
fn detect_libc() -> Result<&'static str> {
    #[cfg(target_env = "musl")]
    {
        Ok("musl")
    }

    #[cfg(target_env = "gnu")]
    {
        Ok("gnu")
    }

    #[cfg(not(any(target_env = "musl", target_env = "gnu")))]
    {
        anyhow::bail!("unable to detect libc variant — unsupported target_env")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_target_format() {
        let target = detect_target().expect("should detect target");

        // Should match pattern: discord-rumi-{arch}-linux-{gnu|musl}
        assert!(target.starts_with("discord-rumi-"));
        assert!(target.contains("-linux-"));
        assert!(target.ends_with("-gnu") || target.ends_with("-musl"));
    }

    #[test]
    fn test_detect_libc() {
        let libc = detect_libc().expect("should detect libc");
        assert!(libc == "gnu" || libc == "musl");
    }
}
