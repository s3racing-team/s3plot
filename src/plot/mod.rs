use std::ops::{Deref, DerefMut};

use egui::plot::{Legend, Line, LinkedAxisGroup, Plot, PlotPoint, PlotPoints};
use egui::{TextStyle, Ui};
use serde::{Deserialize, Serialize};

use crate::util;

pub use custom::*;
pub use motor::*;
pub use temp::*;

mod custom;
mod motor;
mod temp;

const DEFAULT_GRID_MODE: bool = true;
const DEFAULT_LINKED: bool = true;

pub trait WheelPlotConfig: Deref<Target = WheelConfig> + DerefMut {
    const NAME: &'static str;
    const ASPECT_RATIO: f32;
    fn format_label(name: &str, val: &PlotPoint) -> String;
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

fn line(values: Vec<PlotPoint>) -> Line {
    Line::new(PlotPoints::Owned(values))
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
