use egui::plot::{Legend, Line, Plot, Value, Values};
use egui::{TextStyle, Ui};
use serde::{Deserialize, Serialize};

use crate::app::PlotData;
use crate::util;

const POWER_ASPECT_RATIO: f32 = 0.006;
const VELOCITY_ASPECT_RATIO: f32 = 1.0;
const TORQUE_ASPECT_RATIO: f32 = 0.08;

trait MotorConfig {
    fn format_label(name: &str, value: &Value) -> String;
    fn name() -> &'static str;
    fn aspect_ratio(&self) -> f32;
    fn mode(&self) -> Mode;
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Mode {
    Split,
    Single,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Split
    }
}

impl From<bool> for Mode {
    fn from(checked: bool) -> Self {
        match checked {
            true => Self::Split,
            false => Self::Single,
        }
    }
}

impl Mode {
    /// Returns `true` if the mode is [`Split`].
    ///
    /// [`Split`]: Mode::Split
    pub fn is_split(&self) -> bool {
        matches!(self, Self::Split)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PowerConfig {
    aspect_ratio: f32,
    mode: Mode,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: POWER_ASPECT_RATIO,
            mode: Mode::default(),
        }
    }
}

impl MotorConfig for PowerConfig {
    fn format_label(_name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}s\nP = {y}W")
    }

    fn name() -> &'static str {
        "power"
    }

    fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    fn mode(&self) -> Mode {
        self.mode
    }
}

#[derive(Serialize, Deserialize)]
pub struct VelocityConfig {
    aspect_ratio: f32,
    mode: Mode,
}

impl Default for VelocityConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: VELOCITY_ASPECT_RATIO,
            mode: Mode::default(),
        }
    }
}

impl MotorConfig for VelocityConfig {
    fn format_label(_name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}s\nv = {y}km/h")
    }

    fn name() -> &'static str {
        "velocity"
    }

    fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    fn mode(&self) -> Mode {
        self.mode
    }
}

#[derive(Serialize, Deserialize)]
pub struct TorqueConfig {
    aspect_ratio: f32,
    mode: Mode,
}

impl Default for TorqueConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: TORQUE_ASPECT_RATIO,
            mode: Mode::default(),
        }
    }
}

impl MotorConfig for TorqueConfig {
    fn format_label(name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("{name}\nt = {x}s\nM = {y}Nm")
    }

    fn name() -> &'static str {
        "torque"
    }

    fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    fn mode(&self) -> Mode {
        self.mode
    }
}

pub fn power_config(ui: &mut Ui, cfg: &mut PowerConfig) {
    util::ratio_slider(ui, &mut cfg.aspect_ratio, POWER_ASPECT_RATIO, 100.0);
    ui.add_space(40.0);
    util::mode_toggle(ui, &mut cfg.mode);
}

pub fn velocity_config(ui: &mut Ui, cfg: &mut VelocityConfig) {
    util::ratio_slider(ui, &mut cfg.aspect_ratio, VELOCITY_ASPECT_RATIO, 100.0);
    ui.add_space(40.0);
    util::mode_toggle(ui, &mut cfg.mode);
}

pub fn torque_config(ui: &mut Ui, cfg: &mut TorqueConfig) {
    util::ratio_slider(ui, &mut cfg.aspect_ratio, TORQUE_ASPECT_RATIO, 100.0);
    ui.add_space(40.0);
    util::mode_toggle(ui, &mut cfg.mode);
}

pub fn power_plot(ui: &mut Ui, data: &PlotData, cfg: &PowerConfig) {
    plot(
        ui,
        cfg,
        [line(data.power.fl.clone(), "")],
        [line(data.power.fr.clone(), "")],
        [line(data.power.rl.clone(), "")],
        [line(data.power.rr.clone(), "")],
    );
}

pub fn velocity_plot(ui: &mut Ui, data: &PlotData, cfg: &VelocityConfig) {
    plot(
        ui,
        cfg,
        [line(data.velocity.fl.clone(), "")],
        [line(data.velocity.fr.clone(), "")],
        [line(data.velocity.rl.clone(), "")],
        [line(data.velocity.rr.clone(), "")],
    );
}

pub fn torque_plot(ui: &mut Ui, data: &PlotData, cfg: &TorqueConfig) {
    plot(
        ui,
        cfg,
        [
            line(data.torque_set.fl.clone(), "set"),
            line(data.torque_real.fl.clone(), "real"),
        ],
        [
            line(data.torque_set.fr.clone(), "set"),
            line(data.torque_real.fr.clone(), "real"),
        ],
        [
            line(data.torque_set.rl.clone(), "set"),
            line(data.torque_real.rl.clone(), "real"),
        ],
        [
            line(data.torque_set.rr.clone(), "set"),
            line(data.torque_real.rr.clone(), "real"),
        ],
    );
}

fn line(values: Vec<Value>, name: &str) -> (Line, &str) {
    (Line::new(Values::from_values(values)), name)
}

fn plot<T: MotorConfig, const COUNT: usize>(
    ui: &mut Ui,
    cfg: &T,
    fl: [(Line, &str); COUNT],
    fr: [(Line, &str); COUNT],
    rl: [(Line, &str); COUNT],
    rr: [(Line, &str); COUNT],
) {
    let h = ui.available_height() / 2.0
        - ui.fonts().row_height(&TextStyle::Body.resolve(ui.style()))
        - ui.style().spacing.item_spacing.y;

    match cfg.mode() {
        Mode::Split => ui.columns(2, |uis| {
            let ui = &mut uis[0];
            ui.label("front left");
            Plot::new(format!("fl_{}", T::name()))
                .height(h)
                .data_aspect(cfg.aspect_ratio())
                .custom_label_func(move |n, v| T::format_label(n, v))
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (l, n) in fl {
                        ui.line(l.name(n));
                    }
                });
            ui.label("rear left");
            Plot::new(format!("rl_{}", T::name()))
                .height(h)
                .data_aspect(cfg.aspect_ratio())
                .custom_label_func(move |n, v| T::format_label(n, v))
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (l, n) in rl {
                        ui.line(l.name(n));
                    }
                });

            let ui = &mut uis[1];
            ui.label("front right");
            Plot::new(format!("fr_{}", T::name()))
                .height(h)
                .data_aspect(cfg.aspect_ratio())
                .custom_label_func(move |n, v| T::format_label(n, v))
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (l, n) in fr {
                        ui.line(l.name(n));
                    }
                });
            ui.label("rear right");
            Plot::new(format!("rr_{}", T::name()))
                .height(h)
                .data_aspect(cfg.aspect_ratio())
                .custom_label_func(move |n, v| T::format_label(n, v))
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (l, n) in rr {
                        ui.line(l.name(n));
                    }
                });
        }),
        Mode::Single => {
            Plot::new(T::name())
                .data_aspect(cfg.aspect_ratio())
                .custom_label_func(move |n, v| T::format_label(n, v))
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (l, n) in fl {
                        ui.line(l.name(format!("{n} front left")));
                    }
                    for (l, n) in fr {
                        ui.line(l.name(format!("{n} front right")));
                    }
                    for (l, n) in rl {
                        ui.line(l.name(format!("{n} rear left")));
                    }
                    for (l, n) in rr {
                        ui.line(l.name(format!("{n} rear right")));
                    }
                });
        }
    }
}
