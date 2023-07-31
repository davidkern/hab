use anyhow::Result;
use crate::config::Config;
use bytes::{Buf, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder, Framed};
use tokio_stream::StreamExt;
use std::num::Wrapping;
use futures_util::{sink::SinkExt, stream};
use influxdb2::models::DataPoint;

/// Store data from mk3 device into influxdb
pub async fn run(config: &Config) -> Result<()> {
    let path = &config.mk3_path;
    let builder = serial_io::build(path, 2400);
    let serial = serial_io::AsyncSerial::from_builder(&builder)?;

    let codec = VeMk3Codec::default();
    let mut mk3 = Framed::new(serial, codec);
    mk3.send(RequestFrame::Version).await?;

    let db = influxdb2::Client::new(&config.influxdb_url, &config.influxdb_org, &config.influxdb_token);
    
    while let Some(result) = mk3.next().await {
        match result {
            Ok(frame) => {
                log::debug!("frame: {}", frame);
                match frame {
                    Frame::Version => {
                        // request status on each version frame
                        //mk3.send(RequestFrame::LedStatus).await?;
                        mk3.send(RequestFrame::DcStatus).await?;
                        mk3.send(RequestFrame::AcL1Status).await?;
                    }
                    Frame::LedStatus { led_status } => {
                        match DataPoint::builder("multiplus")
                            .field("mains", led_status.mains)
                            .field("absorption", led_status.absorption)
                            .field("bulk", led_status.bulk)
                            .field("float", led_status.float)
                            .field("inverter", led_status.inverter)
                            .field("overload", led_status.overload)
                            .field("low_battery", led_status.low_battery)
                            .field("temperature", led_status.temperature)
                            .build() {
                                Ok(point) => {
                                    let points = vec![point];
                                    if let Err(err) = db.write("hab", stream::iter(points)).await {
                                        log::debug!("failed to write led_status: {:?}", err);
                                    }
                                }
    
                                Err(err) => {
                                    log::debug!("failed to build led_status point: {:?}", err);
                                }    
                            }
                    }
                    Frame::Ac { ac } => {
                        let state = match ac.state {
                            AcState::Down => "down",
                            AcState::Startup => "startup",
                            AcState::Off => "off",
                            AcState::Slave => "slave",
                            AcState::InvertFull => "invert-full",
                            AcState::InvertHalf => "invert-half",
                            AcState::InvertAes => "invert-aes",
                            AcState::PowerAssist => "power-assist",
                            AcState::Bypass => "bypass",
                            AcState::Charge => "charge",
                            AcState::Unknown => "unknown",                        
                        };

                        match DataPoint::builder("ac")
                            .field("bf_factor", ac.bf_factor as f64)
                            .field("inverter_factor", ac.inverter_factor as f64)
                            .field("state", state)
                            .field("mains_voltage", ac.mains_voltage as f64)
                            .field("mains_current", ac.mains_current as f64)
                            .field("mains_watts", ac.mains_watts as f64)
                            .field("inverter_voltage", ac.inverter_voltage as f64)
                            .field("inverter_current", ac.inverter_current as f64)
                            .field("inverter_watts", ac.inverter_watts as f64)
                            .field("mains_frequency", ac.mains_frequency as f64)
                            .build() {
                                Ok(point) => {
                                    let points = vec![point];
                                    if let Err(err) = db.write("hab", stream::iter(points)).await {
                                        log::debug!("failed to write ac: {:?}", err);
                                    }
                                }
    
                                Err(err) => {
                                    log::debug!("failed to build ac point: {:?}", err);
                                }    
                            }
                    }
                    Frame::Dc { dc } => {
                        match DataPoint::builder("dc")
                            .field("voltage", dc.voltage as f64)
                            .field("inverter_current", dc.inverter_current as f64)
                            .field("inverter_watts", dc.inverter_watts as f64)
                            .field("charger_current", dc.charger_current as f64)
                            .field("charger_watts", dc.charger_watts as f64)
                            .field("inverter_frequency", dc.inverter_frequency as f64)
                            .build() {

                            Ok(point) => {
                                let points = vec![point];
                                if let Err(err) = db.write("hab", stream::iter(points)).await {
                                    log::debug!("failed to write dc: {:?}", err);
                                }
                            }

                            Err(err) => {
                                log::debug!("failed to build dc point: {:?}", err);
                            }
                        }

                    }
                    _ => {}
                }
            }
            Err(e) => {
                log::error!("error: {}", e);
            }
        }
    }
    Ok(())
}

pub struct VeMk3Codec {
    synchronized: bool,
}

impl Default for VeMk3Codec {
    fn default() -> Self {
        Self {
            synchronized: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Frame {
    Unknown,
    Version,
    LedStatus { led_status: LedStatus },
    Ac { ac: AcMeasurement },
    Dc { dc: DcMeasurement },
}

#[derive(Clone, Debug)]
pub struct LedStatus {
    mains: bool,
    absorption: bool,
    bulk: bool,
    float: bool,
    inverter: bool,
    overload: bool,
    low_battery: bool,
    temperature: bool,
}

#[derive(Clone, Debug)]
pub struct DcMeasurement {
    voltage: f32,
    inverter_current: f32,
    inverter_watts: f32,
    charger_current: f32,
    charger_watts: f32,
    inverter_frequency: f32,
}

#[derive(Clone, Debug)]
pub enum AcState {
    Down,
    Startup,
    Off,
    Slave,
    InvertFull,
    InvertHalf,
    InvertAes,
    PowerAssist,
    Bypass,
    Charge,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct AcMeasurement {
    bf_factor: u8,
    inverter_factor: u8,
    state: AcState,
    mains_voltage: f32,
    mains_current: f32,
    mains_watts: f32,
    inverter_voltage: f32,
    inverter_current: f32,
    inverter_watts: f32,
    mains_frequency: f32,
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => { write!(f, "unknown") }
            Self::Version => { write!(f, "version") }
            Self::LedStatus {
                led_status
            } => { write!(f, "led: {:?}", led_status) }
            Self::Dc {
                dc
            } => { write!(f, "dc: {:?}", dc) }
            Self::Ac {
                ac
            } => { write!(f, "ac: {:?}", ac) }
        }
    }
}

impl VeMk3Codec {
    fn decode_synchronized(&mut self, src: &mut BytesMut) -> Result<Option<Frame>> {
        let buffer = Vec::from(&src[..]);
        log::trace!("decode sync buffer: {:?}", buffer);

        if src.len() < 1 {
            log::trace!("decode sync, waiting for length byte");
            Ok(None)
        } else {
            let expected_len: usize = <u8 as Into<usize>>::into(src[0]) + 2;

            if src.len() < expected_len {
                log::trace!("decode sync, waiting for expected length");
                Ok(None)
            } else {
                let result = {
                    if src[1] == 0xff && src[2] == 0x56 {
                        // version frame
                        Ok(Some(Frame::Version))
                    } else if src[1] == 0xff && src[2] == 0x4c {
                        // led frame
                        let active = src[3] | src[4]; // either on, or blinking

                        Ok(Some(Frame::LedStatus {
                            led_status: LedStatus {
                                mains: active & 0x01 != 0,
                                absorption: active & 0x02 != 0,
                                bulk: active & 0x04 != 0,
                                float: active & 0x08 != 0,
                                inverter: active & 0x10 != 0,
                                overload: active & 0x20 != 0,
                                low_battery: active & 0x40 != 0,
                                temperature: active & 0x80 != 0,    
                            }
                        }))
                    } else if src[1] == 0x20 {
                        Ok(Some(decode_info_frame(&src[2..16])))
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
        log::trace!("decode unsync buffer: {:?}", buffer);

        if src.len() < 9 {
            log::trace!("decode unsync, waiting for enough bytes");
            Ok(None)
        } else {
            if src[0] == 0x07 && src[1] == 0xff && src[2] == 0x56 && checksum_ok(&src[0..9]) {
                log::trace!("decode unsync version frame");
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
                    log::trace!("decode unsync discarded {} to next potential", index);
                    src.advance(index);
                } else {
                    log::trace!("decode unsync discarded entire buffer");
                    src.advance(src.len());
                }

                Ok(None)
            }
        }
    }
}

fn decode_info_frame(d: &[u8]) -> Frame {
    let phase_info = d[4];
    if phase_info == 0x0c {
        // DC
        let voltage = ((d[6] as f32) * 256.0 + (d[5] as f32)) / 100.0;
        let inverter_current = ((d[9] as f32) * 65536.0 + (d[8] as f32) * 256.0 + (d[7] as f32)) / 10.0;
        let charger_current = ((d[12] as f32) * 65536.0 + (d[11] as f32) * 256.0 + (d[10] as f32)) / 10.0;

        Frame::Dc {
            dc: DcMeasurement {
                voltage,
                inverter_current,
                inverter_watts: voltage * inverter_current,
                charger_current,
                charger_watts: voltage * charger_current,
                inverter_frequency: 10000.0 / (d[13] as f32),            
            }
        }
    } else if phase_info >= 0x05 && phase_info <= 0x0b {
        // AC
        let state = match d[3] {
            0x00 => AcState::Down,
            0x01 => AcState::Startup,
            0x02 => AcState::Off,
            0x03 => AcState::Slave,
            0x04 => AcState::InvertFull,
            0x05 => AcState::InvertHalf,
            0x06 => AcState::InvertAes,
            0x07 => AcState::PowerAssist,
            0x08 => AcState::Bypass,
            0x09 => AcState::Charge,
            _ => AcState::Unknown,
        };

        let mains_voltage = ((d[6] as f32) * 256.0 + (d[5] as f32)) / 100.0;
        let mains_current = ((d[8] as f32) * 256.0 + (d[7] as f32)) / 100.0;
        let inverter_voltage = ((d[10] as f32) * 256.0 + (d[9] as f32)) / 100.0;
        let inverter_current = ((d[12] as f32) * 256.0 + (d[11] as f32)) / 100.0;

        Frame::Ac {
            ac: AcMeasurement {
                bf_factor: d[0],
                inverter_factor: d[1],
                state,
                mains_voltage,
                mains_current,
                mains_watts: mains_voltage * mains_current,
                inverter_voltage,
                inverter_current,
                inverter_watts: inverter_voltage * inverter_current,
                mains_frequency: 10000.0 / (d[13] as f32),
            }
        }
    } else {
        Frame::Unknown
    }
}

impl Decoder for VeMk3Codec {
    type Item = Frame;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        log::trace!("decode buffer is {} bytes", src.len());

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

// fn checksum(src: &[u8]) -> u8 {
//     let mut checksum: Wrapping<u8> = Wrapping(0);

//     for v in src.iter() {
//         checksum -= Wrapping(*v);
//     }

//     checksum.0
// }

#[derive(Clone, Debug)]
pub enum RequestFrame {
    Version,
    LedStatus,
    DcStatus,
    AcL1Status,
}

impl Encoder<RequestFrame> for VeMk3Codec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RequestFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            RequestFrame::Version => {
                let request = [0x02, 0xff, 0x56, 0xa9];
                dst.reserve(request.len());
                dst.extend_from_slice(&request);
            }
            RequestFrame::LedStatus => {
                // request led status
                let request = [0x02, 0xff, 0x4c, 0xb3];
                dst.reserve(request.len());
                dst.extend_from_slice(&request);
            },
            RequestFrame::DcStatus => {
                // request DC status
                let request = [0x03, 0xff, 0x46, 0x00, 0xb8];
                dst.reserve(request.len());
                dst.extend_from_slice(&request);                
            }
            RequestFrame::AcL1Status => {
                // request DC status
                let request = [0x03, 0xff, 0x46, 0x01, 0xb7];
                dst.reserve(request.len());
                dst.extend_from_slice(&request);                
            }
        }

        Ok(())
    }
}
