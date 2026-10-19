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
    /// Protocol parameters are limited to 255 bytes. All command builders in
    /// this crate enforce smaller limits, so this method only asserts the
    /// protocol invariant in debug builds.
    pub fn encode(&self) -> Vec<u8> {
        debug_assert!(
            self.params.len() <= u8::MAX as usize,
            "packet parameters exceed 255 bytes"
        );
        let param_len = self.params.len() as u8;
        let body_len = param_len.saturating_add(2);

        let mut bytes = Vec::with_capacity(4 + self.params.len());
        bytes.push(PACKET_HEADER);
        bytes.push(body_len);
        bytes.push(self.opcode);
        bytes.push(param_len);
        bytes.extend_from_slice(&self.params);
        bytes
    }
}

/// Parses one or more blocks from a command-channel notification.
///
/// The first two bytes are the `0xFF` header and `body_len`. Blocks are then
/// read sequentially from offset 2. A truncated trailing block is discarded,
/// which matches the behavior of the reference implementation.
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
}
