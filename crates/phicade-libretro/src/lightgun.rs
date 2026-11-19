use phicade_runtime::ActionKind;

/// libretro.h absolute lightgun ABI. Only port 0 is exposed by this host.
/// https://github.com/libretro/libretro-common/blob/master/include/libretro.h
#[derive(Debug, Clone, Copy)]
pub struct LightGunState {
    pub x: i16,
    pub y: i16,
    pub trigger: bool,
    pub reload: bool,
    pub offscreen: bool,
    trigger_pulse: bool,
    reload_pulse: bool,
}
impl Default for LightGunState {
    fn default() -> Self {
        Self {
            x: -32768,
            y: -32768,
            trigger: false,
            reload: false,
            offscreen: true,
            trigger_pulse: false,
            reload_pulse: false,
        }
    }
}
impl LightGunState {
    pub fn apply(&mut self, action: &ActionKind) {
        match action {
            ActionKind::Axis { axis, value } if axis == "GUN_X" => self.x = *value,
            ActionKind::Axis { axis, value } if axis == "GUN_Y" => self.y = *value,
            ActionKind::Button { button, pressed } if button == "GUN_TRIGGER" => {
                self.trigger = *pressed;
                self.trigger_pulse |= *pressed;
            }
            ActionKind::Button { button, pressed } if button == "GUN_RELOAD" => {
                self.reload = *pressed;
                self.reload_pulse |= *pressed;
            }
            ActionKind::Button { button, pressed } if button == "GUN_OFFSCREEN" => {
                self.offscreen = *pressed
            }
            _ => {}
        }
    }
    pub fn input(&self, id: u32) -> i16 {
        match id {
            13 => self.x,
            14 => self.y,
            15 => i16::from(self.offscreen),
            2 => i16::from(self.trigger || self.trigger_pulse),
            16 => i16::from(self.reload || self.reload_pulse),
            _ => 0,
        }
    }
    pub fn finish_frame(&mut self) {
        self.trigger_pulse = false;
        self.reload_pulse = false;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absolute_coordinates_trigger_reload_and_offscreen_match_abi() {
        let mut g = LightGunState::default();
        assert_eq!(g.input(15), 1);
        g.apply(&ActionKind::Axis {
            axis: "GUN_X".into(),
            value: 12345,
        });
        g.apply(&ActionKind::Button {
            button: "GUN_TRIGGER".into(),
            pressed: true,
        });
        assert_eq!(g.input(13), 12345);
        assert_eq!(g.input(2), 1);
        assert_eq!(g.input(999), 0);
    }
    #[test]
    fn brief_fire_and_reload_edges_survive_one_native_frame() {
        let mut g = LightGunState::default();
        for pressed in [true, false] {
            g.apply(&ActionKind::Button {
                button: "GUN_TRIGGER".into(),
                pressed,
            });
            g.apply(&ActionKind::Button {
                button: "GUN_RELOAD".into(),
                pressed,
            });
        }
        assert_eq!(g.input(2), 1);
        assert_eq!(g.input(16), 1);
        g.finish_frame();
        assert_eq!(g.input(2), 0);
        assert_eq!(g.input(16), 0);
    }
}
