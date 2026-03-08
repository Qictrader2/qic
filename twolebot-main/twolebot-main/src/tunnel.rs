use crate::logging::SharedLogger;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

/// Pre-compiled regex for extracting the tunnel URL from cloudflared output.
/// Returns None only if the hardcoded pattern is somehow invalid (should never happen).
fn tunnel_url_regex() -> Option<&'static Regex> {
    static RE: OnceLock<Option<Regex>> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"https://[a-z0-9][-a-z0-9]*\.trycloudflare\.com").ok()
    })
    .as_ref()
}

/// Run a resilient tunnel loop that automatically restarts on failure.
///
/// This function never returns unless `shutdown` is cancelled. It:
/// - Ensures cloudflared is available (downloads if needed)
/// - Spawns a tunnel, waits for the URL
/// - When the tunnel dies, resets the URL to `None` and restarts with backoff
/// - Prints the QR code to the terminal on each successful start
pub async fn run_resilient_tunnel(
    data_dir: PathBuf,
    local_port: u16,
    url_tx: watch::Sender<Option<String>>,
    shutdown: CancellationToken,
    auth_token: String,
    logger: SharedLogger,
) {
    const INITIAL_BACKOFF: Duration = Duration::from_secs(2);
    const MAX_BACKOFF: Duration = Duration::from_secs(60);
    const URL_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

    let mut backoff = INITIAL_BACKOFF;

    loop {
        if shutdown.is_cancelled() {
            break;
        }

        // Step 1: Ensure cloudflared binary is available
        let cloudflared_path = match ensure_cloudflared(&data_dir).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Failed to ensure cloudflared: {}", e);
                logger.warn("tunnel", format!("cloudflared not available: {}", e));
                wait_or_shutdown(&shutdown, backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        };

        // Step 2: Spawn the tunnel process
        let (exit_rx, child_url_rx) = match spawn_tunnel_process(&cloudflared_path, local_port, shutdown.clone()) {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("Failed to spawn tunnel: {}", e);
                logger.warn("tunnel", format!("Failed to spawn: {}", e));
                wait_or_shutdown(&shutdown, backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        };

        // Step 3: Wait for the URL
        let url = match wait_for_url(child_url_rx, URL_WAIT_TIMEOUT).await {
            Some(u) => u,
            None => {
                tracing::warn!("Tunnel: timed out waiting for URL");
                logger.warn("tunnel", "Timed out waiting for tunnel URL");
                wait_or_shutdown(&shutdown, backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
                continue;
            }
        };

        // Step 4: Tunnel is live — publish URL and print QR
        println!();
        println!("  Tunnel: {}", url);
        println!("  Chat:   {}/chat", url);
        println!();
        if !auth_token.is_empty() {
            print_terminal_qr(&format!("{}/auth/qr?token={}", url, auth_token));
            println!("  Scan QR code to log in from your phone");
        } else {
            tracing::warn!("Tunnel: auth token not configured, skipping QR code");
        }
        println!();

        let _ = url_tx.send(Some(url.clone()));
        logger.info("tunnel", format!("Tunnel URL: {}", url));
        backoff = INITIAL_BACKOFF; // Reset backoff on success

        // Step 5: Wait for the tunnel to die or for shutdown
        tokio::select! {
            _ = shutdown.cancelled() => {
                tracing::info!("Tunnel: shutdown requested");
                break;
            }
            _ = exit_rx => {
                tracing::warn!("Tunnel process exited, will restart");
                logger.warn("tunnel", "Tunnel process exited, restarting...");
                // Reset URL so dashboard shows tunnel as inactive
                let _ = url_tx.send(None);
            }
        }

        // Brief pause before restart
        wait_or_shutdown(&shutdown, backoff).await;
    }
}

/// Wait for the specified duration or until shutdown is requested.
async fn wait_or_shutdown(shutdown: &CancellationToken, duration: Duration) {
    tokio::select! {
        _ = shutdown.cancelled() => {}
        _ = tokio::time::sleep(duration) => {}
    }
}

/// Wait for a URL to arrive on the channel (with timeout).
async fn wait_for_url(
    mut rx: watch::Receiver<Option<String>>,
    timeout: Duration,
) -> Option<String> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if let Some(url) = rx.borrow().clone() {
            return Some(url);
        }
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return None;
        }
        if tokio::time::timeout(remaining, rx.changed())
            .await
            .is_err()
        {
            return None;
        }
    }
}

/// Spawn a single cloudflared process and return:
/// - A oneshot receiver that fires when the process exits
/// - A watch receiver for the extracted URL
fn spawn_tunnel_process(
    cloudflared_path: &Path,
    local_port: u16,
    shutdown: CancellationToken,
) -> anyhow::Result<(
    tokio::sync::oneshot::Receiver<()>,
    watch::Receiver<Option<String>>,
)> {
    let local_url = format!("http://localhost:{}", local_port);

    let mut child = TokioCommand::new(cloudflared_path)
        .args(["tunnel", "--url", &local_url])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture cloudflared stderr"))?;

    let (url_tx, url_rx) = watch::channel::<Option<String>>(None);
    let (exit_tx, exit_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    tracing::info!("Tunnel shutdown requested, killing cloudflared");
                    drop(child);
                    break;
                }
                line = lines.next_line() => {
                    match line {
                        Ok(Some(text)) => {
                            tracing::debug!(target: "cloudflared", "{}", text);
                            if let Some(re) = tunnel_url_regex() {
                                if let Some(m) = re.find(&text) {
                                    let tunnel_url = m.as_str().to_string();
                                    tracing::info!("Tunnel URL: {}", tunnel_url);
                                    let _ = url_tx.send(Some(tunnel_url));
                                }
                            }
                        }
                        Ok(None) => {
                            tracing::warn!("cloudflared stderr closed (process exited)");
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("Error reading cloudflared stderr: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        // Signal that the tunnel process has exited
        let _ = exit_tx.send(());
    });

    Ok((exit_rx, url_rx))
}

/// Check if cloudflared is available in PATH, otherwise download it.
pub async fn ensure_cloudflared(data_dir: &Path) -> anyhow::Result<PathBuf> {
    // Check PATH first
    if let Ok(output) = std::process::Command::new("cloudflared")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            tracing::info!(
                "cloudflared found in PATH: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
            return Ok(PathBuf::from("cloudflared"));
        }
    }

    // Check data_dir/bin/cloudflared
    let bin_dir = data_dir.join("bin");
    let cloudflared_path = bin_dir.join("cloudflared");
    if cloudflared_path.exists() {
        if let Ok(output) = std::process::Command::new(&cloudflared_path)
            .arg("--version")
            .output()
        {
            if output.status.success() {
                tracing::info!(
                    "cloudflared found at {}: {}",
                    cloudflared_path.display(),
                    String::from_utf8_lossy(&output.stderr).trim()
                );
                return Ok(cloudflared_path);
            }
        }
    }

    // Download cloudflared
    tracing::info!("cloudflared not found, downloading...");
    download_cloudflared(&bin_dir).await?;
    Ok(cloudflared_path)
}

/// Download the cloudflared binary for the current platform (streamed to disk).
async fn download_cloudflared(bin_dir: &Path) -> anyhow::Result<()> {
    let (os, arch) = platform_tag()?;
    let url = format!(
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-{}-{}",
        os, arch
    );
    tracing::info!("Downloading cloudflared from {}", url);

    std::fs::create_dir_all(bin_dir)?;
    let dest = bin_dir.join("cloudflared");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!(
            "Failed to download cloudflared: HTTP {}",
            resp.status()
        );
    }

    // Stream to disk instead of buffering the full binary in memory
    let mut file = tokio::fs::File::create(&dest).await?;
    let mut stream = resp.bytes_stream();
    let mut total_bytes: u64 = 0;
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        total_bytes += chunk.len() as u64;
    }
    file.flush().await?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))?;
    }

    tracing::info!(
        "cloudflared downloaded to {} ({} bytes)",
        dest.display(),
        total_bytes
    );
    Ok(())
}

/// Get the platform tag for cloudflared downloads.
fn platform_tag() -> anyhow::Result<(&'static str, &'static str)> {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        anyhow::bail!("Unsupported OS for cloudflared auto-download");
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        anyhow::bail!("Unsupported architecture for cloudflared auto-download");
    };

    Ok((os, arch))
}

/// Generate a QR code as an SVG string.
pub fn generate_qr_svg(data: &str) -> Result<String, String> {
    let code = qrcode::QrCode::new(data.as_bytes())
        .map_err(|e| format!("QR encode: {e}"))?;
    let svg = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(200, 200)
        .dark_color(qrcode::render::svg::Color("#e1e4e8"))
        .light_color(qrcode::render::svg::Color("#1c1f26"))
        .build();
    Ok(svg)
}

/// Print a QR code to the terminal using unicode block characters.
pub fn print_terminal_qr(data: &str) {
    let code = match qrcode::QrCode::new(data.as_bytes()) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to generate terminal QR: {}", e);
            return;
        }
    };

    // Use half-block characters for double vertical density
    // Upper half block: ▀, Lower half block: ▄, Full block: █, Space: ' '
    let matrix = code.to_colors();
    let width = code.width() as usize;
    let quiet = 1; // 1 module quiet zone

    // Process two rows at a time using half blocks
    let total_height = matrix.len() / width;
    let padded_height = total_height + 2 * quiet;
    let padded_width = width + 2 * quiet;

    let is_dark = |row: usize, col: usize| -> bool {
        if row < quiet || row >= quiet + total_height || col < quiet || col >= quiet + width {
            return false;
        }
        let matrix_row = row - quiet;
        let matrix_col = col - quiet;
        let idx = matrix_row * width + matrix_col;
        idx < matrix.len() && matrix[idx] == qrcode::types::Color::Dark
    };

    let mut output = String::new();
    let mut row = 0;
    while row < padded_height {
        output.push_str("  "); // indent
        for col in 0..padded_width {
            let top = is_dark(row, col);
            let bottom = if row + 1 < padded_height {
                is_dark(row + 1, col)
            } else {
                false
            };
            // On dark terminal: dark QR modules → white, light → terminal bg
            let ch = match (top, bottom) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };
            output.push(ch);
        }
        output.push('\n');
        row += 2;
    }

    print!("{}", output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_svg() {
        let svg = generate_qr_svg("https://example.com/auth?token=abc123").unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("#e1e4e8")); // dark color
        assert!(svg.contains("#1c1f26")); // light color
    }

    #[test]
    fn test_generate_qr_svg_empty_string() {
        // Even empty strings should produce valid QR
        let svg = generate_qr_svg("").unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_platform_tag() {
        let result = platform_tag();
        assert!(result.is_ok());
        let (os, arch) = result.unwrap();
        assert!(!os.is_empty());
        assert!(!arch.is_empty());
    }

    #[test]
    fn test_url_regex() {
        let re = tunnel_url_regex().expect("regex should compile");
        let line = "2024-01-15T10:30:00Z INF |  https://crazy-words-here.trycloudflare.com  |";
        let m = re.find(line);
        assert!(m.is_some());
        assert_eq!(
            m.unwrap().as_str(),
            "https://crazy-words-here.trycloudflare.com"
        );
    }

    #[test]
    fn test_url_regex_no_match() {
        let re = tunnel_url_regex().expect("regex should compile");
        let line = "2024-01-15T10:30:00Z INF Starting tunnel";
        assert!(re.find(line).is_none());
    }
}
