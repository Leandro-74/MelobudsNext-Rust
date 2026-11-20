use crate::error::ProtocolError;

/// Fixed header byte used by every command-channel packet.
pub const PACKET_HEADER: u8 = 0xFF;

/// A single command-channel block: opcode followed by its parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub opcode: u8,
    pub params: Vec<u8>,
}

impl Packet {
    pub fn new(opcode: impl Into<u8>, params: impl Into<Vec<u8>>) -> Self {
        Self {
            opcode: opcode.into(),
            params: params.into(),
        }
    }

    /// Encodes the packet as `FF body_len opcode param_len params...`.
    ///
    /// `body_len` includes the opcode and the parameter length, so the
    /// protocol allows at most 253 parameter bytes. This variant reports the
    /// overflow instead of truncating the length byte.
    pub fn try_encode(&self) -> Result<Vec<u8>, ProtocolError> {
        if self.params.len() > 253 {
            return Err(ProtocolError::ParamsTooLong(self.params.len()));
        }

        let param_len = self.params.len() as u8;
        let body_len = param_len + 2;

        let mut bytes = Vec::with_capacity(4 + self.params.len());
        bytes.push(PACKET_HEADER);
        bytes.push(body_len);
        bytes.push(self.opcode);
        bytes.push(param_len);
        bytes.extend_from_slice(&self.params);
        Ok(bytes)
    }

    /// Convenience wrapper around [`Packet::try_encode`] for packets whose
    /// parameters are already validated.
    ///
    /// # Panics
    ///
    /// Panics when the parameters exceed the 253-byte protocol limit. Prefer
    /// [`Packet::try_encode`] for values that come from outside the crate.
    pub fn encode(&self) -> Vec<u8> {
        self.try_encode()
            .expect("packet parameters exceed the 253-byte protocol limit")
    }
}

/// Parses one or more blocks from a command-channel notification.
///
/// The first two bytes are the `0xFF` header and `body_len`. Blocks are then
/// read sequentially from offset 2. `body_len` is intentionally ignored because
/// composite packets do not always report the total body length. A truncated
/// trailing block is discarded, and extra bytes that do not form a complete
/// block are ignored, which matches the behavior of the reference
/// implementation.
pub fn parse(bytes: &[u8]) -> Result<Vec<Packet>, ProtocolError> {
    if bytes.is_empty() {
        return Err(ProtocolError::EmptyPacket);
    }
    if bytes.len() < 2 {
        return Err(ProtocolError::TruncatedHeader);
    }
    if bytes[0] != PACKET_HEADER {
        return Err(ProtocolError::InvalidHeader(bytes[0]));
    }

    let mut packets = Vec::new();
    let mut offset = 2;

    while offset + 2 <= bytes.len() {
        let opcode = bytes[offset];
        let param_len = bytes[offset + 1] as usize;
        let params_start = offset + 2;

        if params_start + param_len > bytes.len() {
            break;
        }

        packets.push(Packet {
            opcode,
            params: bytes[params_start..params_start + param_len].to_vec(),
        });
        offset = params_start + param_len;
    }

    Ok(packets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_single_packet() {
        let packets = parse(&[0xFF, 0x03, 0x09, 0x01, 0x01]).unwrap();
        assert_eq!(packets, vec![Packet::new(0x09, [0x01])]);
    }

    #[test]
    fn parses_composite_packets() {
        let bytes = [0xFF, 0x06, 0x09, 0x01, 0x01, 0x17, 0x03, 0x00, 0x00, 0x00];
        let packets = parse(&bytes).unwrap();
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0], Packet::new(0x09, [0x01]));
        assert_eq!(packets[1], Packet::new(0x17, [0x00, 0x00, 0x00]));
    }

    #[test]
    fn discards_a_truncated_block() {
        assert!(parse(&[0xFF, 0x05, 0x17, 0x03, 0x00]).unwrap().is_empty());
    }

    #[test]
    fn keeps_complete_blocks_before_a_truncated_one() {
        let bytes = [0xFF, 0x06, 0x09, 0x01, 0x01, 0x17, 0x03, 0x00];

        assert_eq!(parse(&bytes).unwrap(), vec![Packet::new(0x09, [0x01])]);
    }

    #[test]
    fn ignores_extra_bytes_after_the_last_complete_block() {
        let bytes = [0xFF, 0x03, 0x09, 0x01, 0x01, 0xAA, 0xBB];

        assert_eq!(parse(&bytes).unwrap(), vec![Packet::new(0x09, [0x01])]);
    }

    #[test]
    fn discards_a_trailing_single_byte() {
        let bytes = [0xFF, 0x03, 0x09, 0x01, 0x01, 0xAA];

        assert_eq!(parse(&bytes).unwrap(), vec![Packet::new(0x09, [0x01])]);
    }

    #[test]
    fn ignores_body_len_when_parsing_blocks() {
        // body_len is 0x00, but the two blocks after it are complete.
        let bytes = [0xFF, 0x00, 0x09, 0x01, 0x01, 0x17, 0x03, 0x00, 0x00, 0x00];
        let packets = parse(&bytes).unwrap();

        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0], Packet::new(0x09, [0x01]));
        assert_eq!(packets[1], Packet::new(0x17, [0x00, 0x00, 0x00]));
    }

    #[test]
    fn parses_three_complete_blocks() {
        let bytes = [
            0xFF, 0x09, 0x09, 0x01, 0x01, 0x16, 0x01, 0x3C, 0x23, 0x01, 0x01,
        ];
        let packets = parse(&bytes).unwrap();

        assert_eq!(
            packets,
            vec![
                Packet::new(0x09, [0x01]),
                Packet::new(0x16, [0x3C]),
                Packet::new(0x23, [0x01]),
            ]
        );
    }

    #[test]
    fn accepts_a_header_without_blocks() {
        assert!(parse(&[0xFF, 0x00]).unwrap().is_empty());
    }

    #[test]
    fn rejects_invalid_headers() {
        assert_eq!(parse(&[]).unwrap_err(), ProtocolError::EmptyPacket);
        assert_eq!(parse(&[0xFF]).unwrap_err(), ProtocolError::TruncatedHeader);
        assert_eq!(
            parse(&[0x00, 0x03, 0x09, 0x01, 0x01]).unwrap_err(),
            ProtocolError::InvalidHeader(0x00)
        );
    }

    #[test]
    fn encodes_the_documented_example() {
        assert_eq!(
            Packet::new(0x09, [0x01]).encode(),
            [0xFF, 0x03, 0x09, 0x01, 0x01]
        );
    }

    #[test]
    #[should_panic(expected = "253-byte protocol limit")]
    fn rejects_oversized_parameters_instead_of_truncating() {
        Packet::new(0x42, vec![0u8; 254]).encode();
    }

    #[test]
    fn try_encode_reports_oversized_parameters() {
        let packet = Packet::new(0x42, vec![0u8; 254]);
        assert_eq!(packet.try_encode(), Err(ProtocolError::ParamsTooLong(254)));
    }

    #[test]
    fn try_encode_accepts_the_maximum_payload() {
        let packet = Packet::new(0x42, vec![0u8; 253]);
        let bytes = packet.try_encode().unwrap();

        assert_eq!(bytes[0], PACKET_HEADER);
        assert_eq!(bytes[1], 0xFF);
        assert_eq!(bytes[2], 0x42);
        assert_eq!(bytes[3], 253);
        assert_eq!(bytes.len(), 257);
    }
}
