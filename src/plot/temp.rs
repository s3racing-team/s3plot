use derive_more::{Deref, DerefMut};
use egui::plot::{Legend, LinkedAxisGroup, Plot, PlotPoint, PlotUi};
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
            (line(&data.temp.fl), "temp"),
            (line(&data.room_temp.fl), "room temp"),
            (line(&data.heatsink_temp.fl), "heatsink temp"),
        ],
        [
            (line(&data.temp.fr), "temp"),
            (line(&data.room_temp.fr), "room temp"),
            (line(&data.heatsink_temp.fr), "heatsink temp"),
        ],
        [
            (line(&data.temp.rl), "temp"),
            (line(&data.room_temp.rl), "room temp"),
            (line(&data.heatsink_temp.rl), "heatsink temp"),
        ],
        [
            (line(&data.temp.rr), "temp"),
            (line(&data.room_temp.rr), "room temp"),
            (line(&data.heatsink_temp.rr), "heatsink temp"),
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

pub fn temp2_plot<'a>(ui: &mut Ui, data: &'a PlotData, cfg: &Temp2Config) {
    Plot::new("temp2")
        .data_aspect(cfg.aspect_ratio)
        .label_formatter(move |n, v| Temp1Config::format_label(n, v))
        .legend(Legend::default())
        .show(ui, |ui: &mut PlotUi<'a>| {
            ui.line(line(&data.ams_temp_max).name("ams temp max"));
            ui.line(line(&data.water_temp_converter).name("water temp converter"));
            ui.line(line(&data.water_temp_motor).name("water temp motor"));
        });
}
