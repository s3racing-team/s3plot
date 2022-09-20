use egui::plot::{Line, PlotPoints};

use std::ops::Range;
use std::sync::Arc;

use cods::{Pos, UserFacing};
use egui::plot::{Legend, Plot};
use egui::style::Margin;
use egui::text::{LayoutJob, LayoutSection};
use egui::{
    Align, Button, CentralPanel, CollapsingHeader, Color32, Frame, Label, Layout, RichText,
    Rounding, ScrollArea, SidePanel, TextEdit, TextFormat, TextStyle, Ui, Vec2,
};
use serde::{Deserialize, Serialize};

use crate::app::{Job, PlotData, PlotValues};
use crate::eval::Expr;
use crate::util::{self, format_time};

const TAB_CROSS_WIDTH: f32 = 20.0;
const TAB_BUTTON_WIDTH: f32 = 80.0;
const TAB_WIDTH: f32 = TAB_BUTTON_WIDTH + TAB_CROSS_WIDTH;

const DEFAULT_ASPECT_RATIO: f32 = 0.1;
const ERROR_RED: Color32 = Color32::from_rgb(0xf0, 0x56, 0x56);

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub show_help: bool,
    pub selected_tab: usize,
    pub tabs: Vec<TabConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_help: true,
            selected_tab: 0,
            tabs: vec![TabConfig {
                name: "Tab 1".into(),
                aspect_ratio: DEFAULT_ASPECT_RATIO,
                plots: vec![
                    NamedPlot {
                        name: "1.".into(),
                        expr: Expr {
                            x: "time".into(),
                            y: "sin(time / PI) * 10.0".into(),
                        },
                    },
                    NamedPlot {
                        name: "2.".into(),
                        expr: Expr {
                            x: "time".into(),
                            y: "cos(time / PI - PI) * 10.0".into(),
                        },
                    },
                ],
            }],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TabConfig {
    pub name: String,
    pub aspect_ratio: f32,
    pub plots: Vec<NamedPlot>,
}

impl TabConfig {
    pub fn named(name: String) -> Self {
        Self {
            name,
            aspect_ratio: DEFAULT_ASPECT_RATIO,
            plots: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct NamedPlot {
    pub name: String,
    pub expr: Expr,
}

impl NamedPlot {
    fn new(name: String, expr: Expr) -> Self {
        Self { name, expr }
    }
}

pub fn tab_bar(ui: &mut Ui, data: &mut PlotData, cfg: &mut Config) {
    ui.horizontal(|ui| {
        let mut i = 0;
        while i < cfg.tabs.len() {
            let t = &mut cfg.tabs[i];
            let selected = cfg.selected_tab == i;

            let mut remove = false;

            let tab_fill = if selected {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().faint_bg_color
            };
            Frame::default()
                .rounding(Rounding::same(5.0))
                .fill(tab_fill)
                .show(ui, |ui| {
                    ui.set_width(TAB_WIDTH);

                    let mut rect = ui.available_rect_before_wrap();

                    if selected {
                        TextEdit::singleline(&mut t.name)
                            .desired_width(TAB_BUTTON_WIDTH - ui.spacing().button_padding.x * 2.0)
                            .frame(false)
                            .show(ui);
                    } else {
                        // tab text
                        ui.add_space(ui.spacing().button_padding.x);
                        ui.label(&t.name);

                        // clickable area
                        ui.allocate_ui_at_rect(rect, |ui| {
                            let resp = ui.add_sized(
                                Vec2::new(TAB_BUTTON_WIDTH, ui.available_height()),
                                Button::new("").frame(false),
                            );
                            if resp.clicked() {
                                cfg.selected_tab = i;
                            }
                        });
                    }

                    *rect.left_mut() += TAB_BUTTON_WIDTH;

                    // clickable area
                    ui.allocate_ui_at_rect(rect, |ui| {
                        let resp = ui.add_sized(
                            Vec2::new(TAB_CROSS_WIDTH, ui.available_height()),
                            Button::new(" 🗙 ").frame(false),
                        );
                        remove = resp.clicked();
                    });
                });

            if remove && cfg.tabs.len() > 1 {
                cfg.tabs.remove(i);
                data.plots.remove(i);

                if cfg.selected_tab > i || cfg.selected_tab == cfg.tabs.len() {
                    cfg.selected_tab -= 1;
                }
            } else {
                i += 1;
            }
        }

        if ui
            .add(Button::new(" + ").fill(ui.visuals().faint_bg_color))
            .clicked()
        {
            cfg.tabs
                .push(TabConfig::named(format!("Tab {}", cfg.tabs.len() + 1)));
            data.plots.push(Vec::new());
        }

        util::ratio_slider(
            ui,
            &mut cfg.tabs[cfg.selected_tab].aspect_ratio,
            DEFAULT_ASPECT_RATIO,
            1000.0,
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.toggle_value(&mut cfg.show_help, "?");
        });
    });
}

pub fn tab_plot(ui: &mut Ui, data: &mut PlotData, cfg: &mut Config) {
    let panel_fill = if ui.style().visuals.dark_mode {
        Color32::from_gray(0x20)
    } else {
        Color32::from_gray(0xf0)
    };
    SidePanel::left("expressions")
        .resizable(true)
        .default_width(350.0)
        .frame(Frame {
            inner_margin: Margin::same(6.0),
            rounding: Rounding::same(5.0),
            fill: panel_fill,
            ..Default::default()
        })
        .show_inside(ui, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    sidebar(ui, data, cfg);
                });
        });

    if cfg.show_help {
        SidePanel::right("help")
            .resizable(true)
            .default_width(300.0)
            .frame(Frame {
                inner_margin: Margin::same(6.0),
                rounding: Rounding::same(5.0),
                fill: panel_fill,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        CollapsingHeader::new(
                            RichText::new("Variables").text_style(TextStyle::Heading),
                        )
                        .default_open(true)
                        .show(ui, |ui| {
                            for s in data.streams.iter() {
                                for e in s.entries.iter() {
                                    ui.label(&e.name);
                                }
                                ui.add_space(10.0);
                            }
                        });
                    });
            });
    }

    CentralPanel::default()
        .frame(Frame::none())
        .show_inside(ui, |ui| {
            let tab_cfg = &mut cfg.tabs[cfg.selected_tab];

            Plot::new("custom")
                .data_aspect(tab_cfg.aspect_ratio)
                .label_formatter(|_, v| {
                    let x = format_time(v.x);
                    let y = (v.y * 1000.0).round() / 1000.0;
                    format!("t = {x}\ny = {y}")
                })
                .legend(Legend::default())
                .show(ui, |ui| {
                    for (values, p) in data.plots[cfg.selected_tab]
                        .iter_mut()
                        .zip(tab_cfg.plots.iter())
                    {
                        if let PlotValues::Job(j) = values {
                            if j.is_done() {
                                let job = std::mem::replace(values, PlotValues::empty());
                                *values = PlotValues::Result(job.into_job().unwrap().join());
                            } else {
                                ui.ctx().request_repaint();
                            }
                        }

                        match values {
                            PlotValues::Result(Ok(d)) if !d.is_empty() => {
                                ui.line(Line::new(PlotPoints::Owned(d.clone())).name(&p.name));
                            }
                            _ => ui.line(Line::new([0.0, f64::NAN]).name(&p.name)),
                        }
                    }
                });
        });
}

fn sidebar(ui: &mut Ui, data: &mut PlotData, cfg: &mut Config) {
    let tab_cfg = &mut cfg.tabs[cfg.selected_tab];
    let mut i = 0;
    while i < tab_cfg.plots.len() {
        let p = &mut tab_cfg.plots[i];
        let d = &data.plots[cfg.selected_tab][i];
        let input = expr_inputs(ui, p, d);

        if input.removed {
            tab_cfg.plots.remove(i);
            let _ = data.plots[cfg.selected_tab].remove(i);
        } else {
            if input.x_changed || input.y_changed {
                data.plots[cfg.selected_tab][i] =
                    PlotValues::Job(Job::start(p.expr.clone(), Arc::clone(&data.streams)));
            }
            i += 1;
        }
    }

    ui.horizontal(|ui| {
        if ui.button(" + ").clicked() {
            tab_cfg.plots.push(NamedPlot::new(
                format!("{}.", i + 1),
                Expr::new("time".into(), "".into()),
            ));
            data.plots[cfg.selected_tab].push(PlotValues::Result(Ok(Vec::new())));
        }

        ui.menu_button("...", |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.allocate_ui(Vec2::new(300.0, 500.0), |ui| {
                    for e in data.streams.iter().flat_map(|s| s.entries.iter()) {
                        if ui.button(&e.name).clicked() {
                            let plot = NamedPlot::new(
                                e.name.clone(),
                                Expr::new("time".into(), e.name.clone()),
                            );
                            data.plots[cfg.selected_tab].push(PlotValues::Job(Job::start(
                                plot.expr.clone(),
                                Arc::clone(&data.streams),
                            )));
                            tab_cfg.plots.push(plot);

                            ui.close_menu();
                        }
                    }
                });
            });
        });
    });
}

struct ExprInput {
    removed: bool,
    x_changed: bool,
    y_changed: bool,
}

fn expr_inputs(ui: &mut Ui, p: &mut NamedPlot, c: &PlotValues) -> ExprInput {
    let removed = ui.horizontal(|ui| {
        let r = ui.button(" − ").clicked();
        let width = ui.available_width() - ui.spacing().interact_size.x;
        TextEdit::singleline(&mut p.name)
            .desired_width(width)
            .frame(false)
            .show(ui);

        if let PlotValues::Job(_) = c {
            ui.spinner();
        }
        r
    });

    let mut x_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
        let mut layout_job = match c {
            PlotValues::Result(Err(e)) => match &e.x {
                Some(e) => mark_errors(string, e),
                None => LayoutJob::single_section(string.to_string(), TextFormat::default()),
            },
            _ => LayoutJob::single_section(string.to_string(), TextFormat::default()),
        };
        layout_job.wrap.max_width = wrap_width;
        ui.fonts().layout_job(layout_job)
    };
    let x_changed = ui.horizontal(|ui| {
        ui.add_sized(
            Vec2::new(20.0, 10.0),
            Label::new(RichText::new(" X ").monospace()),
        );
        ui.add(
            TextEdit::multiline(&mut p.expr.x)
                .desired_width(ui.available_width())
                .desired_rows(1)
                .font(TextStyle::Monospace)
                .layouter(&mut x_layouter),
        )
        .changed()
    });
    if let PlotValues::Result(Err(e)) = c {
        if let Some(e) = &e.x {
            ui.colored_label(ERROR_RED, e.to_string());
        }
    }

    let mut y_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
        let mut layout_job = match c {
            PlotValues::Result(Err(e)) => match &e.y {
                Some(e) => mark_errors(string, e),
                None => LayoutJob::single_section(string.to_string(), TextFormat::default()),
            },
            _ => LayoutJob::single_section(string.to_string(), TextFormat::default()),
        };
        layout_job.wrap.max_width = wrap_width;
        ui.fonts().layout_job(layout_job)
    };
    let y_changed = ui.horizontal(|ui| {
        ui.add_sized(
            Vec2::new(20.0, 10.0),
            Label::new(RichText::new(" Y ").monospace()),
        );
        ui.add(
            TextEdit::multiline(&mut p.expr.y)
                .desired_width(ui.available_width())
                .desired_rows(1)
                .font(TextStyle::Monospace)
                .layouter(&mut y_layouter),
        )
        .changed()
    });
    if let PlotValues::Result(Err(e)) = c {
        if let Some(e) = &e.y {
            ui.colored_label(ERROR_RED, e.to_string());
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
                color: ERROR_RED,
            },
            ..Default::default()
        },
    }
}
