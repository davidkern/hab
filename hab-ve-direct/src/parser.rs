use arrayvec::ArrayVec;
use anyhow::Result;
use std::num::Wrapping;
use std::str::from_utf8;
#[cfg(test)]
use mockall::{automock, mock, predicate::*};

// Constants defined in VE.Direct Protocol doc in "Implementation Guidelines" section.
const LABEL_LEN: usize = 9;
const VALUE_LEN: usize = 33;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Record {
    label: ArrayVec<u8, LABEL_LEN>,
    value: ArrayVec<u8, VALUE_LEN>,
}

impl Record {
    pub fn clear(&mut self) {
        self.label.clear();
        self.value.clear();
    }
}

#[cfg_attr(test, automock)]
pub trait ParseEvent {
    fn record(&mut self, label: &str, value: &str);
    fn checksum_valid(&mut self);
    fn checksum_invalid(&mut self);
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ParseState {
    #[default]
    Idle,
    RecordLabel,
    RecordValue,
    Checksum,
    RecordHex,
}

#[derive(Debug, Default)]
pub struct Parser
{
    pub state: ParseState,
    pub record: Record,
    pub checksum: Wrapping<u8>,
}

impl Parser
{
    pub fn parse<T: ParseEvent>(&mut self, parse_event: &mut T, inp: &[u8]) -> Result<()> {
        inp.iter().try_for_each(|b| self.parse_input_byte(parse_event, *b))
    }

    pub fn parse_input_byte<T: ParseEvent>(&mut self, parse_event: &mut T, inp: u8) -> Result<()> {
        const COLON: u8 = 0x3a; // ':'
        const NL: u8 = 0x0a; // '\n'
        const CR: u8 = 0x0d; // '\r'
        const TAB: u8 = 0x09; // '\t'
        const CHECKSUM_LABEL: &'static [u8] = "CHECKSUM".as_bytes();

        // adapted from reference implementation at
        // https://www.victronenergy.com/live/vedirect_protocol:faq
        if inp == COLON && self.state != ParseState::Checksum {
            self.state = ParseState::RecordHex;
        }

        if self.state != ParseState::RecordHex {
            self.checksum += inp;
        }

        let inp = to_upper(inp);

        match self.state {
            ParseState::Idle => {
                if inp == NL {
                    self.record.clear();
                    self.state = ParseState::RecordLabel;
                }
            }
            ParseState::RecordLabel => {
                if inp == TAB {
                    if self.record.label.as_slice() == CHECKSUM_LABEL {
                        self.state = ParseState::Checksum
                    } else {
                        self.state = ParseState::RecordValue
                    }
                } else {
                    self.record.label.try_push(inp)?;
                }
            }
            ParseState::RecordValue => {
                match inp {
                    NL => {
                        parse_event.record(from_utf8(self.record.label.as_slice())?, from_utf8(self.record.value.as_slice())?);
                        self.record.clear();
                        self.state = ParseState::RecordLabel;
                    }
                    CR => {
                        // skip
                    }
                    _ => {
                        self.record.value.try_push(inp)?;
                    }
                }
            }
            ParseState::Checksum => {
                self.state = ParseState::Idle;
                if self.checksum == Wrapping::default() {
                    parse_event.checksum_valid();
                } else {
                    parse_event.checksum_invalid();
                }
                self.checksum = Wrapping::default();
            }
            ParseState::RecordHex => {
                if inp == NL {
                    self.state = ParseState::Idle;
                }
                // ignore hex record data
            }
        }

        Ok(())
    }
}

pub fn to_upper(b: u8) -> u8 {
    if b >= 0x61 && b <= 0x7a {
        b - 0x20
    } else {
        b
    }
}

#[cfg(test)]
mod test {
    use super::{MockParseEvent, Parser, ParseState, to_upper};
    use mockall::{automock, mock, predicate::*};

    #[derive(Default)]
    struct Mock(MockParseEvent);

    impl Mock {
        fn expect_record(&mut self, label: &'static str, value: &'static str) {
            self.0
                .expect_record()
                .with(eq(label), eq(value))
                .returning(|_, _| {});
        }

        fn expect_checksum_valid(&mut self) {
            self.0
                .expect_checksum_valid()
                .returning(|| {});
        }

        fn expect_checksum_invalid(&mut self) {
            self.0
                .expect_checksum_invalid()
                .returning(|| {});
        }
    }

    #[test]
    fn test_parse_frame() {
        let data = b"\r\nERR\t0\r\nLOAD\tON\r\nRelay\tOFF\r\nH19\t29051\r\nH20\t725\r\nH21\t1376\r\nH22\t917\r\nH23\t1419\r\nHSDS\t191\r\nChecksum\t\xd2";

        let mut mock = Mock::default();
        mock.expect_record("ERR", "0");
        mock.expect_record("LOAD", "ON");
        mock.expect_record("RELAY", "OFF");
        mock.expect_record("H19", "29051");
        mock.expect_record("H20", "725");
        mock.expect_record("H21", "1376");
        mock.expect_record("H22", "917");
        mock.expect_record("H23", "1419");
        mock.expect_record("HSDS", "191");
        mock.expect_checksum_valid();

        let mut parser = Parser::default();
        parser.parse(&mut mock.0, data);
        assert_eq!(ParseState::Idle, parser.state);
    }

    #[test]
    fn test_parse_frame_with_hex() {
        let data = b":A200100ADB50200C6\n\r\nERR\t0\r\nLOAD\tON\r\nRelay\tOFF\r\nH19\t29067\r\nH20\t741\r\nH21\t1376\r\nH22\t917\r\nH23\t1419\r\nHSDS\t191\r\nChecksum\t\xcd";

        let mut mock = Mock::default();
        mock.expect_record("ERR", "0");
        mock.expect_record("LOAD", "ON");
        mock.expect_record("RELAY", "OFF");
        mock.expect_record("H19", "29067");
        mock.expect_record("H20", "741");
        mock.expect_record("H21", "1376");
        mock.expect_record("H22", "917");
        mock.expect_record("H23", "1419");
        mock.expect_record("HSDS", "191");
        mock.expect_checksum_valid();

        let mut parser = Parser::default();
        parser.parse(&mut mock.0, data);
        assert_eq!(ParseState::Idle, parser.state);
    }
}