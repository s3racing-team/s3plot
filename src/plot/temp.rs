use derive_more::{Deref, DerefMut};
use egui::plot::{Legend, LinkedAxisGroup, Plot, PlotPoint};
use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::app::PlotData;
use crate::util::{self, format_time};

use super::{line, wheel_plot, WheelConfig, WheelPlotConfig, DEFAULT_GRID_MODE, DEFAULT_LINKED};

const TEMP_ASPECT_RATIO: f32 = 2.0;

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

    fn format_label(name: &str, val: &PlotPoint) -> String {
        let x = format_time(val.x);
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("{name}\nt = {x}\nT = {y}°C")
    }
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

#[derive(Serialize, Deserialize)]
pub struct Temp2Config {
    aspect_ratio: f32,
}

impl Default for Temp2Config {
    fn default() -> Self {
        Self {
            aspect_ratio: TEMP_ASPECT_RATIO,
        }
    }
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
