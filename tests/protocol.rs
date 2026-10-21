use melobudsnext::hex;
use melobudsnext::protocol::anc::{AncMode, AncScene, TransparencyMode};
use melobudsnext::protocol::commands::{self, NotificationVolume, WearDetection};
use melobudsnext::protocol::opcode::Opcode;
use melobudsnext::protocol::readings::{Battery, FirmwareVersion};
use melobudsnext::protocol::touch::{TouchFunction, TouchKey, TouchMapping};
use melobudsnext::DeviceState;

fn bytes(input: &str) -> Vec<u8> {
    hex::parse(input).unwrap()
}

#[test]
fn builds_all_documented_packet_examples() {
    let examples = [
        (commands::game_mode(true), "FF 03 09 01 01"),
        (commands::game_mode(false), "FF 03 09 01 02"),
        (commands::query(Opcode::Anc), "FF 03 FE 01 17"),
        (commands::anc(AncMode::Off), "FF 05 17 03 00 00 00"),
        (
            commands::anc(AncMode::anc(AncScene::Indoor, Some(2)).unwrap()),
            "FF 05 17 03 01 01 01",
        ),
        (
            commands::anc(AncMode::transparency(TransparencyMode::Intensity(4)).unwrap()),
            "FF 05 17 03 03 01 04",
        ),
        (
            commands::anc(AncMode::transparency(TransparencyMode::VoiceEnhancement).unwrap()),
            "FF 05 17 03 03 01 00",
        ),
        (commands::rename("Fone").unwrap(), "FF 06 18 04 46 6F 6E 65"),
        (commands::channel_balance(60), "FF 03 16 01 3C"),
        (
            commands::notification_volume(NotificationVolume::Medium),
            "FF 04 1D 02 06 06",
        ),
        (
            commands::wear_detection(WearDetection {
                enabled: true,
                music_index: 1,
                keep_anc: true,
                tone: 0,
            }),
            "FF 06 2C 04 01 01 01 00",
        ),
    ];

    for (packet, expected) in examples {
        assert_eq!(hex::format(&packet.encode()), expected);
    }
}

#[test]
fn encodes_and_decodes_the_touch_example() {
    let encoded = bytes("01 01 02 03 03 00 04 00 05 00 06 00 07 00 08 00 00 00 00 00");

    let mapping = TouchMapping::parse(&encoded).unwrap();
    assert_eq!(mapping.get(TouchKey::LeftSingle), TouchFunction::PlayPause);
    assert_eq!(mapping.get(TouchKey::RightSingle), TouchFunction::Next);

    assert_eq!(mapping.encode().to_vec(), encoded);
}

#[test]
fn decodes_battery_and_firmware_readings() {
    let battery = Battery::parse(&bytes("D0 46 00")).unwrap();
    assert_eq!(battery.left.percent(), 80);
    assert!(battery.left.charging());
    assert_eq!(battery.right.percent(), 70);

    let short = Battery::parse(&bytes("50 46")).unwrap();
    assert_eq!(short.left.percent(), 80);
    assert_eq!(short.right.percent(), 70);

    let firmware = FirmwareVersion::parse(&bytes("02 00 06")).unwrap();
    assert_eq!(firmware.to_string(), "2.0.6");
}

#[test]
fn tolerates_firmware_variations_reported_by_the_python_client() {
    let mut state = DeviceState::default();
    state
        .apply_bytes(&bytes("FF 05 17 03 01 09 00"))
        .expect("unknown ANC triple");
    assert_eq!(state.anc, AncMode::Unknown([0x01, 0x09, 0x00]));

    let touch =
        TouchMapping::parse(&bytes("01 01 02 03 03 00 04 00 05 00 06 00 07 00 08 00")).unwrap();
    assert_eq!(touch.get(TouchKey::RightSingle), TouchFunction::Next);

    let mut renamed = DeviceState::default();
    renamed
        .apply_bytes(&bytes("FF 06 18 04 46 6F 6E 65"))
        .unwrap();
    renamed.apply_bytes(&bytes("FF 04 18 02 00 00")).unwrap();
    assert_eq!(renamed.name.as_deref(), Some("Fone"));
}

#[test]
fn applies_a_sequence_of_notifications_to_the_state() {
    let mut state = DeviceState::default();

    state
        .apply_bytes(&bytes("FF 03 09 01 01"))
        .expect("game mode notification");
    state
        .apply_bytes(&bytes("FF 05 17 03 01 02 02"))
        .expect("anc notification");
    state
        .apply_bytes(&bytes("FF 06 18 04 46 6F 6E 65"))
        .expect("rename notification");
    state
        .apply_bytes(&bytes("FF 03 16 01 3C"))
        .expect("balance notification");
    state
        .apply_bytes(&bytes("FF 04 1D 02 06 06"))
        .expect("notification volume");

    assert!(state.game_mode);
    assert_eq!(state.anc, AncMode::anc(AncScene::Daily, Some(3)).unwrap());
    assert_eq!(state.name.as_deref(), Some("Fone"));
    assert_eq!(state.channel_balance, 60);
    assert_eq!(state.notification_volume, Some(0x06));
}

#[test]
fn parses_composite_notifications() {
    let mut state = DeviceState::default();
    let applied = state
        .apply_bytes(&bytes("FF 06 09 01 01 17 03 00 00 00"))
        .unwrap();

    assert_eq!(applied, 2);
    assert!(state.game_mode);
    assert_eq!(state.anc, AncMode::Off);
}
