use std::ops::{Deref, DerefMut};

use derive_more::{Deref, DerefMut};
use egui::plot::{Legend, Line, LinkedAxisGroup, Plot, Value, Values};
use egui::{TextStyle, Ui};
use serde::{Deserialize, Serialize};

use crate::app::PlotData;
use crate::util::{self, format_time};

const POWER_ASPECT_RATIO: f32 = 0.006;
const VELOCITY_ASPECT_RATIO: f32 = 1.0;
const TORQUE_ASPECT_RATIO: f32 = 0.08;
const TEMP_ASPECT_RATIO: f32 = 2.0;

const DEFAULT_GRID_MODE: bool = true;
const DEFAULT_LINKED: bool = true;

pub trait WheelPlotConfig: Deref<Target = WheelConfig> + DerefMut {
    const NAME: &'static str;
    const ASPECT_RATIO: f32;
    fn format_label(name: &str, val: &Value) -> String;
}

#[derive(Serialize, Deserialize)]
pub struct WheelConfig {
    aspect_ratio: f32,
    grid_mode: bool,
    linked: bool,
    #[serde(skip)]
    #[serde(default = "LinkedAxisGroup::both")]
    axis_group: LinkedAxisGroup,
}

#[derive(Serialize, Deserialize, Deref, DerefMut)]
pub struct PowerConfig(WheelConfig);

impl Default for PowerConfig {
    fn default() -> Self {
        Self(WheelConfig {
            aspect_ratio: POWER_ASPECT_RATIO,
            grid_mode: DEFAULT_GRID_MODE,
            linked: DEFAULT_LINKED,
            axis_group: LinkedAxisGroup::both(),
        })
    }
}

impl WheelPlotConfig for PowerConfig {
    const NAME: &'static str = "power";
    const ASPECT_RATIO: f32 = POWER_ASPECT_RATIO;

    fn format_label(_name: &str, val: &Value) -> String {
        let x = format_time(val.x);
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}\nP = {y}W")
    }
}

#[derive(Serialize, Deserialize, Deref, DerefMut)]
pub struct VelocityConfig(WheelConfig);

impl Default for VelocityConfig {
    fn default() -> Self {
        Self(WheelConfig {
            aspect_ratio: VELOCITY_ASPECT_RATIO,
            grid_mode: DEFAULT_GRID_MODE,
            linked: DEFAULT_LINKED,
            axis_group: LinkedAxisGroup::both(),
        })
    }
}

impl WheelPlotConfig for VelocityConfig {
    const NAME: &'static str = "velocity";
    const ASPECT_RATIO: f32 = VELOCITY_ASPECT_RATIO;

    fn format_label(_name: &str, val: &Value) -> String {
        let x = format_time(val.x);
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}\nv = {y}km/h")
    }
}

#[derive(Serialize, Deserialize, Deref, DerefMut)]
pub struct TorqueConfig(WheelConfig);

impl Default for TorqueConfig {
    fn default() -> Self {
        Self(WheelConfig {
            aspect_ratio: TORQUE_ASPECT_RATIO,
            grid_mode: DEFAULT_GRID_MODE,
            linked: DEFAULT_LINKED,
            axis_group: LinkedAxisGroup::both(),
        })
    }
}

impl WheelPlotConfig for TorqueConfig {
    const NAME: &'static str = "torque";
    const ASPECT_RATIO: f32 = TORQUE_ASPECT_RATIO;

    fn format_label(name: &str, val: &Value) -> String {
        let x = format_time(val.x);
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("{name}\nt = {x}\nM = {y}Nm")
    }
}

#[derive(Serialize, Deserialize, Deref, DerefMut)]
pub struct Temp1Config(WheelConfig);

impl Default for Temp1Config {
    fn default() -> Self {
        Self(WheelConfig {
            aspect_ratio: TEMP_ASPECT_RATIO,
            grid_mode: DEFAULT_GRID_MODE,
            linked: DEFAULT_LINKED,
            axis_group: LinkedAxisGroup::both(),
        })
    }
}

impl WheelPlotConfig for Temp1Config {
    const NAME: &'static str = "temp1";
    const ASPECT_RATIO: f32 = TEMP_ASPECT_RATIO;

    fn format_label(name: &str, val: &Value) -> String {
        let x = format_time(val.x);
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("{name}\nt = {x}\nT = {y}°C")
    }
}

pub fn wheel_config<T: WheelPlotConfig>(ui: &mut Ui, cfg: &mut T) {
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
    wheel_plot(
        ui,
        cfg,
        [(line(data.power.fl.clone()), "")],
        [(line(data.power.fr.clone()), "")],
        [(line(data.power.rl.clone()), "")],
        [(line(data.power.rr.clone()), "")],
    );
}

pub fn velocity_plot(ui: &mut Ui, data: &PlotData, cfg: &VelocityConfig) {
    wheel_plot(
        ui,
        cfg,
        [(line(data.velocity.fl.clone()), "")],
        [(line(data.velocity.fr.clone()), "")],
        [(line(data.velocity.rl.clone()), "")],
        [(line(data.velocity.rr.clone()), "")],
    );
}

pub fn torque_plot(ui: &mut Ui, data: &PlotData, cfg: &TorqueConfig) {
    wheel_plot(
        ui,
        cfg,
        [
            (line(data.torque_set.fl.clone()), "set"),
            (line(data.torque_real.fl.clone()), "real"),
        ],
        [
            (line(data.torque_set.fr.clone()), "set"),
            (line(data.torque_real.fr.clone()), "real"),
        ],
        [
            (line(data.torque_set.rl.clone()), "set"),
            (line(data.torque_real.rl.clone()), "real"),
        ],
        [
            (line(data.torque_set.rr.clone()), "set"),
            (line(data.torque_real.rr.clone()), "real"),
        ],
    );
}

pub fn temp1_plot(ui: &mut Ui, data: &PlotData, cfg: &Temp1Config) {
    wheel_plot(
        ui,
        cfg,
        [
            (line(data.temp.fl.clone()), "temp"),
            (line(data.room_temp.fl.clone()), "room temp"),
            (line(data.heatsink_temp.fl.clone()), "heatsink temp"),
        ],
        [
            (line(data.temp.fr.clone()), "temp"),
            (line(data.room_temp.fr.clone()), "room temp"),
            (line(data.heatsink_temp.fr.clone()), "heatsink temp"),
        ],
        [
            (line(data.temp.rl.clone()), "temp"),
            (line(data.room_temp.rl.clone()), "room temp"),
            (line(data.heatsink_temp.rl.clone()), "heatsink temp"),
        ],
        [
            (line(data.temp.rr.clone()), "temp"),
            (line(data.room_temp.rr.clone()), "room temp"),
            (line(data.heatsink_temp.rr.clone()), "heatsink temp"),
        ],
    );
}

fn line(values: Vec<Value>) -> Line {
    Line::new(Values::from_values(values))
}

fn wheel_plot<T: WheelPlotConfig, const COUNT: usize>(
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
                .label_formatter(move |n, v| T::format_label(n, v))
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
                .label_formatter(move |n, v| T::format_label(n, v))
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
                .label_formatter(move |n, v| T::format_label(n, v))
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
                .label_formatter(move |n, v| T::format_label(n, v))
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
            .link_axis(cfg.axis_group.clone())
            .label_formatter(move |n, v| T::format_label(n, v))
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

#[derive(Serialize, Deserialize, Default)]
pub struct Temp2Config {
    aspect_ratio: f32,
}

pub fn temp2_config(ui: &mut Ui, cfg: &mut Temp2Config) {
    util::ratio_slider(ui, &mut cfg.aspect_ratio, TEMP_ASPECT_RATIO, 100.0);
}

pub fn temp2_plot(ui: &mut Ui, data: &PlotData, cfg: &Temp2Config) {
    Plot::new("temp2")
        .data_aspect(cfg.aspect_ratio)
        .label_formatter(move |n, v| Temp1Config::format_label(n, v))
        .legend(Legend::default())
        .show(ui, |ui| {
            ui.line(line(data.ams_temp_max.clone()).name("ams temp max"));
            ui.line(line(data.water_temp_converter.clone()).name("water temp converter"));
            ui.line(line(data.water_temp_motor.clone()).name("water temp motor"));
        });
}
