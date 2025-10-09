use std::time::{SystemTime, UNIX_EPOCH};

use tokio::fs;

use crate::error::Result;
use crate::AdbManager;

// TODO: Implement streaming
pub async fn dump_logcat(stream: bool, remote_auth_url: Option<String>) -> Result<()> {
    let mut adb = AdbManager::connect(remote_auth_url).await?;

    let result = adb.shell("logcat -d").await?;

    let line_count = result.split("\n").count();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let filename = format!("penumbra_log_dump_{timestamp}.log");

    fs::write(&filename, result).await?;

    println!("Wrote {line_count} lines to {filename}");

    Ok(())
}
