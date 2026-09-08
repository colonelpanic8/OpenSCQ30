use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct State {
    pub host: u8,
    pub tws_connected: bool,
    pub battery_left: u8,
    pub battery_right: u8,
    pub battery_case: Option<u8>,
    pub firmware_left: String,
    pub firmware_right: String,
    pub firmware_case: Option<String>,
    pub serial: String,
    pub buttons: [Option<[u8; 2]>; 12],
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(super) enum ParseError {
    #[error("truncated D1203 state tag")]
    Truncated,
    #[error("duplicate D1203 state tag {0}")]
    Duplicate(u8),
    #[error("missing or invalid D1203 state tag {0}")]
    Invalid(u8),
    #[error("state packet does not identify a D1203 device")]
    WrongModel,
}

impl State {
    pub fn parse(mut input: &[u8]) -> Result<Self, ParseError> {
        let mut fields = BTreeMap::new();
        while !input.is_empty() {
            if input.len() < 2 {
                return Err(ParseError::Truncated);
            }
            let tag = input[0];
            let length = usize::from(input[1]);
            let value = input.get(2..2 + length).ok_or(ParseError::Truncated)?;
            if fields.insert(tag, value).is_some() {
                return Err(ParseError::Duplicate(tag));
            }
            input = &input[2 + length..];
        }
        let field = |tag, length| -> Result<&[u8], ParseError> {
            fields
                .get(&tag)
                .copied()
                .filter(|it| it.len() == length)
                .ok_or(ParseError::Invalid(tag))
        };
        let text = |tag, length| -> Result<String, ParseError> {
            let bytes = field(tag, length)?;
            let value = std::str::from_utf8(bytes).map_err(|_| ParseError::Invalid(tag))?;
            if !value
                .trim_end_matches('\0')
                .bytes()
                .all(|byte| byte.is_ascii_graphic())
            {
                return Err(ParseError::Invalid(tag));
            }
            Ok(value.trim_end_matches('\0').to_owned())
        };
        let battery = |tag| -> Result<u8, ParseError> {
            let level = field(tag, 2)?[1];
            if level > 100 {
                return Err(ParseError::Invalid(tag));
            }
            Ok(level)
        };
        let host = field(1, 1)?[0];
        let tws = field(2, 1)?[0];
        if host > 1 {
            return Err(ParseError::Invalid(1));
        }
        if tws > 1 {
            return Err(ParseError::Invalid(2));
        }
        let serial = text(7, 17)?;
        if serial.len() != 16 || !serial.starts_with("1203") {
            return Err(ParseError::WrongModel);
        }
        let mut buttons = [None; 12];
        for (index, button) in buttons.iter_mut().enumerate() {
            let tag = 13 + index as u8;
            if fields.contains_key(&tag) {
                *button = Some(field(tag, 2)?.try_into().unwrap());
            }
        }
        Ok(Self {
            host,
            tws_connected: tws == 1,
            battery_left: battery(3)?,
            battery_right: battery(4)?,
            battery_case: fields.contains_key(&8).then(|| battery(8)).transpose()?,
            firmware_left: text(5, 5)?,
            firmware_right: text(6, 5)?,
            firmware_case: fields.contains_key(&9).then(|| text(9, 5)).transpose()?,
            serial,
            buttons,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Public capture: https://github.com/Oppzippy/OpenSCQ30/issues/342 (identifiers redacted).
    const CAPTURE: &[u8] = include_bytes!("state.bin");

    #[test]
    fn decodes_real_d1203_state() {
        let state = State::parse(CAPTURE).unwrap();
        assert_eq!((state.host, state.tws_connected), (0, true));
        assert_eq!(
            (state.battery_left, state.battery_right, state.battery_case),
            (60, 58, Some(99))
        );
        assert_eq!(
            (state.firmware_left.as_str(), state.firmware_right.as_str()),
            ("05.33", "05.33")
        );
        assert_eq!(state.firmware_case.as_deref(), Some("01.46"));
        assert_eq!(state.buttons[2], Some([3, 6]));
        assert_eq!(state.buttons[6], Some([4, 4]));
    }

    #[test]
    fn skips_unknown_tags_without_shifting_fields() {
        let mut extended = vec![250, 3, 9, 8, 7];
        extended.extend_from_slice(CAPTURE);
        assert_eq!(State::parse(&extended), State::parse(CAPTURE));
    }

    #[test]
    fn rejects_truncation_and_duplicate_tags() {
        let mut bytes = CAPTURE.to_vec();
        bytes.push(250);
        assert_eq!(State::parse(&bytes), Err(ParseError::Truncated));
        bytes.push(2);
        bytes.push(0);
        assert_eq!(State::parse(&bytes), Err(ParseError::Truncated));
        bytes.truncate(CAPTURE.len());
        bytes.extend_from_slice(&[1, 1, 0]);
        assert_eq!(State::parse(&bytes), Err(ParseError::Duplicate(1)));
    }

    #[test]
    fn rejects_wrong_model_and_invalid_battery() {
        let mut bytes = CAPTURE.to_vec();
        let serial = bytes.windows(4).position(|it| it == b"1203").unwrap();
        bytes[serial + 3] = b'4';
        assert_eq!(State::parse(&bytes), Err(ParseError::WrongModel));
        let mut bytes = CAPTURE.to_vec();
        bytes[9] = 101;
        assert_eq!(State::parse(&bytes), Err(ParseError::Invalid(3)));
    }
}
