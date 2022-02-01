use std::ops::{Deref, DerefMut};

use egui::plot::{Legend, Line, LinkedAxisGroup, Plot, Value, Values};
use egui::{TextStyle, Ui};
use serde::{Deserialize, Serialize};

use crate::app::PlotData;
use crate::util;

const POWER_ASPECT_RATIO: f32 = 0.006;
const VELOCITY_ASPECT_RATIO: f32 = 1.0;
const TORQUE_ASPECT_RATIO: f32 = 0.08;

const DEFAULT_GRID_MODE: bool = true;
const DEFAULT_LINKED: bool = true;

pub trait MotorPlotConfig: Deref<Target = MotorConfig> + DerefMut {
    const NAME: &'static str;
    const ASPECT_RATIO: f32;
    fn format_label(name: &str, val: &Value) -> String;
}

#[derive(Serialize, Deserialize)]
pub struct MotorConfig {
    aspect_ratio: f32,
    grid_mode: bool,
    linked: bool,
    #[serde(skip)]
    #[serde(default = "LinkedAxisGroup::both")]
    axis_group: LinkedAxisGroup,
}

#[derive(Serialize, Deserialize)]
pub struct PowerConfig(MotorConfig);

impl Default for PowerConfig {
    fn default() -> Self {
        Self(MotorConfig {
            aspect_ratio: POWER_ASPECT_RATIO,
            grid_mode: DEFAULT_GRID_MODE,
            linked: DEFAULT_LINKED,
            axis_group: LinkedAxisGroup::both(),
        })
    }
}

impl Deref for PowerConfig {
    type Target = MotorConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PowerConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MotorPlotConfig for PowerConfig {
    const NAME: &'static str = "power";
    const ASPECT_RATIO: f32 = POWER_ASPECT_RATIO;

    fn format_label(_name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}s\nP = {y}W")
    }
}

#[derive(Serialize, Deserialize)]
pub struct VelocityConfig(MotorConfig);

impl Default for VelocityConfig {
    fn default() -> Self {
        Self(MotorConfig {
            aspect_ratio: VELOCITY_ASPECT_RATIO,
            grid_mode: DEFAULT_GRID_MODE,
            linked: DEFAULT_LINKED,
            axis_group: LinkedAxisGroup::both(),
        })
    }
}

impl Deref for VelocityConfig {
    type Target = MotorConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VelocityConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MotorPlotConfig for VelocityConfig {
    const NAME: &'static str = "velocity";
    const ASPECT_RATIO: f32 = VELOCITY_ASPECT_RATIO;

    fn format_label(_name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}s\nv = {y}km/h")
    }
}

#[derive(Serialize, Deserialize)]
pub struct TorqueConfig(MotorConfig);

impl Default for TorqueConfig {
    fn default() -> Self {
        Self(MotorConfig {
            aspect_ratio: TORQUE_ASPECT_RATIO,
            grid_mode: DEFAULT_GRID_MODE,
            linked: DEFAULT_LINKED,
            axis_group: LinkedAxisGroup::both(),
        })
    }
}

impl Deref for TorqueConfig {
    type Target = MotorConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TorqueConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MotorPlotConfig for TorqueConfig {
    const NAME: &'static str = "torque";
    const ASPECT_RATIO: f32 = TORQUE_ASPECT_RATIO;

    fn format_label(name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("{name}\nt = {x}s\nM = {y}Nm")
    }
}

pub fn config<T: MotorPlotConfig>(ui: &mut Ui, cfg: &mut T) {
    util::ratio_slider(ui, &mut cfg.aspect_ratio, T::ASPECT_RATIO, 100.0);
    ui.add_space(30.0);

    ui.checkbox(&mut cfg.grid_mode, "grid mode");
    ui.add_space(30.0);

    ui.checkbox(&mut cfg.linked, "linked");
    let linked = cfg.linked;
    cfg.axis_group.set_link_x(linked);
    cfg.axis_group.set_link_y(linked);
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

fn plot<T: MotorPlotConfig, const COUNT: usize>(
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

    if cfg.grid_mode {
        ui.columns(2, |uis| {
            let ui = &mut uis[0];
            ui.label("front left");
            Plot::new(format!("fl_{}", T::NAME))
                .height(h)
                .data_aspect(cfg.aspect_ratio)
                .link_axis(cfg.axis_group.clone())
                .custom_label_func(move |n, v| T::format_label(n, v))
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (l, n) in fl {
                        ui.line(l.name(n));
                    }
                });
            ui.label("rear left");
            Plot::new(format!("rl_{}", T::NAME))
                .height(h)
                .data_aspect(cfg.aspect_ratio)
                .link_axis(cfg.axis_group.clone())
                .custom_label_func(move |n, v| T::format_label(n, v))
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (l, n) in rl {
                        ui.line(l.name(n));
                    }
                });

            let ui = &mut uis[1];
            ui.label("front right");
            Plot::new(format!("fr_{}", T::NAME))
                .height(h)
                .data_aspect(cfg.aspect_ratio)
                .link_axis(cfg.axis_group.clone())
                .custom_label_func(move |n, v| T::format_label(n, v))
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (l, n) in fr {
                        ui.line(l.name(n));
                    }
                });
            ui.label("rear right");
            Plot::new(format!("rr_{}", T::NAME))
                .height(h)
                .data_aspect(cfg.aspect_ratio)
                .link_axis(cfg.axis_group.clone())
                .custom_label_func(move |n, v| T::format_label(n, v))
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (l, n) in rr {
                        ui.line(l.name(n));
                    }
                });
        })
    } else {
        Plot::new(T::NAME)
            .data_aspect(cfg.aspect_ratio)
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