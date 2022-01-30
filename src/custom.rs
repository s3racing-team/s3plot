use eframe::egui::plot::{Legend, Line, Plot, Values};
use eframe::egui::{Label, RichText, TextStyle, Ui};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::app::PlotData;
use crate::util;
use crate::eval::{self, Var};

const CUSTOM_ASPECT_RATIO: f32 = 0.1;

#[derive(Serialize, Deserialize)]
pub struct CustomConfig {
    pub aspect_ratio: f32,
    pub expr_x: String,
    pub expr_y: String,
}

impl Default for CustomConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: CUSTOM_ASPECT_RATIO,
            expr_x: String::from("t"),
            expr_y: String::from("sin(t / PI) *  sqrt(P_fl) * 2^3"),
        }
    }
}

pub fn ratio_slider(ui: &mut Ui, cfg: &mut CustomConfig) {
    util::ratio_slider(
        ui,
        &mut cfg.aspect_ratio,
        CUSTOM_ASPECT_RATIO,
        1000.0,
    );
}

pub fn plot(ui: &mut Ui, data: &mut PlotData, cfg: &mut CustomConfig) {
    let h = ui.available_height();
    ui.horizontal_top(|ui| {
        ui.set_height(h);

        ui.vertical(|ui| {
            ui.label("X-Axis");
            let x_changed = ui.text_edit_multiline(&mut cfg.expr_x).changed();

            ui.label("Y-Axis");
            let y_changed = ui.text_edit_multiline(&mut cfg.expr_y).changed();

            ui.add_space(20.0);

            if x_changed || y_changed {
                data.custom = eval::eval(&cfg.expr_x, &cfg.expr_y, &data.raw).unwrap_or_default();
            }

            ui.add(Label::new(
                RichText::new("Variables").text_style(TextStyle::Heading),
            ));

            for v in Var::iter() {
                ui.label(v.to_string());
            }
        });

        let h = ui.available_height();
        Plot::new("rr_motor")
            .height(h)
            .data_aspect(cfg.aspect_ratio)
            .custom_label_func(|_, v| {
                let x = (v.x * 1000.0).round() / 1000.0;
                let y = (v.y * 1000.0).round() / 1000.0;
                format!("t = {x}s\ny = {y}")
            })
            .legend(Legend::default())
            .show(ui, |ui| {
                ui.line(Line::new(Values::from_values(data.custom.clone())));
            });
    });
}
