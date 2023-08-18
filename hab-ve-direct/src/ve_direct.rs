//! Victron VE-Direct interface
use crate::config::Config;
use crate::parser::ParseEvent;
use anyhow::Result;
use bitflags::bitflags;
use futures_util::stream;
use influxdb2::models::DataPoint;
use serde::Serialize;
use serial_io::{build, AsyncSerial};
use std::fmt::Display;
use std::str;
use tokio::io::AsyncReadExt;

const BUFFER_SIZE: usize = 128;

pub async fn run(config: &Config) -> Result<()> {
    log::trace!("{}: starting VeDirectMppt", config.device_name);
    let builder = build(config.ve_direct_path.as_str(), 19200);
    let mut serial = AsyncSerial::from_builder(&builder)?;

    let db = influxdb2::Client::new(
        &config.influxdb_url,
        &config.influxdb_org,
        &config.influxdb_token,
    );

    let mut ve_direct_mppt = VeDirectMppt::new(&config.device_name);
    let mut parser = crate::parser::Parser::default();

    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    loop {
        // read from the device
        let count = serial.read(&mut buffer[..]).await?;

        // parse the read bytes
        parser.parse(&mut ve_direct_mppt, &buffer[0..count])?;

        // store decoded points
        if ve_direct_mppt.points.len() > 0 {
            let submission = ve_direct_mppt.points.clone();
            ve_direct_mppt.points.clear();
            if let Err(err) = db.write("hab", stream::iter(submission)).await {
                log::debug!("failed to write to influxdb: {:?}", err);
            }
        }
    }
}

#[derive(Debug)]
pub struct VeDirectMppt {
    // name of the device for these measurements
    device_name: String,

    // points ready to submit to the database
    points: Vec<DataPoint>,

    // current records
    records: Vec<(String, String)>,
}

impl VeDirectMppt {
    pub fn new(device_name: &str) -> Self {
        Self {
            device_name: device_name.to_string(),
            points: Default::default(),
            records: Default::default(),
        }
    }
}

impl ParseEvent for VeDirectMppt {
    fn record(&mut self, label: &str, value: &str) {
        self.records.push((label.to_string(), value.to_string()));
    }

    fn checksum_valid(&mut self) {
        log::info!("{:?}", self.records);

        let mut builder = DataPoint::builder(&self.device_name);
        for (label, value) in self.records.iter() {
            let (label, value) = (label.as_str(), value.as_str());
            match label {
                "V" => {
                    if let Ok(v) = u32::from_str_radix(value, 10) {
                        builder = builder.field("battery_voltage", v as f64 / 1000.0);
                    }
                }
                "VPV" => {
                    if let Ok(v) = u32::from_str_radix(value, 10) {
                        builder = builder.field("panel_voltage", v as f64 / 1000.0);
                    }
                }
                "PPV" => {
                    if let Ok(v) = u16::from_str_radix(value, 10) {
                        builder = builder.field("panel_power", v as f64);
                    }
                }
                "I" => {
                    if let Ok(v) = i32::from_str_radix(value, 10) {
                        builder = builder.field("battery_current", v as f64 / 1000.0);
                    }
                }
                "IL" => {
                    if let Ok(v) = i32::from_str_radix(value, 10) {
                        builder = builder.field("load_current", v as f64 / 1000.0);
                    }
                }
                "LOAD" => {
                    if value == "ON" {
                        builder = builder.field("load_state", true);
                    } else if value == "OFF" {
                        builder = builder.field("load_state", false);
                    }
                }
                "RELAY" => {
                    if value == "ON" {
                        builder = builder.field("relay_state", true);
                    } else if value == "OFF" {
                        builder = builder.field("relay_state", false);
                    }
                }
                "OR" => {
                    if let Ok(v) = u32::from_str_radix(&value[2..], 16) {
                        if let Some(or) = OffReason::from_bits(v) {
                            builder = builder.field("off_reason", or.to_string());
                        }
                    }
                }
                "H19" => {
                    if let Ok(v) = u32::from_str_radix(value, 10) {
                        builder = builder.field("yield_total", v as f64 * 10.0);
                    }
                }
                "H20" => {
                    if let Ok(v) = u16::from_str_radix(value, 10) {
                        builder = builder.field("yield_today", v as f64 * 10.0);
                    }
                }
                "H21" => {
                    if let Ok(v) = u16::from_str_radix(value, 10) {
                        builder = builder.field("maximum_power_today", v as f64);
                    }
                }
                "H22" => {
                    if let Ok(v) = u16::from_str_radix(value, 10) {
                        builder = builder.field("yield_yesterday", v as f64 * 10.0);
                    }
                }
                "H23" => {
                    if let Ok(v) = u16::from_str_radix(value, 10) {
                        builder = builder.field("maximum_power_yesterday", v as f64);
                    }
                }
                "ERR" => {
                    if let Ok(v) = u32::from_str_radix(value, 10) {
                        if let Some(err) = ErrorCode::from_u32(v) {
                            builder = builder.field("error", err.to_string());
                        }
                    }
                }
                "CS" => {
                    if let Ok(v) = u32::from_str_radix(value, 10) {
                        if let Some(cs) = StateOfOperation::from_u32(v) {
                            builder = builder.field("state", cs.to_string());
                        }
                    }
                }
                "FW" => {
                    builder = builder.field("firmware_version", value);
                }
                "FWE" => {
                    builder = builder.field("firmware_version_24", value);
                }
                "PID" => {
                    if let Ok(v) = u32::from_str_radix(&value[2..], 16) {
                        builder = builder.field("product_id", v as i64);
                    }
                }
                "SER#" => {
                    builder = builder.field("serial_number", value);
                }
                "HSDS" => {
                    if let Ok(v) = u16::from_str_radix(value, 10) {
                        builder = builder.field("day_number", v as i64);
                    }
                }
                "MPPT" => {
                    if let Ok(v) = u32::from_str_radix(value, 10) {
                        if let Some(mppt) = Mppt::from_u32(v) {
                            builder = builder.field("mppt_status", mppt.to_string());
                        }
                    }
                }
                unknown => {
                    log::warn!("Skipping unknown field {}", unknown);
                }
            }
        }

        match builder.build() {
            Ok(point) => {
                self.points.push(point);
            }
            Err(err) => {
                log::error!("failed to build datapoint: {:?}", err);
            }
        }

        self.records.clear();
    }

    fn checksum_invalid(&mut self) {
        log::warn!("BAD CHECKSUM: {:?}", self.records);
        self.records.clear();
    }
}

bitflags! {
    pub struct OffReason: u32 {
        const NONE = 0x0000_0000;
        const NO_INPUT_POWER = 0x0000_0001;
        const SWITCHED_OFF_POWER_SWITCH = 0x0000_0002;
        const SWITCHED_OFF_REGISTER = 0x0000_0004;
        const REMOTE_INPUT = 0x0000_0008;
        const PROTECTION_ACTIVE = 0x0000_0010;
        const PAYGO = 0x0000_0020;
        const BMS = 0x0000_0040;
        const ENGINE_SHUTDOWN_DETECTION = 0x0000_0080;
        const ANALYSING_INPUT_VOLTAGE = 0x0000_0100;
    }
}

impl Display for OffReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
pub enum StateOfOperation {
    Off,
    LowPower,
    Fault,
    Bulk,
    Absorption,
    Float,
    Storage,
    Equalize,
    Inverting,
    PowerSupply,
    StartingUp,
    RepeatedAbsorption,
    AutoEqualize,
    BatterySafe,
    ExternalControl,
}

impl Display for StateOfOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateOfOperation::Off => write!(f, "Off"),
            StateOfOperation::LowPower => write!(f, "Low Power"),
            StateOfOperation::Fault => write!(f, "Fault"),
            StateOfOperation::Bulk => write!(f, "Bulk"),
            StateOfOperation::Absorption => write!(f, "Absorption"),
            StateOfOperation::Float => write!(f, "Float"),
            StateOfOperation::Storage => write!(f, "Storage"),
            StateOfOperation::Equalize => write!(f, "Equalize"),
            StateOfOperation::Inverting => write!(f, "Inverting"),
            StateOfOperation::PowerSupply => write!(f, "Power Supply"),
            StateOfOperation::StartingUp => write!(f, "Starting Up"),
            StateOfOperation::RepeatedAbsorption => write!(f, "Repeated Absorption"),
            StateOfOperation::AutoEqualize => write!(f, "Auto Equalize"),
            StateOfOperation::BatterySafe => write!(f, "Battery Safe"),
            StateOfOperation::ExternalControl => write!(f, "External Control"),
        }
    }
}

impl StateOfOperation {
    fn from_u32(val: u32) -> Option<Self> {
        match val {
            0 => Some(StateOfOperation::Off),
            1 => Some(StateOfOperation::LowPower),
            2 => Some(StateOfOperation::Fault),
            3 => Some(StateOfOperation::Bulk),
            4 => Some(StateOfOperation::Absorption),
            5 => Some(StateOfOperation::Float),
            6 => Some(StateOfOperation::Storage),
            7 => Some(StateOfOperation::Equalize),
            9 => Some(StateOfOperation::Inverting),
            11 => Some(StateOfOperation::PowerSupply),
            245 => Some(StateOfOperation::StartingUp),
            246 => Some(StateOfOperation::RepeatedAbsorption),
            247 => Some(StateOfOperation::AutoEqualize),
            248 => Some(StateOfOperation::BatterySafe),
            252 => Some(StateOfOperation::ExternalControl),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
pub enum ErrorCode {
    NoError,
    BatteryVoltageHigh,
    ChargerTemperatureHigh,
    ChargerCurrentHigh,
    ChargerCurrentReversed,
    BulkTimeLimit,
    CurrentSensor,
    TerminalTemperatureHigh,
    Converter,
    InputVoltageHigh,
    InputCurrentHigh,
    InputShutdownDueToBatteryVoltage,
    InputShutdownDueToCurrentFlowWhileOff,
    LostCommunication,
    SynchronizedChargingConfiguration,
    BmsConnectionLost,
    NetworkMisconfigured,
    FactoryCalibrationDataLost,
    InvalidFirmware,
    InvalidUserSettings,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::NoError => write!(f, "No Error"),
            ErrorCode::BatteryVoltageHigh => write!(f, "Battery Voltage High"),
            ErrorCode::ChargerTemperatureHigh => write!(f, "Charger Temperature High"),
            ErrorCode::ChargerCurrentHigh => write!(f, "Charger Current High"),
            ErrorCode::ChargerCurrentReversed => write!(f, "Charger Current Reversed"),
            ErrorCode::BulkTimeLimit => write!(f, "Bulk Time Limit"),
            ErrorCode::CurrentSensor => write!(f, "Current Sensor"),
            ErrorCode::TerminalTemperatureHigh => write!(f, "Terminal Temperature High"),
            ErrorCode::Converter => write!(f, "Converter"),
            ErrorCode::InputVoltageHigh => write!(f, "Input Voltage High"),
            ErrorCode::InputCurrentHigh => write!(f, "Input Current High"),
            ErrorCode::InputShutdownDueToBatteryVoltage => {
                write!(f, "Input Shutdown Due To Battery Voltage")
            }
            ErrorCode::InputShutdownDueToCurrentFlowWhileOff => {
                write!(f, "Input Shutdown Due To Current Flow While Off")
            }
            ErrorCode::LostCommunication => write!(f, "Lost Communication"),
            ErrorCode::SynchronizedChargingConfiguration => {
                write!(f, "Synchronized Charging Configuration")
            }
            ErrorCode::BmsConnectionLost => write!(f, "Bms Connection Lost"),
            ErrorCode::NetworkMisconfigured => write!(f, "Network Misconfigured"),
            ErrorCode::FactoryCalibrationDataLost => write!(f, "Factory Calibration Data Lost"),
            ErrorCode::InvalidFirmware => write!(f, "Invalid Firmware"),
            ErrorCode::InvalidUserSettings => write!(f, "Invalid User Settings"),
        }
    }
}

impl ErrorCode {
    fn from_u32(val: u32) -> Option<ErrorCode> {
        match val {
            0 => Some(ErrorCode::NoError),
            2 => Some(ErrorCode::BatteryVoltageHigh),
            17 => Some(ErrorCode::ChargerTemperatureHigh),
            18 => Some(ErrorCode::ChargerCurrentHigh),
            19 => Some(ErrorCode::ChargerCurrentReversed),
            20 => Some(ErrorCode::BulkTimeLimit),
            21 => Some(ErrorCode::CurrentSensor),
            26 => Some(ErrorCode::TerminalTemperatureHigh),
            28 => Some(ErrorCode::Converter),
            33 => Some(ErrorCode::InputVoltageHigh),
            34 => Some(ErrorCode::InputCurrentHigh),
            38 => Some(ErrorCode::InputShutdownDueToBatteryVoltage),
            39 => Some(ErrorCode::InputShutdownDueToCurrentFlowWhileOff),
            65 => Some(ErrorCode::LostCommunication),
            66 => Some(ErrorCode::SynchronizedChargingConfiguration),
            67 => Some(ErrorCode::BmsConnectionLost),
            68 => Some(ErrorCode::NetworkMisconfigured),
            116 => Some(ErrorCode::FactoryCalibrationDataLost),
            117 => Some(ErrorCode::InvalidFirmware),
            119 => Some(ErrorCode::InvalidUserSettings),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
pub enum Mppt {
    Off = 0,
    VoltageOrCurrentLimited = 1,
    MpptTrackerActive = 2,
}

impl Mppt {
    fn from_u32(val: u32) -> Option<Mppt> {
        match val {
            0 => Some(Mppt::Off),
            1 => Some(Mppt::VoltageOrCurrentLimited),
            2 => Some(Mppt::MpptTrackerActive),
            _ => None,
        }
    }
}

impl Display for Mppt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mppt::Off => {
                write!(f, "Off")
            }
            Mppt::VoltageOrCurrentLimited => {
                write!(f, "Voltage Or Current Limited")
            }
            Mppt::MpptTrackerActive => {
                write!(f, "Mppt Tracker Active")
            }
        }
    }
}
