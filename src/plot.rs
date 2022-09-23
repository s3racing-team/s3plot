use std::fmt::Write;
use std::ops::Range;
use std::sync::Arc;

use cods::{BuiltinFun, DataType, Pos, UserFacing};
use egui::plot::{Legend, Line, Plot, PlotPoints};
use egui::style::Margin;
use egui::text::{LayoutJob, LayoutSection};
use egui::{
    Align, Button, CentralPanel, CollapsingHeader, Color32, Frame, Key, Label, Layout, Modifiers,
    RichText, Rounding, ScrollArea, SidePanel, TextEdit, TextFormat, TextStyle, Ui, Vec2,
    WidgetText,
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
const HL_YELLOW: Color32 = Color32::from_rgb(0xc0, 0xc0, 0x76);

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub show_help: bool,
    #[serde(skip)]
    pub search_help: String,
    pub selected_tab: usize,
    pub tabs: Vec<TabConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_help: true,
            search_help: "".into(),
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

pub fn add_tab(data: &mut PlotData, cfg: &mut Config) {
    cfg.tabs
        .push(TabConfig::named(format!("Tab {}", cfg.tabs.len() + 1)));
    data.plots.push(Vec::new());
    cfg.selected_tab = cfg.tabs.len() - 1;
}

pub fn remove_tab(data: &mut PlotData, cfg: &mut Config, tab: usize) -> bool {
    if cfg.tabs.len() == 1 {
        return false;
    }
    cfg.tabs.remove(tab);
    data.plots.remove(tab);

    if cfg.selected_tab > tab || cfg.selected_tab == cfg.tabs.len() {
        cfg.selected_tab -= 1;
    }

    true
}

pub fn select_next_tab(cfg: &mut Config) {
    cfg.selected_tab = (cfg.selected_tab + 1) % cfg.tabs.len()
}

pub fn select_prev_tab(cfg: &mut Config) {
    cfg.selected_tab = (cfg.tabs.len() + cfg.selected_tab - 1) % cfg.tabs.len()
}

pub fn add_plot(data: &mut PlotData, cfg: &mut Config, tab: usize, y_expr: String) {
    let plots = &mut cfg.tabs[tab].plots;
    let name = format!("{}.", plots.len());
    plots.push(NamedPlot::new(name, Expr::new("time".into(), y_expr)));
    data.plots[tab].push(PlotValues::Result(Ok(Vec::new())));
}

pub fn keybindings(ui: &mut Ui, data: &mut PlotData, cfg: &mut Config) {
    let mut input = ui.input_mut();
    if input.consume_key(Modifiers::CTRL, Key::T) {
        add_tab(data, cfg);
    }
    if input.consume_key(Modifiers::CTRL, Key::W) {
        let tab = cfg.selected_tab;
        remove_tab(data, cfg, tab);
    }

    if input.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::Tab)
        || input.consume_key(Modifiers::ALT, Key::ArrowLeft)
    {
        select_prev_tab(cfg);
    }
    if input.consume_key(Modifiers::CTRL, Key::Tab)
        || input.consume_key(Modifiers::ALT, Key::ArrowRight)
    {
        select_next_tab(cfg);
    }

    if input.consume_key(Modifiers::CTRL, Key::H) {
        cfg.show_help = !cfg.show_help;
    }
    if input.consume_key(Modifiers::CTRL, Key::N) {
        add_plot(data, cfg, cfg.selected_tab, "".into());
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

            if !(remove && remove_tab(data, cfg, i)) {
                i += 1;
            }
        }

        let resp = ui.add(Button::new(" + ").fill(ui.visuals().faint_bg_color));
        if resp.clicked() {
            add_tab(data, cfg);
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
                    input_sidebar(ui, data, cfg);
                });
        });

    if cfg.show_help {
        SidePanel::right("help")
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
                        help_sidebar(ui, data, cfg);
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

fn input_sidebar(ui: &mut Ui, data: &mut PlotData, cfg: &mut Config) {
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
            add_plot(data, cfg, cfg.selected_tab, "".into());
        }

        ui.menu_button("...", |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.allocate_ui(Vec2::new(300.0, 500.0), |ui| {
                    for i in 0..data.streams.len() {
                        for j in 0..data.streams[i].entries.len() {
                            let name = &data.streams[i].entries[j].name;
                            if ui.button(name).clicked() {
                                add_plot(data, cfg, cfg.selected_tab, name.clone());

                                ui.close_menu();
                            }
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
        let format = TextFormat {
            font_id: TextStyle::Monospace.resolve(ui.style()),
            ..Default::default()
        };
        let mut layout_job = match c {
            PlotValues::Result(Err(e)) => match &e.x {
                Some(e) => mark_errors(string, e, format),
                None => LayoutJob::single_section(string.to_string(), format),
            },
            _ => LayoutJob::single_section(string.to_string(), format),
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
        let format = TextFormat {
            font_id: TextStyle::Monospace.resolve(ui.style()),
            ..Default::default()
        };
        let mut layout_job = match c {
            PlotValues::Result(Err(e)) => match &e.y {
                Some(e) => mark_errors(string, e, format),
                None => LayoutJob::single_section(string.to_string(), format),
            },
            _ => LayoutJob::single_section(string.to_string(), format),
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

fn mark_errors(input: &str, error: &cods::Error, format: TextFormat) -> LayoutJob {
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
                    sections.push(normal_section(range.clone(), format.clone()));
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
                    sections.push(error_section(range.clone(), format.clone()));
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
            sections.push(normal_section(range, format));
        } else {
            sections.push(error_section(range, format));
        }
    }

    LayoutJob {
        text: input.to_string(),
        sections,
        ..Default::default()
    }
}

fn normal_section(range: Range<usize>, format: TextFormat) -> LayoutSection {
    LayoutSection {
        leading_space: 0.0,
        byte_range: range,
        format,
    }
}

fn error_section(range: Range<usize>, format: TextFormat) -> LayoutSection {
    LayoutSection {
        leading_space: 0.0,
        byte_range: range,
        format: TextFormat {
            underline: egui::Stroke {
                width: 2.0,
                color: ERROR_RED,
            },
            ..format
        },
    }
}

fn help_sidebar(ui: &mut Ui, data: &mut PlotData, cfg: &mut Config) {
    let resp = TextEdit::singleline(&mut cfg.search_help)
        .desired_width(ui.available_width())
        .font(TextStyle::Monospace)
        .hint_text("Search...")
        .show(ui);

    if ui.input_mut().consume_key(Modifiers::CTRL, Key::F) {
        resp.response.request_focus();
    }

    let query = &cfg.search_help.to_lowercase();

    CollapsingHeader::new(RichText::new("Variables").text_style(TextStyle::Heading))
        .default_open(true)
        .show(ui, |ui| {
            for s in data.streams.iter() {
                let mut one_shown = false;
                for e in s.entries.iter() {
                    one_shown |= highlight_matches(ui, &e.name, query);
                }
                if one_shown {
                    ui.add_space(10.0);
                }
            }
        });

    CollapsingHeader::new(RichText::new("Functions").text_style(TextStyle::Heading))
        .default_open(true)
        .show(ui, |ui| {
            for f in BuiltinFun::members() {
                let signatures: &[(_, _)] = match f {
                    BuiltinFun::Pow => &cods::POW_SIGNATURES,
                    BuiltinFun::Ln => &cods::LN_SIGNATURES,
                    BuiltinFun::Log => &cods::LOG_SIGNATURES,
                    BuiltinFun::Sqrt => &cods::SQRT_SIGNATURES,
                    BuiltinFun::Ncr => &cods::NCR_SIGNATURES,
                    BuiltinFun::ToDeg => &cods::TO_DEG_SIGNATURES,
                    BuiltinFun::ToRad => &cods::TO_RAD_SIGNATURES,
                    BuiltinFun::Sin => &cods::SIN_SIGNATURES,
                    BuiltinFun::Cos => &cods::COS_SIGNATURES,
                    BuiltinFun::Tan => &cods::TAN_SIGNATURES,
                    BuiltinFun::Asin => &cods::ASIN_SIGNATURES,
                    BuiltinFun::Acos => &cods::ACOS_SIGNATURES,
                    BuiltinFun::Atan => &cods::ATAN_SIGNATURES,
                    BuiltinFun::Gcd => &cods::GCD_SIGNATURES,
                    BuiltinFun::Min => &cods::MIN_SIGNATURES,
                    BuiltinFun::Max => &cods::MAX_SIGNATURES,
                    BuiltinFun::Clamp => &cods::CLAMP_SIGNATURES,
                    BuiltinFun::Abs => &cods::ABS_SIGNATURES,
                    BuiltinFun::Print => &cods::PRINT_SIGNATURES,
                    BuiltinFun::Println => &cods::PRINTLN_SIGNATURES,
                    BuiltinFun::Spill => continue,
                    BuiltinFun::SpillLocal => continue,
                    BuiltinFun::Assert => &cods::ASSERT_SIGNATURES,
                    BuiltinFun::AssertEq => &cods::ASSERT_EQ_SIGNATURES,
                };

                let mut one_shown = false;
                for (_, s) in signatures {
                    let mut text = format!("{f}(");
                    if let Some((first, others)) = s.params.split_first() {
                        let _ = write!(text, "{first}");
                        for d in others {
                            let _ = write!(text, ", {d}");
                        }
                    }

                    match s.repetition {
                        cods::Repetition::One => (),
                        cods::Repetition::ZeroOrMore => {
                            let _ = write!(text, "..");
                        }
                        cods::Repetition::OneOrMore => {
                            let _ = write!(text, "...");
                        }
                    }

                    let _ = write!(text, ")");

                    if s.return_type != DataType::Unit {
                        let _ = write!(text, " -> {}", s.return_type);
                    }

                    one_shown |= highlight_matches(ui, &text, query);
                }
                if one_shown {
                    ui.add_space(10.0);
                }
            }
        });
}

fn highlight_matches(ui: &mut Ui, text: &str, query: &str) -> bool {
    if query.is_empty() {
        ui.label(WidgetText::LayoutJob(LayoutJob {
            text: text.into(),
            sections: vec![LayoutSection {
                byte_range: 0..text.len(),
                format: TextFormat {
                    font_id: TextStyle::Monospace.resolve(ui.style()),
                    color: ui.visuals().text_color(),
                    ..Default::default()
                },
                leading_space: 0.0,
            }],
            ..Default::default()
        }));
    } else {
        if let Some(pos) = text.to_lowercase().find(query) {
            ui.label(WidgetText::LayoutJob(LayoutJob {
                text: text.into(),
                sections: vec![
                    LayoutSection {
                        byte_range: 0..pos,
                        format: TextFormat {
                            font_id: TextStyle::Monospace.resolve(ui.style()),
                            color: ui.visuals().text_color(),
                            ..Default::default()
                        },
                        leading_space: 0.0,
                    },
                    LayoutSection {
                        byte_range: pos..pos + query.len(),
                        format: TextFormat {
                            font_id: TextStyle::Monospace.resolve(ui.style()),
                            color: ui.visuals().text_color(),
                            background: HL_YELLOW,
                            ..Default::default()
                        },
                        leading_space: 0.0,
                    },
                    LayoutSection {
                        byte_range: pos + query.len()..text.len(),
                        format: TextFormat {
                            font_id: TextStyle::Monospace.resolve(ui.style()),
                            color: ui.visuals().text_color(),
                            ..Default::default()
                        },
                        leading_space: 0.0,
                    },
                ],
                ..Default::default()
            }));
        } else {
            return false;
        }
    }

    true
}
