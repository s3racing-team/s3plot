use cods::UserFacing;
use egui::plot::{Legend, Plot};
use egui::style::Margin;
use egui::{
    CentralPanel, Color32, Frame, Label, RichText, Rounding, ScrollArea, SidePanel, TextEdit,
    TextStyle, Ui,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::app::{CustomValues, PlotData};
use crate::eval::{self, Expr, Var};
use crate::util::{self, format_time};

use super::line;

const CUSTOM_ASPECT_RATIO: f32 = 0.1;
const RED: Color32 = Color32::from_rgb(0xf0, 0x56, 0x56);

#[derive(Serialize, Deserialize)]
pub struct CustomConfig {
    pub aspect_ratio: f32,
    pub plots: Vec<CustomPlot>,
}

impl Default for CustomConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: CUSTOM_ASPECT_RATIO,
            plots: vec![
                CustomPlot {
                    name: "1.".into(),
                    expr: Expr {
                        x: "t".into(),
                        y: "sin(t / PI) * sqrt(abs(P_fl))".into(),
                    },
                },
                CustomPlot {
                    name: "2.".into(),
                    expr: Expr {
                        x: "t".into(),
                        y: "cos(t / PI - PI) * sqrt(abs(P_fl))".into(),
                    },
                },
            ],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CustomPlot {
    pub name: String,
    pub expr: Expr,
}

impl CustomPlot {
    fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            expr: Expr::default(),
        }
    }
}

pub fn custom_config(ui: &mut Ui, cfg: &mut CustomConfig) {
    util::ratio_slider(ui, &mut cfg.aspect_ratio, CUSTOM_ASPECT_RATIO, 1000.0);
}

pub fn custom_plot(ui: &mut Ui, data: &mut PlotData, cfg: &mut CustomConfig) {
    SidePanel::left("side_panel")
        .resizable(true)
        .frame(Frame {
            inner_margin: Margin::same(6.0),
            rounding: Rounding::same(5.0),
            fill: Color32::from_rgb(0x20, 0x20, 0x20),
            ..Default::default()
        })
        .show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                sidebar(ui, data, cfg);
            });
        });

    CentralPanel::default()
        .frame(Frame::none())
        .show_inside(ui, |ui| {
            Plot::new("rr_motor")
                .data_aspect(cfg.aspect_ratio)
                .label_formatter(|_, v| {
                    let x = format_time(v.x);
                    let y = (v.y * 1000.0).round() / 1000.0;
                    format!("t = {x}\ny = {y}")
                })
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (d, p) in data.custom.iter().zip(cfg.plots.iter()) {
                        ui.line(line(d.values.clone()).name(&p.name));
                    }
                });
        });
}

fn sidebar(ui: &mut Ui, data: &mut PlotData, cfg: &mut CustomConfig) {
    let mut i = 0;
    while i < cfg.plots.len() {
        let p = &mut cfg.plots[i];
        let d = &data.custom[i];
        let input = expr_inputs(ui, p, d);

        if input.removed {
            cfg.plots.remove(i);
            data.custom.remove(i);
        } else {
            if input.x_changed || input.y_changed {
                let r = eval::eval(&p.expr, &data.raw_data, &data.raw_temp);
                data.custom[i] = CustomValues::from_result(r);
            }
            i += 1;
        }
    }

    if ui.button(" + ").clicked() {
        cfg.plots.push(CustomPlot::named(format!("{}.", i + 1)));
        data.custom.push(CustomValues::default());
    }
    ui.add_space(10.0);

    ui.add(Label::new(
        RichText::new("Variables").text_style(TextStyle::Heading),
    ));
    for v in Var::iter() {
        ui.add(Label::new(RichText::new(v.to_string()).monospace()));
    }
}

struct ExprInput {
    removed: bool,
    x_changed: bool,
    y_changed: bool,
}

fn expr_inputs(ui: &mut Ui, p: &mut CustomPlot, d: &CustomValues) -> ExprInput {
    let removed = ui.horizontal(|ui| {
        let r = ui.button(" − ").clicked();
        ui.add(
            TextEdit::singleline(&mut p.name)
                .desired_width(ui.available_width())
                .frame(false),
        );
        r
    });

    let x_changed = ui.horizontal(|ui| {
        ui.add(Label::new(RichText::new(" X ").monospace()));
        ui.add(
            TextEdit::multiline(&mut p.expr.x)
                .desired_width(ui.available_width())
                .desired_rows(1)
                .font(TextStyle::Monospace),
        )
        .changed()
    });
    if let Some(e) = &d.error.x {
        ui.colored_label(RED, e.to_string());
    }

    let y_changed = ui.horizontal(|ui| {
        ui.add(Label::new(RichText::new(" Y ").monospace()));
        ui.add(
            TextEdit::multiline(&mut p.expr.y)
                .desired_width(ui.available_width())
                .desired_rows(1)
                .font(TextStyle::Monospace),
        )
        .changed()
    });
    if let Some(e) = &d.error.y {
        e.spans();
        ui.colored_label(RED, e.to_string());
    }

    ui.add_space(10.0);

    ExprInput {
        removed: removed.inner,
        x_changed: x_changed.inner,
        y_changed: y_changed.inner,
    }
}
