use eframe::egui::plot::{Legend, Line, Plot, Values};
use eframe::egui::{Label, RichText, ScrollArea, TextEdit, TextStyle, Ui};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::app::PlotData;
use crate::eval::{self, Expr, Var};
use crate::util;

const CUSTOM_ASPECT_RATIO: f32 = 0.1;

#[derive(Serialize, Deserialize)]
pub struct CustomConfig {
    pub aspect_ratio: f32,
    pub exprs: Vec<Expr>,
}

impl Default for CustomConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: CUSTOM_ASPECT_RATIO,
            exprs: vec![Expr {
                x: "t".into(),
                y: "sin(t / PI) *  sqrt(P_fl) * 2^3".into(),
            }],
        }
    }
}

pub fn ratio_slider(ui: &mut Ui, cfg: &mut CustomConfig) {
    util::ratio_slider(ui, &mut cfg.aspect_ratio, CUSTOM_ASPECT_RATIO, 1000.0);
}

pub fn plot(ui: &mut Ui, data: &mut PlotData, cfg: &mut CustomConfig) {
    let h = ui.available_height();
    ui.horizontal_top(|ui| {
        ui.set_height(h);

        ui.vertical(|ui| {
            ui.add_space(ui.style().spacing.window_padding.y);

            ScrollArea::vertical().show(ui, |ui| {
                let mut i = 0;
                while i < cfg.exprs.len() {
                    let e = &mut cfg.exprs[i];
                    let removed = ui.horizontal(|ui| {
                        ui.label(format!("{}", i + 1));
                        ui.button(" − ").clicked()
                    });

                    let x_changed = ui.horizontal(|ui| {
                        ui.label("X");
                        ui.add(TextEdit::multiline(&mut e.x).desired_rows(1))
                            .changed()
                    });
                    let y_changed = ui.horizontal(|ui| {
                        ui.label("Y");
                        ui.add(TextEdit::multiline(&mut e.y).desired_rows(1))
                            .changed()
                    });
                    ui.add_space(10.0);

                    if removed.inner {
                        cfg.exprs.remove(i);
                        data.custom.remove(i);
                    } else {
                        if x_changed.inner || y_changed.inner {
                            data.custom[i] = eval::eval(&e, &data.raw).unwrap_or_default();
                        }
                        i += 1;
                    }
                }

                if ui.button(" + ").clicked() {
                    cfg.exprs.push(Expr::default());
                    data.custom.push(Vec::new());
                }
                ui.add_space(10.0);

                ui.add(Label::new(
                    RichText::new("Variables").text_style(TextStyle::Heading),
                ));

                for v in Var::iter() {
                    ui.label(v.to_string());
                }
            });
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
                for (i, d) in data.custom.iter().enumerate() {
                    ui.line(Line::new(Values::from_values(d.clone())).name(format!("{}", i + 1)));
                }
            });
    });
}
