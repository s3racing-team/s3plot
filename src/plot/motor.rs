use derive_more::{Deref, DerefMut};
use egui::plot::{LinkedAxisGroup, Value};
use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::app::PlotData;
use crate::util::format_time;

use super::{line, wheel_plot, WheelConfig, WheelPlotConfig, DEFAULT_GRID_MODE, DEFAULT_LINKED};

const POWER_ASPECT_RATIO: f32 = 0.006;
const VELOCITY_ASPECT_RATIO: f32 = 1.0;
const TORQUE_ASPECT_RATIO: f32 = 0.08;

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
