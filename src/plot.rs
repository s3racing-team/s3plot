use std::fmt::Write;
use std::ops::Range;
use std::sync::Arc;

use cods::{BuiltinConst, BuiltinFun, DataType, Pos, SignatureKind, UserFacing};
use egui::emath::TSTransform;
use egui::text::{LayoutJob, LayoutSection};
use egui::{
    Align, Button, CentralPanel, CollapsingHeader, Color32, CursorIcon, Frame, Id, Key, Label,
    LayerId, Layout, Margin, Modifiers, Order, Pos2, Rect, RichText, Rounding, ScrollArea, Sense,
    SidePanel, TextEdit, TextFormat, TextStyle, Ui, Vec2, WidgetText,
};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use serde::{Deserialize, Serialize};

use crate::app::{Job, PlotData, PlotValues};
use crate::eval::Expr;
use crate::util::{self, format_time};

const TAB_CROSS_WIDTH: f32 = 20.0;
const TAB_BUTTON_WIDTH: f32 = 80.0;

const PLOT_FRAME_PADDING: f32 = 2.0;

const TEXT_EDIT_MARGIN_X: f32 = 4.0;
const TEXT_EDIT_MARGIN_Y: f32 = 2.0;

const DEFAULT_ASPECT_RATIO: f32 = 0.1;
const ERROR_RED: Color32 = Color32::from_rgb(0xf0, 0x56, 0x56);

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub show_help: bool,
    #[serde(skip)]
    pub search_help: String,
    pub selected_tab: usize,
    pub tabs: Vec<TabConfig>,
    #[serde(skip)]
    pub dragged_tab: Option<(usize, Pos2)>,
    #[serde(skip)]
    pub dragged_plot: Option<(usize, Pos2)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_help: true,
            search_help: "".into(),
            selected_tab: 0,
            tabs: vec![TabConfig::new(
                "Tab 1".into(),
                DEFAULT_ASPECT_RATIO,
                vec![
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
            )],
            dragged_tab: None,
            dragged_plot: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TabConfig {
    pub name: String,
    pub id: u64,
    pub aspect_ratio: f32,
    pub plots: Vec<NamedPlot>,
    #[serde(skip)]
    pub init_plot: bool,
}

impl TabConfig {
    pub fn new(name: String, aspect_ratio: f32, plots: Vec<NamedPlot>) -> Self {
        Self {
            name,
            id: rand::random(),
            aspect_ratio,
            plots,
            init_plot: false,
        }
    }

    pub fn named(name: String) -> Self {
        Self::new(name, DEFAULT_ASPECT_RATIO, Vec::new())
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

pub fn move_tab(data: &mut PlotData, cfg: &mut Config, from: usize, to: usize) {
    let selected_tab = cfg.selected_tab;
    if from < to {
        for i in from..to {
            cfg.tabs.swap(i, i + 1);
            data.plots.swap(i, i + 1);
        }

        if selected_tab > from && selected_tab <= to {
            cfg.selected_tab -= 1;
        }
    } else {
        for i in (to..from).rev() {
            cfg.tabs.swap(i + 1, i);
            data.plots.swap(i + 1, i);
        }

        if selected_tab < from && selected_tab >= to {
            cfg.selected_tab += 1;
        }
    }

    if selected_tab == from {
        cfg.selected_tab = to;
    }
}

pub fn select_next_tab(cfg: &mut Config) {
    cfg.selected_tab = (cfg.selected_tab + 1) % cfg.tabs.len()
}

pub fn select_prev_tab(cfg: &mut Config) {
    cfg.selected_tab = (cfg.tabs.len() + cfg.selected_tab - 1) % cfg.tabs.len()
}

pub fn add_plot(data: &mut PlotData, cfg: &mut Config, plot: NamedPlot, eval: bool) {
    let tab = cfg.selected_tab;
    let plots = &mut cfg.tabs[tab].plots;

    if eval {
        let job = Job::start(plot.expr.clone(), Arc::clone(&data.streams));
        data.plots[tab].push(PlotValues::Job(job));
    } else {
        data.plots[tab].push(PlotValues::Result(Ok(Vec::new())));
    }
    plots.push(plot);
}

pub fn move_plot(data: &mut PlotData, cfg: &mut Config, from: usize, to: usize) {
    let tab = cfg.selected_tab;
    if from < to {
        for i in from..to {
            cfg.tabs[tab].plots.swap(i, i + 1);
            data.plots[tab].swap(i, i + 1);
        }
    } else {
        for i in (to..from).rev() {
            cfg.tabs[tab].plots.swap(i + 1, i);
            data.plots[tab].swap(i + 1, i);
        }
    }
}

pub fn keybindings(ui: &mut Ui, data: &mut PlotData, cfg: &mut Config) {
    ui.input_mut(|input| {
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
        // Open help sidebar so the search bar can be focused
        if !cfg.show_help
            && input.modifiers.matches_exact(Modifiers::CTRL)
            && input.key_pressed(Key::F)
        {
            cfg.show_help = true;
        }

        if input.consume_key(Modifiers::CTRL, Key::N) {
            let name = format!("{}.", cfg.tabs[cfg.selected_tab].plots.len() + 1);
            add_plot(
                data,
                cfg,
                NamedPlot::new(name, Expr::new("time", "")),
                false,
            );
        }
    });
}

#[inline]
fn tab_button_width() -> f32 {
    TAB_BUTTON_WIDTH + 2.0 * TEXT_EDIT_MARGIN_X
}

#[inline]
fn tab_width(ui: &Ui) -> f32 {
    tab_button_width() + ui.spacing().item_spacing.x + TAB_CROSS_WIDTH
}

pub fn tab_bar(ui: &mut Ui, data: &mut PlotData, cfg: &mut Config) {
    ui.horizontal(|ui| {
        let tab_width = tab_width(ui);
        let tab_spacing = ui.spacing().item_spacing.x;

        let mut i = 0;
        while i < cfg.tabs.len() {
            // check if the tab was clicked or dragged
            let tab_pos = ui.cursor().min;
            let size = Vec2::new(tab_width, ui.spacing().interact_size.y);
            let rect = Rect::from_min_size(tab_pos, size);
            let resp = ui.allocate_rect(rect, Sense::click_and_drag());
            let pointer_pos = ui.ctx().pointer_interact_pos();

            if resp.drag_started() {
                if let Some(p) = pointer_pos {
                    cfg.dragged_tab = Some((i, p));
                }
            }

            let selected = cfg.selected_tab == i;
            let mut edit_name = false;
            let mut removed = false;
            if resp.clicked() {
                if let Some(clicked_pos) = pointer_pos {
                    let relative_pos = clicked_pos - tab_pos;
                    let close_button_start = tab_button_width() + tab_spacing;
                    // FIXME: doesn't work
                    if relative_pos.x < close_button_start {
                        if selected {
                            edit_name = true;
                        } else {
                            cfg.selected_tab = i;
                        }
                    } else {
                        removed = true;
                    }
                }
            }

            // move the tab if it was dropped
            if ui.input(|i| i.pointer.any_released()) {
                match (pointer_pos, cfg.dragged_tab) {
                    (Some(pointer_pos), Some((dragged_idx, grab_pos))) if dragged_idx == i => {
                        let distance = pointer_pos.x - grab_pos.x;
                        let tab_distance = tab_width + tab_spacing;
                        let moved = (distance / tab_distance) as isize;
                        let idx = (i as isize + moved).clamp(0, cfg.tabs.len() as isize - 1);
                        move_tab(data, cfg, i, idx as usize);
                        cfg.dragged_tab = None;
                    }
                    _ => (),
                }
            }

            // actually draw tab
            let t = &mut cfg.tabs[i];
            ui.allocate_ui_at_rect(rect, |ui| match (pointer_pos, cfg.dragged_tab) {
                (Some(pointer_pos), Some((dragged_idx, grab_pos))) if dragged_idx == i => {
                    let id = Id::new("tab").with(i);
                    let layer_id = LayerId::new(Order::Tooltip, id);
                    let distance = Vec2::new(pointer_pos.x - grab_pos.x, 0.0);

                    ui.with_layer_id(layer_id, |ui| {
                        draw_tab(ui, &mut t.name, selected, edit_name)
                    });
                    let transform = TSTransform::new(distance, 1.0);
                    ui.ctx().transform_layer_shapes(layer_id, transform);
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);
                }
                _ => {
                    draw_tab(ui, &mut t.name, selected, edit_name);
                }
            });

            if !(removed && remove_tab(data, cfg, i)) {
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

fn draw_tab(ui: &mut Ui, name: &mut String, selected: bool, edit_name: bool) {
    let tab_fill = if selected {
        ui.visuals().selection.bg_fill
    } else {
        ui.visuals().faint_bg_color
    };

    Frame::default()
        .rounding(Rounding::same(5.0))
        .fill(tab_fill)
        .show(ui, |ui| {
            ui.set_width(tab_width(ui));

            let edit_resp = TextEdit::singleline(name)
                .desired_width(TAB_BUTTON_WIDTH)
                .frame(false)
                .interactive(selected)
                .show(ui);

            if edit_name {
                edit_resp.response.request_focus();
            }

            ui.add_sized(
                Vec2::new(TAB_CROSS_WIDTH, ui.available_height()),
                Button::new("🗙").frame(false),
            );
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
                help_sidebar(ui, data, cfg);
            });
    }

    CentralPanel::default()
        .frame(Frame::none())
        .show_inside(ui, |ui| {
            let tab_cfg = &mut cfg.tabs[cfg.selected_tab];

            Plot::new(tab_cfg.id)
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
    let plot_height = 3.0 * ui.spacing().interact_size.y
        + 2.0 * ui.spacing().item_spacing.y
        + 6.0 * TEXT_EDIT_MARGIN_Y
        + 2.0 * PLOT_FRAME_PADDING;
    let plot_spacing = ui.spacing().item_spacing.y;

    let mut i = 0;
    while i < cfg.tabs[cfg.selected_tab].plots.len() {
        let plot = &mut cfg.tabs[cfg.selected_tab].plots[i];
        let values = &data.plots[cfg.selected_tab][i];
        let pointer_pos = ui.ctx().pointer_interact_pos();

        let mut input = None;
        match (pointer_pos, cfg.dragged_plot) {
            (Some(pointer_pos), Some((dragged_idx, grab_pos))) if dragged_idx == i => {
                let id = Id::new("plot").with(i);
                let layer_id = LayerId::new(Order::Tooltip, id);
                let distance = Vec2::new(0.0, pointer_pos.y - grab_pos.y);

                ui.with_layer_id(layer_id, |ui| {
                    expr_inputs(ui, plot, values, i, &mut cfg.dragged_plot);
                });
                let transform = TSTransform::new(distance, 1.0);
                ui.ctx().transform_layer_shapes(layer_id, transform);
                // FIXME: doesn't work
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);
            }
            _ => {
                input = Some(expr_inputs(ui, plot, values, i, &mut cfg.dragged_plot));
            }
        };

        // move the plot if it was dropped
        if ui.input(|i| i.pointer.any_released()) {
            match (pointer_pos, cfg.dragged_plot) {
                (Some(pointer_pos), Some((dragged_idx, grab_pos))) if dragged_idx == i => {
                    let distance = pointer_pos.y - grab_pos.y;
                    let plot_distance = plot_height + plot_spacing;
                    let moved = (distance / plot_distance) as isize;
                    let len = cfg.tabs[cfg.selected_tab].plots.len();
                    let new_idx = (i as isize + moved).clamp(0, len as isize - 1);
                    move_plot(data, cfg, i, new_idx as usize);
                    cfg.dragged_plot = None;
                }
                _ => (),
            }
        }

        let tab_cfg = &mut cfg.tabs[cfg.selected_tab];
        let plot = &mut tab_cfg.plots[i];
        match input {
            Some(input) if input.removed => {
                tab_cfg.plots.remove(i);
                let _ = data.plots[cfg.selected_tab].remove(i);
            }
            Some(input) => {
                if input.x_changed || input.y_changed {
                    data.plots[cfg.selected_tab][i] =
                        PlotValues::Job(Job::start(plot.expr.clone(), Arc::clone(&data.streams)));
                }
                i += 1;
            }
            None => i += 1,
        }
    }

    ui.horizontal(|ui| {
        if ui.button(" + ").clicked() {
            let name = format!("{}.", cfg.tabs[cfg.selected_tab].plots.len() + 1);
            add_plot(
                data,
                cfg,
                NamedPlot::new(name, Expr::new("time", "")),
                false,
            );
        }

        ui.menu_button("...", |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.allocate_ui(Vec2::new(300.0, 500.0), |ui| {
                    for i in 0..data.streams.len() {
                        for j in 0..data.streams[i].entries.len() {
                            let name = &data.streams[i].entries[j].name;
                            if ui.button(name).clicked() {
                                let plot = NamedPlot::new(name.into(), Expr::new("time", name));
                                add_plot(data, cfg, plot, true);

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

fn expr_inputs(
    ui: &mut Ui,
    plot: &mut NamedPlot,
    values: &PlotValues,
    idx: usize,
    dragged_plot: &mut Option<(usize, Pos2)>,
) -> ExprInput {
    let plot_fill = match dragged_plot {
        Some((i, _)) if idx == *i => Color32::from_rgba_unmultiplied(0x80, 0x80, 0x80, 0x20),
        _ => Color32::TRANSPARENT,
    };
    let resp = Frame::default()
        .rounding(Rounding::same(3.0))
        .fill(plot_fill)
        .inner_margin(PLOT_FRAME_PADDING)
        .show(ui, |ui| {
            let removed = ui.horizontal(|ui| {
                let r = ui.add(Button::new(" − ").sense(Sense::click_and_drag()));
                let width = ui.available_width() - ui.spacing().interact_size.x;
                TextEdit::singleline(&mut plot.name)
                    .desired_width(width)
                    .frame(false)
                    .show(ui);

                if let PlotValues::Job(_) = values {
                    ui.spinner();
                }

                r.clicked()
            });

            let mut x_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let format = TextFormat {
                    font_id: TextStyle::Monospace.resolve(ui.style()),
                    ..Default::default()
                };
                let mut layout_job = match values {
                    PlotValues::Result(Err(e)) => match &e.x {
                        Some(e) => mark_errors(string, e, format),
                        None => LayoutJob::single_section(string.to_string(), format),
                    },
                    _ => LayoutJob::single_section(string.to_string(), format),
                };
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };
            let x_changed = ui.horizontal(|ui| {
                ui.add_sized(
                    Vec2::new(20.0, 10.0),
                    Label::new(RichText::new(" X ").monospace()),
                );
                ui.add(
                    TextEdit::multiline(&mut plot.expr.x)
                        .desired_width(ui.available_width())
                        .desired_rows(1)
                        .layouter(&mut x_layouter),
                )
                .changed()
            });
            if let PlotValues::Result(Err(e)) = values {
                if let Some(e) = &e.x {
                    ui.colored_label(ERROR_RED, e.to_string());
                }
            }

            let mut y_layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let format = TextFormat {
                    font_id: TextStyle::Monospace.resolve(ui.style()),
                    ..Default::default()
                };
                let mut layout_job = match values {
                    PlotValues::Result(Err(e)) => match &e.y {
                        Some(e) => mark_errors(string, e, format),
                        None => LayoutJob::single_section(string.to_string(), format),
                    },
                    _ => LayoutJob::single_section(string.to_string(), format),
                };
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };
            let y_changed = ui.horizontal(|ui| {
                ui.add_sized(
                    Vec2::new(20.0, 10.0),
                    Label::new(RichText::new(" Y ").monospace()),
                );
                ui.add(
                    TextEdit::multiline(&mut plot.expr.y)
                        .desired_width(ui.available_width())
                        .desired_rows(1)
                        .layouter(&mut y_layouter),
                )
                .changed()
            });
            if let PlotValues::Result(Err(e)) = values {
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
        });

    if dragged_plot.is_none() {
        let resp = resp.response.interact(Sense::drag());
        if resp.drag_started() {
            if let Some(pointer_pos) = resp.hover_pos() {
                *dragged_plot = Some((idx, pointer_pos));
            }
        }
    }

    resp.inner
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

    if ui.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::F)) {
        resp.response.request_focus();
    }

    let query = &cfg.search_help.to_lowercase();

    ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
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

            CollapsingHeader::new(RichText::new("Constants").text_style(TextStyle::Heading))
                .default_open(true)
                .show(ui, |ui| {
                    for c in BuiltinConst::members() {
                        let text = format!("{c}: {} = {}", c.data_type(), c.val());
                        highlight_matches(ui, &text, query);
                    }
                });

            CollapsingHeader::new(RichText::new("Datatypes").text_style(TextStyle::Heading))
                .default_open(true)
                .show(ui, |ui| {
                    for d in DataType::members() {
                        highlight_matches(ui, &d.to_string(), query);
                    }
                });

            CollapsingHeader::new(RichText::new("Functions").text_style(TextStyle::Heading))
                .default_open(true)
                .show(ui, |ui| {
                    for f in BuiltinFun::members() {
                        let signatures: &[(_, _)] = match f.signatures() {
                            SignatureKind::Normal(s) => s,
                            SignatureKind::Spill(_) => continue,
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
                            ui.add_space(5.0);
                        }
                    }
                });
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
    } else if let Some(pos) = text.to_lowercase().find(query) {
        let hl_color = if ui.style().visuals.dark_mode {
            Color32::from_rgb(0xfa, 0xc6, 0x26)
        } else {
            Color32::from_rgb(0xfa, 0xc6, 0x96)
        };
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
                        background: hl_color,
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

    true
}
