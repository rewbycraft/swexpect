use crate::error::{SwitchExpectError, SwitchExpectResult};
use crate::hay::ReadUntil;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub mod error;
pub mod hay;

trait StorageHelper: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send + Sync> StorageHelper for T {}

pub struct SwitchExpect {
    io: Box<dyn StorageHelper + Send + Sync>,
    buffer: String,
    timeout: Option<Duration>,
}

impl SwitchExpect {
    pub fn new<IO: 'static + AsyncRead + AsyncWrite + Unpin  + Send + Sync>(io: IO, timeout: Option<Duration>) -> SwitchExpect {
        SwitchExpect {
            io: Box::new(io),
            buffer: String::new(),
            timeout,
        }
    }

    pub async fn send(&mut self, s: &str) -> SwitchExpectResult<()> {
        self.io.write_all(s.as_bytes()).await?;
        Ok(())
    }

    pub async fn send_line(&mut self, s: &str) -> SwitchExpectResult<()> {
        let s = format!("{s}\n");
        self.send(&s).await
    }

    pub async fn exp_string(&mut self, s: &str) -> SwitchExpectResult<(String, String)> {
        self.expect(&ReadUntil::String(s.to_string())).await
    }

    pub async fn expect(
        &mut self,
        needle: &ReadUntil,
    ) -> SwitchExpectResult<(String, String)> {
        let mut interval = tokio::time::interval(self.timeout.unwrap_or(Duration::MAX));
        interval.tick().await;
        loop {
            let mut data = [0u8; 128];
            tokio::select! {
                res = self.io.read(&mut data) => {
                    let res = res?;
                    self.buffer.extend(String::from_utf8_lossy(&data[..res]).chars());
                    if let Some((left, right)) = hay::find(needle, &self.buffer, false) {
                        let first = self.buffer.drain(..left).collect();
                        let second = self.buffer.drain(..right - left).collect();
                        return Ok((first, second));
                    }
                },
                _ = interval.tick() => {
                    return Err(SwitchExpectError::ExpectTimeout);
                },
            }
        }
    }

    pub async fn flush(&mut self) -> SwitchExpectResult<()> {
        self.io.flush().await?;
        Ok(())
    }

    pub async fn send_control(&mut self, c: char) -> SwitchExpectResult<()> {
        let code = match c {
            'a'..='z' => c as u8 + 1 - b'a',
            'A'..='Z' => c as u8 + 1 - b'A',
            '[' => 27,
            '\\' => 28,
            ']' => 29,
            '^' => 30,
            '_' => 31,
            _ => return Err(SwitchExpectError::UnknownControlCode(c)),
        };
        self.io.write_all(&[code]).await?;
        // stdout is line buffered, so needs a flush
        self.io.flush().await?;
        Ok(())
    }

}
