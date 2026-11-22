//! Tests for the application state, update function and regressions.

use melobudsnext::config::Config;

use crate::app::{update, App, Message, OFFLINE_NOTICE};
use crate::ble_task::ActionResult;
use crate::options::{AncChoice, Tab, TouchOption, VolumeOption};
use crate::view::view;

fn offline_app() -> App {
    let (app, _) = App::build(Config::default(), None, true);
    app
}

fn online_app() -> App {
    let (app, _) = App::build(Config::default(), None, false);
    app
}

#[test]
fn shows_unknown_touch_functions_in_the_selector() {
    use melobudsnext::protocol::touch::TouchFunction;

    let function = TouchFunction::Unknown(0x42);
    let options = TouchOption::options(function);

    assert_eq!(options.len(), TouchOption::ALL.len() + 1);
    let unknown = options
        .iter()
        .find(|option| option.0 == function)
        .expect("unknown function must be selectable");
    assert_eq!(unknown.to_string(), "Desconhecida (0x42)");

    // Known values must not grow the list.
    assert_eq!(
        TouchOption::options(TouchFunction::Next).len(),
        TouchOption::ALL.len()
    );
}

#[test]
fn unknown_anc_is_preserved_and_never_applied_as_off() {
    use melobudsnext::protocol::anc::AncMode;

    let mut app = online_app();
    app.state.anc = AncMode::Unknown([0x01, 0x09, 0x00]);

    app.load_drafts();

    assert_eq!(app.anc_choice, AncChoice::Unknown([0x01, 0x09, 0x00]));
    let error = app.active_anc_mode().unwrap_err();
    assert!(error.contains("01 09 00"), "error was: {error}");
}

#[test]
fn offline_mode_blocks_every_device_action() {
    use melobudsnext::protocol::commands::NotificationVolume;

    let actions = [
        Message::Connect,
        Message::Refresh,
        Message::Disconnect,
        Message::ApplyAnc,
        Message::ToggleGame(true),
        Message::ToggleSleep(false),
        Message::ToggleLdac(true),
        Message::ApplyBalance,
        Message::VolumeSelected(VolumeOption(NotificationVolume::Medium)),
        Message::ApplyTouch,
        Message::ApplyWear,
        Message::ApplyRename,
    ];

    for action in actions {
        let mut app = offline_app();
        app.notice = None;

        let _ = update(&mut app, action);

        assert!(app.busy.is_none(), "offline action started a BLE task");
        assert!(app.error.is_none(), "offline action produced an error");
        assert_eq!(app.notice.as_deref(), Some(OFFLINE_NOTICE));
    }
}

#[test]
fn offline_mode_does_not_change_toggles_optimistically() {
    let mut app = offline_app();
    app.state.game_mode = false;
    app.state.ldac = false;

    let _ = update(&mut app, Message::ToggleGame(true));
    let _ = update(&mut app, Message::ToggleLdac(true));

    assert!(!app.state.game_mode);
    assert!(!app.state.ldac);
}

#[test]
fn offline_mode_still_allows_local_drafts() {
    let mut app = offline_app();
    let _ = update(&mut app, Message::Tab(Tab::Audio));
    let _ = update(&mut app, Message::BalanceChanged(30));

    assert_eq!(app.tab, Tab::Audio);
    assert_eq!(app.balance, 30);
}

// -----------------------------------------------------------------------
// TST-03 — transições de estado da GUI
// -----------------------------------------------------------------------

#[test]
fn finish_applies_the_session_result_and_reloads_drafts() {
    use melobudsnext::protocol::anc::{AncMode, AncScene};
    use melobudsnext::protocol::touch::{TouchFunction, TouchKey, TouchMapping};
    use melobudsnext::DeviceState;

    let mut touch = TouchMapping::new();
    touch.set(TouchKey::LeftSingle, TouchFunction::PlayPause);
    let state = DeviceState {
        anc: AncMode::anc(AncScene::Indoor, Some(2)).unwrap(),
        channel_balance: 30,
        game_mode: true,
        name: Some("Meu Fone".to_string()),
        touch: Some(touch),
        ..DeviceState::default()
    };

    let mut app = online_app();
    app.busy = Some("Sincronizando estado…".to_string());
    app.error = Some("erro antigo".to_string());

    let _ = app.finish(ActionResult {
        state: state.clone(),
        notice: Some("Estado atualizado a partir do fone.".to_string()),
        connected: true,
        address: Some("AA:BB:CC:DD:EE:FF".to_string()),
        refresh_drafts: true,
        ..ActionResult::default()
    });

    assert!(app.busy.is_none());
    assert!(app.connected);
    assert_eq!(app.state, state);
    assert_eq!(app.address_input, "AA:BB:CC:DD:EE:FF");
    assert_eq!(app.anc_choice, AncChoice::Scene(AncScene::Indoor));
    assert_eq!(app.anc_level, 2);
    assert_eq!(app.balance, 30);
    assert_eq!(
        app.touch_draft.get(TouchKey::LeftSingle),
        TouchFunction::PlayPause
    );
    assert_eq!(app.rename_input, "Meu Fone");
    assert!(app.error.is_none());
    assert_eq!(
        app.notice.as_deref(),
        Some("Estado atualizado a partir do fone.")
    );
}

#[test]
fn finish_reports_a_lost_link_and_keeps_drafts_without_refresh() {
    use melobudsnext::protocol::anc::AncScene;
    use melobudsnext::DeviceState;

    let mut app = online_app();
    app.connected = true;
    app.busy = Some("Desligando…".to_string());
    app.anc_choice = AncChoice::Scene(AncScene::Wind);

    let _ = app.finish(ActionResult {
        state: DeviceState::default(),
        error: Some("BLE link lost; connect again".to_string()),
        connected: false,
        refresh_drafts: false,
        ..ActionResult::default()
    });

    assert!(!app.connected);
    assert!(app.busy.is_none());
    assert_eq!(app.error.as_deref(), Some("BLE link lost; connect again"));
    assert!(app.notice.is_none());
    // The action did not ask for a draft reload, so local edits survive.
    assert_eq!(app.anc_choice, AncChoice::Scene(AncScene::Wind));
}

#[test]
fn load_drafts_maps_every_anc_mode() {
    use melobudsnext::protocol::anc::{AncMode, AncScene, TransparencyMode};

    let cases = [
        (AncMode::Off, AncChoice::Off),
        (
            AncMode::anc(AncScene::Daily, Some(3)).unwrap(),
            AncChoice::Scene(AncScene::Daily),
        ),
        (
            AncMode::transparency(TransparencyMode::Intensity(4)).unwrap(),
            AncChoice::Transparency,
        ),
        (
            AncMode::transparency(TransparencyMode::VoiceEnhancement).unwrap(),
            AncChoice::Transparency,
        ),
        (
            AncMode::Unknown([0x02, 0x07, 0x01]),
            AncChoice::Unknown([0x02, 0x07, 0x01]),
        ),
    ];

    for (mode, expected) in cases {
        let mut app = online_app();
        app.state.anc = mode;
        app.load_drafts();
        assert_eq!(app.anc_choice, expected, "mode {mode}");
    }

    let mut app = online_app();
    app.state.anc = AncMode::transparency(TransparencyMode::Intensity(5)).unwrap();
    app.load_drafts();
    assert!(!app.transparency_voice);
    assert_eq!(app.transparency_intensity, 5);

    app.state.anc = AncMode::transparency(TransparencyMode::VoiceEnhancement).unwrap();
    app.load_drafts();
    assert!(app.transparency_voice);

    app.state.anc = AncMode::anc(AncScene::Indoor, Some(1)).unwrap();
    app.load_drafts();
    assert_eq!(app.anc_level, 1);
}

#[test]
fn load_drafts_reads_volume_touch_wear_and_name() {
    use melobudsnext::protocol::commands::{NotificationVolume, WearDetection};
    use melobudsnext::protocol::touch::{TouchFunction, TouchKey, TouchMapping};

    let mut app = online_app();
    let mut touch = TouchMapping::new();
    touch.set(TouchKey::RightDouble, TouchFunction::VolumeUp);
    app.state.touch = Some(touch.clone());
    app.state.notification_volume = Some(0x08);
    app.state.channel_balance = 20;
    app.state.wear_detection = Some(WearDetection {
        enabled: true,
        music_index: 2,
        keep_anc: true,
        tone: 3,
    });
    app.state.name = Some("Meu Fone".to_string());

    app.load_drafts();

    assert_eq!(app.balance, 20);
    assert_eq!(app.volume, Some(VolumeOption(NotificationVolume::High)));
    assert_eq!(app.touch_draft, touch);
    assert!(app.wear_enabled);
    assert!(app.wear_keep_anc);
    assert_eq!(app.wear_music, "2");
    assert_eq!(app.wear_tone, "3");
    assert_eq!(app.rename_input, "Meu Fone");
}

#[test]
fn load_drafts_uses_defaults_when_the_state_is_missing() {
    let mut app = online_app();
    app.state.touch = None;
    app.state.notification_volume = None;
    app.state.wear_detection = None;
    app.state.name = None;

    app.load_drafts();

    assert_eq!(app.volume, None);
    assert_eq!(app.touch_draft, Default::default());
    assert!(!app.wear_enabled);
    assert!(!app.wear_keep_anc);
    assert_eq!(app.wear_music, "1");
    assert_eq!(app.wear_tone, "0");
    assert!(app.rename_input.is_empty());
}

#[test]
fn active_anc_mode_validates_the_draft() {
    use melobudsnext::protocol::anc::{AncMode, AncScene, TransparencyMode};

    let mut app = online_app();

    app.anc_choice = AncChoice::Off;
    assert_eq!(app.active_anc_mode(), Ok(AncMode::Off));

    app.anc_choice = AncChoice::Scene(AncScene::Noise);
    app.anc_level = 3;
    assert_eq!(
        app.active_anc_mode(),
        Ok(AncMode::anc(AncScene::Noise, Some(3)).unwrap())
    );

    app.anc_level = 0;
    assert!(app.active_anc_mode().is_err());

    app.anc_choice = AncChoice::Scene(AncScene::Wind);
    assert_eq!(
        app.active_anc_mode(),
        Ok(AncMode::anc(AncScene::Wind, None).unwrap())
    );

    app.anc_choice = AncChoice::Transparency;
    app.transparency_voice = false;
    app.transparency_intensity = 6;
    assert_eq!(
        app.active_anc_mode(),
        Ok(AncMode::transparency(TransparencyMode::Intensity(6)).unwrap())
    );

    app.transparency_intensity = 7;
    assert!(app.active_anc_mode().is_err());
}

#[test]
fn offline_build_skips_the_automatic_connection() {
    let config = Config::default().with_mac("AA:BB:CC:DD:EE:FF");

    let (online, _) = App::build(config.clone(), None, false);
    assert_eq!(online.busy.as_deref(), Some("Conectando ao fone…"));
    assert_eq!(online.address_input, "AA:BB:CC:DD:EE:FF");

    let (offline, _) = App::build(config, None, true);
    assert!(offline.busy.is_none());
    assert_eq!(offline.address_input, "AA:BB:CC:DD:EE:FF");
}

// -----------------------------------------------------------------------
// TST-06 — regressões de COR-03..07 e COR-10
// -----------------------------------------------------------------------

#[test]
fn cor_03_offline_actions_never_reach_the_session() {
    let mut app = offline_app();
    app.notice = None;

    for action in [Message::ApplyAnc, Message::ApplyTouch, Message::ApplyWear] {
        let _ = update(&mut app, action);
        assert!(app.busy.is_none());
        assert!(app.notice.is_some());
    }

    // The whole layout must still render without a BLE session.
    for tab in Tab::ALL {
        app.tab = tab;
        let _ = view(&app);
    }
}

#[test]
fn cor_04_lost_or_unknown_link_discards_the_session() {
    use melobudsnext::actions::discards_client;

    assert!(discards_client(Some(false)));
    assert!(discards_client(None));
    assert!(!discards_client(Some(true)));
}

#[test]
fn cor_05_successful_toggle_survives_without_an_echo() {
    use melobudsnext::actions::apply_command_result;
    use melobudsnext::protocol::commands;
    use melobudsnext::DeviceState;

    let mut app = online_app();
    let _ = update(&mut app, Message::ToggleGame(true));
    assert!(app.state.game_mode);

    // The session now applies the command itself because no echo arrived.
    let mut session_state = DeviceState::default();
    apply_command_result(&mut session_state, &commands::game_mode(true), &[]);

    let _ = app.finish(ActionResult {
        state: session_state,
        connected: true,
        notice: Some("Modo jogo ativado.".to_string()),
        ..ActionResult::default()
    });

    assert!(app.state.game_mode, "the toggle must not revert");
}

#[test]
fn cor_06_unknown_anc_is_never_sent_as_off() {
    use melobudsnext::protocol::anc::AncMode;

    let mut app = online_app();
    app.state.anc = AncMode::Unknown([0x01, 0x09, 0x00]);
    app.load_drafts();

    let _ = update(&mut app, Message::ApplyAnc);

    assert!(app.busy.is_none(), "no BLE command must be dispatched");
    assert!(app.error.is_some(), "the user must be told about the state");
    assert_eq!(app.state.anc, AncMode::Unknown([0x01, 0x09, 0x00]));
}

#[test]
fn cor_07_unknown_touch_is_visible_and_preserved_on_save() {
    use melobudsnext::protocol::touch::{TouchFunction, TouchKey, TouchMapping};

    let mut app = online_app();
    let mut touch = TouchMapping::new();
    touch.set(TouchKey::LeftSingle, TouchFunction::Unknown(0x42));
    app.state.touch = Some(touch);
    app.load_drafts();

    assert_eq!(
        app.touch_draft.get(TouchKey::LeftSingle),
        TouchFunction::Unknown(0x42)
    );

    let option = TouchOption::options(TouchFunction::Unknown(0x42))
        .into_iter()
        .find(|option| option.0 == TouchFunction::Unknown(0x42))
        .expect("unknown value must be listed");
    let _ = update(
        &mut app,
        Message::TouchChanged(TouchKey::LeftSingle, option),
    );

    // The draft keeps the original byte when the map is written back.
    assert_eq!(app.touch_draft.encode()[1], 0x42);
}
