//! Input device settings through the API (D35): `input.configure` sets
//! the key repeat and the libinput settings of touchpads and mice. The
//! core reads no file; a view or a tool calls the method.

use chora_scene::ipc::{AccelProfile, Capability, InputSettings, PointerSettings, RpcError};
use serde_json::Value;
use smithay::reexports::input;

use crate::{
    ipc::{params_of, rejected},
    state::{REPEAT_DELAY, REPEAT_RATE, State},
};

/// What the core sets on an input device: libinput on the tty backend, a
/// fake in tests.
pub trait Device {
    fn name(&self) -> String;
    /// The kernel name, such as `event5`, unique among the live devices.
    fn sysname(&self) -> String;
    /// Whether the device can tap: a touchpad, by libinput.
    fn is_touchpad(&self) -> bool;
    fn is_pointer(&self) -> bool;
    fn set_tap(&mut self, on: bool) -> Result<(), String>;
    fn set_natural_scroll(&mut self, on: bool) -> Result<(), String>;
    fn set_left_handed(&mut self, on: bool) -> Result<(), String>;
    fn set_accel_profile(&mut self, profile: AccelProfile) -> Result<(), String>;
    fn set_accel_speed(&mut self, speed: f64) -> Result<(), String>;
}

/// Applies the settings of the class of a device. A setting that the
/// device does not have is logged and skipped.
pub fn apply(settings: &InputSettings, device: &mut dyn Device) {
    let Some(p) = class_settings(settings, device) else {
        return;
    };
    let results = [
        p.tap.map(|on| ("tap", device.set_tap(on))),
        p.natural_scroll
            .map(|on| ("natural_scroll", device.set_natural_scroll(on))),
        p.left_handed
            .map(|on| ("left_handed", device.set_left_handed(on))),
        p.accel_profile
            .map(|a| ("accel_profile", device.set_accel_profile(a))),
        p.accel_speed
            .map(|s| ("accel_speed", device.set_accel_speed(s))),
    ];
    let failed: Vec<_> = results
        .into_iter()
        .flatten()
        .filter_map(|(setting, result)| Some(format!("{setting}: {}", result.err()?)))
        .collect();
    if !failed.is_empty() {
        tracing::debug!(device = device.name(), ?failed, "settings not applied");
    }
}

/// The settings of the class of a device; none for a keyboard.
fn class_settings<'a>(
    settings: &'a InputSettings,
    device: &dyn Device,
) -> Option<&'a PointerSettings> {
    if device.is_touchpad() {
        Some(&settings.touchpad)
    } else if device.is_pointer() {
        Some(&settings.mouse)
    } else {
        None
    }
}

fn checked(result: input::DeviceConfigResult) -> Result<(), String> {
    result.map_err(|e| format!("{e:?}"))
}

impl Device for input::Device {
    fn name(&self) -> String {
        input::Device::name(self).into_owned()
    }

    fn sysname(&self) -> String {
        input::Device::sysname(self).to_owned()
    }

    fn is_touchpad(&self) -> bool {
        self.config_tap_finger_count() > 0
    }

    fn is_pointer(&self) -> bool {
        self.has_capability(input::DeviceCapability::Pointer)
    }

    fn set_tap(&mut self, on: bool) -> Result<(), String> {
        checked(self.config_tap_set_enabled(on))
    }

    fn set_natural_scroll(&mut self, on: bool) -> Result<(), String> {
        checked(self.config_scroll_set_natural_scroll_enabled(on))
    }

    fn set_left_handed(&mut self, on: bool) -> Result<(), String> {
        checked(self.config_left_handed_set(on))
    }

    fn set_accel_profile(&mut self, profile: AccelProfile) -> Result<(), String> {
        checked(self.config_accel_set_profile(match profile {
            AccelProfile::Flat => input::AccelProfile::Flat,
            AccelProfile::Adaptive => input::AccelProfile::Adaptive,
        }))
    }

    fn set_accel_speed(&mut self, speed: f64) -> Result<(), String> {
        checked(self.config_accel_set_speed(speed))
    }
}

/// The settings before the first `input.configure`: the key repeat of the
/// core, libinput's defaults for the rest.
#[allow(clippy::cast_possible_truncation)] // small
pub fn defaults() -> InputSettings {
    let mut settings = InputSettings::default();
    settings.keyboard.repeat_rate = Some(REPEAT_RATE as u32);
    settings.keyboard.repeat_delay = Some(REPEAT_DELAY as u32);
    settings
}

impl State {
    /// A new libinput device gets the settings of its class.
    pub fn device_added(&mut self, mut device: Box<dyn Device>) {
        apply(&self.input_settings, device.as_mut());
        self.input_devices.push(device);
    }

    pub fn device_removed(&mut self, sysname: &str) {
        self.input_devices.retain(|d| d.sysname() != sysname);
    }

    /// The key repeat: repeats per second, 0 for none, and the delay in
    /// milliseconds.
    pub fn repeat(&self) -> (u32, u32) {
        let k = &self.input_settings.keyboard;
        (k.repeat_rate.unwrap_or(0), k.repeat_delay.unwrap_or(0))
    }

    /// `input.configure`: merges the settings, tells every keyboard of a
    /// client the repeat, and applies the rest to the devices. It changes
    /// the whole session, so a client socket needs `methods.privileged`.
    /// Empty params read the settings.
    pub fn ipc_input_configure(&mut self, conn: u64, params: Value) -> Result<Value, RpcError> {
        if self.ipc.name(conn).is_none() {
            return Err(rejected("send hello first"));
        }
        self.check_socket_grant(conn, Capability::MethodsPrivileged)?;
        let new: InputSettings = params_of(params)?;
        new.check()
            .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e))?;
        self.input_settings.merge(&new);
        let (rate, delay) = self.repeat();
        let keyboard = self.seat.get_keyboard().expect("the seat has a keyboard");
        #[allow(clippy::cast_possible_wrap)] // checked ranges
        keyboard.change_repeat_info(rate as i32, delay as i32);
        for device in &mut self.input_devices {
            apply(&self.input_settings, device.as_mut());
        }
        Ok(serde_json::to_value(&self.input_settings).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, path::PathBuf, rc::Rc};

    use serde_json::json;

    use super::*;
    use crate::testing::{Core, wayland::Wayland};

    /// A device that records what the core set.
    #[derive(Default)]
    struct Fake {
        touchpad: bool,
        set: Rc<RefCell<Vec<String>>>,
    }

    impl Fake {
        fn push(&self, what: String) -> Result<(), String> {
            self.set.borrow_mut().push(what);
            if self.touchpad {
                Ok(())
            } else {
                Err("unsupported".into())
            }
        }
    }

    impl Device for Fake {
        fn name(&self) -> String {
            "fake".into()
        }
        fn sysname(&self) -> String {
            format!("event{}", u8::from(self.touchpad))
        }
        fn is_touchpad(&self) -> bool {
            self.touchpad
        }
        fn is_pointer(&self) -> bool {
            true
        }
        fn set_tap(&mut self, on: bool) -> Result<(), String> {
            self.push(format!("tap {on}"))
        }
        fn set_natural_scroll(&mut self, on: bool) -> Result<(), String> {
            self.push(format!("natural_scroll {on}"))
        }
        fn set_left_handed(&mut self, on: bool) -> Result<(), String> {
            self.push(format!("left_handed {on}"))
        }
        fn set_accel_profile(&mut self, profile: AccelProfile) -> Result<(), String> {
            self.push(format!("accel_profile {profile:?}"))
        }
        fn set_accel_speed(&mut self, speed: f64) -> Result<(), String> {
            self.push(format!("accel_speed {speed}"))
        }
    }

    fn fake(touchpad: bool) -> (Box<Fake>, Rc<RefCell<Vec<String>>>) {
        let device = Box::new(Fake {
            touchpad,
            ..Fake::default()
        });
        let set = device.set.clone();
        (device, set)
    }

    #[test]
    fn a_device_gets_the_settings_of_its_class() {
        let settings = InputSettings {
            touchpad: PointerSettings {
                tap: Some(true),
                natural_scroll: Some(true),
                ..PointerSettings::default()
            },
            mouse: PointerSettings {
                accel_profile: Some(AccelProfile::Flat),
                accel_speed: Some(-0.5),
                ..PointerSettings::default()
            },
            ..InputSettings::default()
        };
        let (mut touchpad, tapped) = fake(true);
        apply(&settings, touchpad.as_mut());
        assert_eq!(*tapped.borrow(), ["tap true", "natural_scroll true"]);
        let (mut mouse, moved) = fake(false);
        apply(&settings, mouse.as_mut());
        assert_eq!(*moved.borrow(), ["accel_profile Flat", "accel_speed -0.5"]);
    }

    /// A core with a socket directory, for client sockets.
    fn core(tag: &str) -> (Core, PathBuf) {
        let dir = std::env::temp_dir().join(format!("chora-input-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut core = Core::new();
        core.state.ipc.path = Some(dir.join("wayland-1.sock"));
        (core, dir)
    }

    /// Gate check 6 of 0.9: `input.configure`, as `choractl view load`
    /// calls it, sets tap and key repeat. A Wayland client gets the new
    /// `repeat_info`, and a touchpad of before and one that comes later
    /// get tap.
    #[test]
    fn input_configure_sets_tap_and_key_repeat() {
        let mut core = Core::new();
        let mut wayland = Wayland::connect(&mut core);
        // The keyboard binds in the second roundtrip of connect.
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.repeat_info(), Some((25, 200)));
        let (before, tapped_before) = fake(true);
        core.state.device_added(before);
        assert!(tapped_before.borrow().is_empty(), "libinput's default");

        let mut view = core.named("view-desk");
        let settings = json!({
            "keyboard": {"repeat_rate": 40, "repeat_delay": 300},
            "touchpad": {"tap": true},
        });
        let answer = view.ok(&mut core, "input.configure", settings.clone());
        assert_eq!(answer, settings);
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.repeat_info(), Some((40, 300)));
        assert_eq!(*tapped_before.borrow(), ["tap true"]);
        let (later, tapped_later) = fake(true);
        core.state.device_added(later);
        assert_eq!(*tapped_later.borrow(), ["tap true"]);
        core.state.device_removed("event1");
        assert!(core.state.input_devices.is_empty());
    }

    /// A missing field keeps its value, empty params read the settings,
    /// values out of range fail, and a client socket needs
    /// `methods.privileged`.
    #[test]
    fn input_configure_merges_checks_and_is_privileged() {
        let (mut core, dir) = core("privileged");
        let mut view = core.named("view-desk");
        view.ok(
            &mut core,
            "input.configure",
            json!({"keyboard": {"repeat_delay": 300}}),
        );
        let answer = view.ok(
            &mut core,
            "input.configure",
            json!({"keyboard": {"repeat_rate": 0}}),
        );
        assert_eq!(
            answer,
            json!({"keyboard": {"repeat_rate": 0, "repeat_delay": 300}})
        );
        assert_eq!(view.ok(&mut core, "input.configure", json!(null)), answer);
        for wrong in [
            json!({"keyboard": {"repeat_delay": 5}}),
            json!({"touchpad": {"tapp": true}}),
            json!({"mouse": {"accel_speed": 2.0}}),
        ] {
            let (error, _) = view.fails(&mut core, "input.configure", wrong);
            assert_eq!(error, RpcError::INVALID_PARAMS);
        }
        let mut admin = core.admin();
        for (name, capabilities, error) in [
            ("agent", json!([]), Some(RpcError::MISSING_CAPABILITY)),
            ("settings", json!(["methods.privileged"]), None),
        ] {
            let create = json!({"name": name, "capabilities": capabilities});
            let created = admin.ok(&mut core, "clients.create", create);
            let mut client = core.connect_to(&PathBuf::from(created["path"].as_str().unwrap()));
            client.ok(&mut core, "hello", json!({"name": name}));
            let result = client.call(&mut core, "input.configure", json!(null));
            let refused = result.err().map(|e| e["code"].as_i64().unwrap());
            assert_eq!(refused, error, "{name}");
        }
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
