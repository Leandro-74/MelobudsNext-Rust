//! Pure validation helpers shared by the command-line front end.
//!
//! Keeping this logic in the library (instead of `main.rs`) makes the
//! validation paths testable without parsing arguments or touching hardware.

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};

use crate::actions::ReconnectPolicy;
use crate::protocol::anc::{AncMode, AncScene, TransparencyMode};
use crate::protocol::touch::{TouchFunction, TouchKey, TouchMapping};

/// Builds the ANC mode selected by the CLI flags.
///
/// Transparency options (`--intensity`, `--voice`) are mutually exclusive with
/// ANC options (`--scene`, `--level`); without any option the mode is `Off`.
pub fn build_anc_mode(
    scene: Option<AncScene>,
    level: Option<u8>,
    intensity: Option<u8>,
    voice: bool,
) -> Result<AncMode> {
    if intensity.is_some() || voice {
        if scene.is_some() || level.is_some() {
            bail!("--scene and --level are only valid in ANC mode");
        }

        let mode = match (voice, intensity) {
            (true, None) => TransparencyMode::VoiceEnhancement,
            (false, Some(intensity)) => TransparencyMode::Intensity(intensity),
            (true, Some(_)) => bail!("--voice and --intensity cannot be used together"),
            (false, None) => unreachable!("guarded by the outer condition"),
        };

        return Ok(AncMode::transparency(mode)?);
    }

    match scene {
        Some(scene) => Ok(AncMode::anc(scene, level)?),
        None => {
            if level.is_some() {
                bail!("--level requires --scene");
            }
            Ok(AncMode::Off)
        }
    }
}

/// Validates the `--wait` value and converts it to a duration.
pub fn wait_duration(seconds: f64) -> Result<Duration> {
    if !(0.0..=3600.0).contains(&seconds) {
        bail!("wait must be a number of seconds between 0 and 3600");
    }
    Ok(Duration::from_secs_f64(seconds))
}

/// Validates the number of reconnection attempts after a headphone restart.
pub fn reconnect_attempts(attempts: u32) -> Result<u32> {
    if attempts == 0 {
        bail!("reconnect attempts must be at least 1");
    }
    Ok(attempts)
}

/// Validates the interval between reconnection attempts.
pub fn reconnect_delay(seconds: f64) -> Result<Duration> {
    if !(0.0..=60.0).contains(&seconds) {
        bail!("reconnect delay must be a number of seconds between 0 and 60");
    }
    Ok(Duration::from_secs_f64(seconds))
}

/// Builds the reconnection policy used when the headphones restart.
pub fn reconnect_policy(attempts: u32, delay_seconds: f64) -> Result<ReconnectPolicy> {
    Ok(ReconnectPolicy {
        attempts: reconnect_attempts(attempts)?,
        delay: reconnect_delay(delay_seconds)?,
    })
}

/// Parses `KEY=FUNCTION` touch bindings into a complete mapping.
pub fn mapping_from_bindings(bindings: &[String]) -> Result<TouchMapping> {
    let mut mapping = TouchMapping::new();

    for binding in bindings {
        let (key, function) = binding
            .split_once('=')
            .ok_or_else(|| anyhow!("invalid binding {binding:?}; expected KEY=FUNCTION"))?;
        let key = key
            .parse::<TouchKey>()
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("invalid touch key {key:?}"))?;
        let function = function
            .parse::<TouchFunction>()
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("invalid touch function {function:?}"))?;
        mapping.set(key, function);
    }

    Ok(mapping)
}

#[cfg(test)]
mod tests {
    use super::{build_anc_mode, mapping_from_bindings, reconnect_policy, wait_duration};
    use crate::protocol::anc::{AncMode, AncScene, TransparencyMode};
    use crate::protocol::touch::{TouchFunction, TouchKey};

    #[test]
    fn builds_off_when_no_anc_option_is_given() {
        assert_eq!(
            build_anc_mode(None, None, None, false).unwrap(),
            AncMode::Off
        );
    }

    #[test]
    fn builds_anc_scenes_with_and_without_levels() {
        assert_eq!(
            build_anc_mode(Some(AncScene::Indoor), Some(2), None, false).unwrap(),
            AncMode::anc(AncScene::Indoor, Some(2)).unwrap()
        );
        assert_eq!(
            build_anc_mode(Some(AncScene::Wind), None, None, false).unwrap(),
            AncMode::anc(AncScene::Wind, None).unwrap()
        );
    }

    #[test]
    fn builds_transparency_with_intensity_or_voice() {
        assert_eq!(
            build_anc_mode(None, None, Some(4), false).unwrap(),
            AncMode::transparency(TransparencyMode::Intensity(4)).unwrap()
        );
        assert_eq!(
            build_anc_mode(None, None, None, true).unwrap(),
            AncMode::transparency(TransparencyMode::VoiceEnhancement).unwrap()
        );
    }

    #[test]
    fn rejects_mixed_anc_and_transparency_options() {
        assert!(build_anc_mode(Some(AncScene::Indoor), None, Some(2), false).is_err());
        assert!(build_anc_mode(Some(AncScene::Indoor), Some(2), None, true).is_err());
        assert!(build_anc_mode(None, None, Some(2), true).is_err());
    }

    #[test]
    fn rejects_level_without_scene_and_invalid_levels() {
        assert!(build_anc_mode(None, Some(2), None, false).is_err());
        assert!(build_anc_mode(Some(AncScene::Indoor), Some(0), None, false).is_err());
        assert!(build_anc_mode(Some(AncScene::Indoor), Some(4), None, false).is_err());
        assert!(build_anc_mode(Some(AncScene::Wind), Some(1), None, false).is_err());
    }

    #[test]
    fn rejects_invalid_transparency_intensity() {
        assert!(build_anc_mode(None, None, Some(0), false).is_err());
        assert!(build_anc_mode(None, None, Some(7), false).is_err());
    }

    #[test]
    fn validates_the_wait_window() {
        assert_eq!(wait_duration(0.0).unwrap().as_secs_f64(), 0.0);
        assert_eq!(wait_duration(1.5).unwrap().as_millis(), 1500);
        assert_eq!(wait_duration(3600.0).unwrap().as_secs(), 3600);
        assert!(wait_duration(-0.1).is_err());
        assert!(wait_duration(3600.1).is_err());
        assert!(wait_duration(f64::NAN).is_err());
    }

    #[test]
    fn validates_the_reconnection_policy() {
        let policy = reconnect_policy(4, 0.5).unwrap();
        assert_eq!(policy.attempts, 4);
        assert_eq!(policy.delay.as_millis(), 500);

        assert!(reconnect_policy(0, 1.0).is_err());
        assert!(reconnect_policy(1, -0.1).is_err());
        assert!(reconnect_policy(1, 60.1).is_err());
        assert!(reconnect_policy(1, f64::NAN).is_err());
    }

    #[test]
    fn parses_touch_bindings() {
        let bindings = vec![
            "left-single=play-pause".to_string(),
            "right-single=next".to_string(),
        ];

        let mapping = mapping_from_bindings(&bindings).unwrap();

        assert_eq!(mapping.get(TouchKey::LeftSingle), TouchFunction::PlayPause);
        assert_eq!(mapping.get(TouchKey::RightSingle), TouchFunction::Next);
        assert_eq!(mapping.get(TouchKey::LeftDouble), TouchFunction::None);
    }

    #[test]
    fn last_binding_wins_for_duplicate_keys() {
        let bindings = vec![
            "left-single=play-pause".to_string(),
            "left-single=next".to_string(),
        ];

        let mapping = mapping_from_bindings(&bindings).unwrap();

        assert_eq!(mapping.get(TouchKey::LeftSingle), TouchFunction::Next);
    }

    #[test]
    fn rejects_malformed_touch_bindings() {
        assert!(mapping_from_bindings(&["left-single".to_string()]).is_err());
        assert!(mapping_from_bindings(&["unknown=play-pause".to_string()]).is_err());
        assert!(mapping_from_bindings(&["left-single=unknown".to_string()]).is_err());
    }
}
