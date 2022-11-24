use anyhow::Result;
use bytes::{Buf, Bytes, BytesMut};
use tokio_util::codec::{Decoder, FramedRead};
use tokio_stream::StreamExt;
use std::num::Wrapping;

/// Store data from mk3 device into influxdb
pub async fn run(path: &str) -> Result<()> {
    let builder = serial_io::build(path, 2400);
    let serial = serial_io::AsyncSerial::from_builder(&builder)?;

    let decoder = VeMk3Decoder::default();
    let mut frame_reader = FramedRead::new(serial, decoder);

    while let Some(result) = frame_reader.next().await {
        match result {
            Ok(frame) => {
                log::debug!("frame: {}", frame);
            }
            Err(e) => {
                log::error!("error: {}", e);
            }
        }
    }
    Ok(())
}

pub struct VeMk3Decoder {
    synchronized: bool,
}

impl Default for VeMk3Decoder {
    fn default() -> Self {
        Self {
            synchronized: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Frame {
    Version,
    LedStatus,
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Version => { write!(f, "version") }
            Self::LedStatus => { write!(f, "led") }
        }
    }
}

impl VeMk3Decoder {
    fn decode_synchronized(&mut self, src: &mut BytesMut) -> Result<Option<Frame>> {
        if src.len() < 1 {
            log::debug!("decode sync, waiting for length byte");
            Ok(None)
        } else {
            let expected_len: usize = <u8 as Into<usize>>::into(src[0]) + 2;

            if src.len() < expected_len {
                log::debug!("decode sync, waiting for expected length");
                Ok(None)
            } else {
                let result = {
                    if src[1] == 0xff && src[2] == 0x56 {
                        // version frame
                        Ok(Some(Frame::Version))
                    } else {
                        Ok(None)
                    }    
                };
                src.advance(expected_len);
                result        
            }
        }
    }

    fn decode_unsynchronized(&mut self, src: &mut BytesMut) -> Result<Option<Frame>> {
        // wait for a version frame, which is 9 bytes long
        // NOTE: future versions could be longer which will break this logic
        let buffer = Vec::from(&src[..]);
        log::debug!("decode unsync buffer: {:?}", buffer);

        if src.len() < 9 {
            log::debug!("decode unsync, waiting for enough bytes");
            Ok(None)
        } else {
            if src[0] == 0x07 && src[1] == 0xff && src[2] == 0x56 && checksum_ok(&src[0..9]) {
                log::debug!("decode unsync version frame");
                // received a version frame, now synced
                self.synchronized = true;
                src.advance(9);
                Ok(Some(Frame::Version))
            } else {
                // not a version frame, consume everything up to the next 0x07 (or the end)
                let mut index = 0;
                for v in src.iter() {
                    // if found, index points to item
                    // but skip instance in index 0 as it didn't match
                    if index != 0 && *v == 0x07 { break };

                    // if not found, index is 1 more than last index
                    index += 1;
                }

                if index < src.len() {
                    // found, discard items up to index
                    log::debug!("decode unsync discarded {} to next potential", index);
                    src.advance(index);
                } else {
                    log::debug!("decode unsync discarded entire buffer");
                    src.advance(src.len());
                }

                Ok(None)
            }
        }
    }
}

impl Decoder for VeMk3Decoder {
    type Item = Frame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        log::debug!("decode buffer is {} bytes", src.len());

        match self.synchronized {
            true => self.decode_synchronized(src),
            false => self.decode_unsynchronized(src)
        }
    }
}

fn checksum_ok(src: &[u8]) -> bool {
    let mut checksum: Wrapping<u8> = Wrapping(0);

    for v in src.iter() {
        checksum += Wrapping(*v);
    }

    checksum == Wrapping(0)
}
