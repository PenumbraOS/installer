use std::fs::File;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::signal::ctrl_c;
use tokio::sync::watch::{self, Sender};
use tokio::task::spawn_blocking;

use crate::{AdbManager, InstallerError};

// Taken from adb_client LogFilter
pub struct LineBuffer<W: Write> {
    writer: W,
    buffer: Vec<u8>,
}

impl<W: Write> LineBuffer<W> {
    pub fn new(writer: W) -> Self {
        LineBuffer {
            writer,
            buffer: Vec::new(),
        }
    }

    fn should_write(&self, _line: &[u8]) -> bool {
        true
    }
}

impl<W: Write> Write for LineBuffer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);

        let buf_clone = self.buffer.clone();
        let mut lines = buf_clone.split_inclusive(|&byte| byte == b'\n').peekable();

        while let Some(line) = lines.next() {
            if lines.peek().is_some() {
                if self.should_write(line) {
                    self.writer.write_all(line)?;
                }
            } else {
                // This is the last (unfinished) element, we keep it for next round
                self.buffer = line.to_vec();
                break;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

struct PrintFileWriter {
    file: File,
    line_count: usize,
    tx: Sender<usize>,
}

impl Write for PrintFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // This isn't correct if there needs to be a retry, but assume it just works
        let result = self.file.write(buf);
        if let Ok(string) = String::from_utf8(buf.into()) {
            println!("{string}");
        }
        self.line_count += 1;
        let _ = self.tx.send(self.line_count);

        result
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

pub async fn dump_logcat_and_exit(stream: bool, remote_auth_url: Option<String>) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let filename = format!("penumbra_log_dump_{timestamp}.log");

    let inner_filename = filename.clone();
    match tokio::spawn(async move {
        let mut adb = AdbManager::connect(remote_auth_url).await?;

        let mut file = File::create(inner_filename)?;

        if stream {
            let (tx, rx) = watch::channel(0);
            spawn_blocking(move || {
                let mut writer = PrintFileWriter {
                    file,
                    line_count: 0,
                    tx,
                };

                let _ = adb.shell_stream("logcat", &mut writer);
            });

            let _ = ctrl_c().await;
            let value = *rx.borrow();
            Ok::<usize, InstallerError>(value)
        } else {
            let result = adb.shell("logcat -d").await?;
            let line_count = result.split("\n").count();
            file.write_all(result.as_bytes())?;
            Ok(line_count)
        }
    })
    .await
    .unwrap()
    {
        Ok(line_count) => {
            println!("\n\nWrote {line_count} lines to {filename}");
            // Forcibly close process due to blocking call staying open
            std::process::exit(0);
        }
        Err(err) => {
            println!("Error: {err}");
            std::process::exit(1);
        }
    };
}
