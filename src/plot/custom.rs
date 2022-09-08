use std::ops::Range;
use std::sync::Arc;

use cods::{Pos, UserFacing};
use egui::plot::{Legend, Line, Plot, Value, Values};
use egui::style::Margin;
use egui::text::{LayoutJob, LayoutSection};
use egui::{
    CentralPanel, Color32, Frame, Label, RichText, Rounding, ScrollArea, SidePanel, TextEdit,
    TextFormat, TextStyle, Ui,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::app::{CustomValues, Job, PlotData};
use crate::eval::{Expr, Var};
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
    let panel_fill = if ui.style().visuals.dark_mode {
        Color32::from_gray(0x20)
    } else {
        Color32::from_gray(0xf0)
    };
    SidePanel::left("side_panel")
        .resizable(true)
        .frame(Frame {
            inner_margin: Margin::same(6.0),
            rounding: Rounding::same(5.0),
            fill: panel_fill,
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
            Plot::new("custom")
                .data_aspect(cfg.aspect_ratio)
                .label_formatter(|_, v| {
                    let x = format_time(v.x);
                    let y = (v.y * 1000.0).round() / 1000.0;
                    format!("t = {x}\ny = {y}")
                })
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (c, p) in data.custom.iter_mut().zip(cfg.plots.iter()) {
                        if let CustomValues::Job(j) = c {
                            if j.is_done() {
                                let job = std::mem::replace(c, CustomValues::empty());
                                *c = CustomValues::Result(job.as_job().unwrap().join());
                            } else {
                                ui.ctx().request_repaint();
                            }
                        }

                        match c {
                            CustomValues::Result(Ok(d)) => ui.line(line(d.clone()).name(&p.name)),
                            _ => ui.line(line(vec![Value::new(0.0, f64::NAN)]).name(&p.name)),
                        }
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
            let _ = data.custom.remove(i);
        } else {
            if input.x_changed || input.y_changed {
                data.custom[i] = CustomValues::Job(Job::start(
                    p.expr.clone(),
                    Arc::clone(&data.raw_data),
                    Arc::clone(&data.raw_temp),
                ));
            }
            i += 1;
        }
    }

    if ui.button(" + ").clicked() {
        cfg.plots.push(CustomPlot::named(format!("{}.", i + 1)));
        data.custom.push(CustomValues::Result(Ok(Vec::new())));
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

fn expr_inputs(ui: &mut Ui, p: &mut CustomPlot, c: &CustomValues) -> ExprInput {
    let removed = ui.horizontal(|ui| {
        let r = ui.button(" − ").clicked();
        ui.add(
            TextEdit::singleline(&mut p.name)
                .desired_width(ui.available_width())
                .frame(false),
        );
        // if let CustomValues::Job(_) = c {
        //     ui.spinner();
        // }
        r
    });

    let mut x_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
        let mut layout_job = match c {
            CustomValues::Result(Err(e)) => match &e.x {
                Some(e) => mark_errors(string, e),
                None => LayoutJob::single_section(string.to_string(), TextFormat::default()),
            },
            _ => LayoutJob::single_section(string.to_string(), TextFormat::default()),
        };
        layout_job.wrap.max_width = wrap_width;
        ui.fonts().layout_job(layout_job)
    };
    let x_changed = ui.horizontal(|ui| {
        ui.add(Label::new(RichText::new(" X ").monospace()));
        ui.add(
            TextEdit::multiline(&mut p.expr.x)
                .desired_width(ui.available_width())
                .desired_rows(1)
                .code_editor()
                .layouter(&mut x_layouter),
        )
        .changed()
    });
    if let CustomValues::Result(Err(e)) = c {
        if let Some(e) = &e.x {
            ui.colored_label(RED, e.to_string());
        }
    }

    let mut y_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
        let mut layout_job = match c {
            CustomValues::Result(Err(e)) => match &e.y {
                Some(e) => mark_errors(string, e),
                None => LayoutJob::single_section(string.to_string(), TextFormat::default()),
            },
            _ => LayoutJob::single_section(string.to_string(), TextFormat::default()),
        };
        layout_job.wrap.max_width = wrap_width;
        ui.fonts().layout_job(layout_job)
    };
    let y_changed = ui.horizontal(|ui| {
        ui.add(Label::new(RichText::new(" Y ").monospace()));
        ui.add(
            TextEdit::multiline(&mut p.expr.y)
                .desired_width(ui.available_width())
                .desired_rows(1)
                .code_editor()
                .layouter(&mut y_layouter),
        )
        .changed()
    });
    if let CustomValues::Result(Err(e)) = c {
        if let Some(e) = &e.y {
            ui.colored_label(RED, e.to_string());
        }
    }

    ui.add_space(10.0);

    ExprInput {
        removed: removed.inner,
        x_changed: x_changed.inner,
        y_changed: y_changed.inner,
    }
}

fn mark_errors(input: &str, error: &cods::Error) -> LayoutJob {
    let spans = error.spans();

    let mut sections = Vec::new();
    let mut pos = Pos::new(0, 0);
    let mut range = 0..input.len();
    let mut errors = 0;
    for (i, c) in input.char_indices() {
        for s in spans.iter() {
            if s.start == pos {
                if errors == 0 && i != 0 {
                    range.end = i;
                    sections.push(normal_section(range.clone()));
                    range.start = i;
                }
                errors += 1;
            }
        }
        for s in spans.iter() {
            if s.end == pos {
                errors -= 1;
                if errors == 0 {
                    range.end = i;
                    sections.push(error_section(range.clone()));
                    range.start = i;
                }
            }
        }

        match c {
            '\n' => {
                pos.line += 1;
                pos.col = 0;
            }
            _ => pos.col += 1,
        }
    }

    if sections.is_empty() || sections.last().unwrap().byte_range.end < input.len() {
        range.end = input.len();
        if errors == 0 {
            sections.push(normal_section(range));
        } else {
            sections.push(error_section(range));
        }
    }

    LayoutJob {
        text: input.to_string(),
        sections,
        ..Default::default()
    }
}

fn normal_section(range: Range<usize>) -> LayoutSection {
    LayoutSection {
        leading_space: 0.0,
        byte_range: range,
        format: TextFormat::default(),
    }
}

fn error_section(range: Range<usize>) -> LayoutSection {
    LayoutSection {
        leading_space: 0.0,
        byte_range: range,
        format: TextFormat {
            underline: egui::Stroke {
                width: 2.0,
                color: RED,
            },
            ..Default::default()
        },
    }
}
