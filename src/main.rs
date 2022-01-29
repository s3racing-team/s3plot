use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use eframe::egui::plot::{Legend, Line, Plot, Value, Values};
use eframe::egui::{
    menu, Align2, CentralPanel, Color32, CtxRef, Id, Key, LayerId, Order, Slider, TextStyle,
    TopBottomPanel, Ui,
};
use eframe::epi::{self, App, Frame};
use eframe::NativeOptions;
use s3plot::Data;
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "s3plot";

const POWER_ASPECT_RATIO: f32 = 0.005;
const SPEED_ASPECT_RATIO: f32 = 0.5;
const TORQUE_ASPECT_RATIO: f32 = 0.04;

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct PlotApp {
    current_path: Option<PathBuf>,
    selected_tab: Tab,
    power_aspect_ratio: f32,
    speed_aspect_ratio: f32,
    torque_aspect_ratio: f32,
    #[serde(skip)]
    data: Option<PlotData>,
}

struct PlotData {
    raw: Data,
    power: QuadValues,
    speed: QuadValues,
    torque_set: QuadValues,
    torque_real: QuadValues,
}

struct QuadValues {
    fl: Vec<Value>,
    fr: Vec<Value>,
    rl: Vec<Value>,
    rr: Vec<Value>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum Tab {
    Power,
    Speed,
    Torque,
}

impl Default for PlotApp {
    fn default() -> Self {
        Self {
            current_path: None,
            data: None,
            selected_tab: Tab::Power,
            power_aspect_ratio: POWER_ASPECT_RATIO,
            speed_aspect_ratio: SPEED_ASPECT_RATIO,
            torque_aspect_ratio: TORQUE_ASPECT_RATIO,
        }
    }
}

impl App for PlotApp {
    fn name(&self) -> &str {
        APP_NAME
    }

    fn setup(
        &mut self,
        _ctx: &eframe::egui::CtxRef,
        _frame: &Frame,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(s) = storage {
            if let Some(app) = epi::get_value(s, epi::APP_KEY) {
                *self = app;
            }
        }
        if let Some(p) = self.current_path.clone() {
            self.try_open(p);
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn update(&mut self, ctx: &CtxRef, _: &Frame) {
        if ctx.input().modifiers.ctrl && ctx.input().key_pressed(Key::O) {
            self.open_dialog();
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                menu::menu_button(ui, "File", |ui| {
                    if ui.button("Open").clicked() {
                        self.open_dialog();
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(d) = &self.data {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_tab, Tab::Power, "Power");
                    ui.selectable_value(&mut self.selected_tab, Tab::Speed, "Speed");
                    ui.selectable_value(&mut self.selected_tab, Tab::Torque, "Torque");
                    ui.add_space(40.0);

                    ui.label("aspect ratio");
                    match self.selected_tab {
                        Tab::Power => {
                            ratio_slider(ui, &mut self.power_aspect_ratio, POWER_ASPECT_RATIO);
                        }
                        Tab::Speed => {
                            ratio_slider(ui, &mut self.speed_aspect_ratio, SPEED_ASPECT_RATIO);
                        }
                        Tab::Torque => {
                            ratio_slider(ui, &mut self.torque_aspect_ratio, TORQUE_ASPECT_RATIO);
                        }
                    }
                    ui.add_space(40.0);

                    if let Some(p) = &self.current_path {
                        ui.label(format!("{}", p.display()));
                    }
                });

                match self.selected_tab {
                    Tab::Power => {
                        motor_plot(
                            ui,
                            Power,
                            self.power_aspect_ratio,
                            [Line::new(Values::from_values(d.power.fl.clone())).name("power")],
                            [Line::new(Values::from_values(d.power.fr.clone())).name("power")],
                            [Line::new(Values::from_values(d.power.rl.clone())).name("power")],
                            [Line::new(Values::from_values(d.power.rr.clone())).name("power")],
                        );
                    }
                    Tab::Speed => {
                        motor_plot(
                            ui,
                            Speed,
                            self.speed_aspect_ratio,
                            [Line::new(Values::from_values(d.speed.fl.clone())).name("speed")],
                            [Line::new(Values::from_values(d.speed.fr.clone())).name("speed")],
                            [Line::new(Values::from_values(d.speed.rl.clone())).name("speed")],
                            [Line::new(Values::from_values(d.speed.rr.clone())).name("speed")],
                        );
                    }
                    Tab::Torque => {
                        motor_plot(
                            ui,
                            Torque,
                            self.torque_aspect_ratio,
                            [
                                Line::new(Values::from_values(d.torque_set.fl.clone())).name("set"),
                                Line::new(Values::from_values(d.torque_real.fl.clone()))
                                    .name("real"),
                            ],
                            [
                                Line::new(Values::from_values(d.torque_set.fr.clone())).name("set"),
                                Line::new(Values::from_values(d.torque_real.fr.clone()))
                                    .name("real"),
                            ],
                            [
                                Line::new(Values::from_values(d.torque_set.rl.clone())).name("set"),
                                Line::new(Values::from_values(d.torque_real.rl.clone()))
                                    .name("real"),
                            ],
                            [
                                Line::new(Values::from_values(d.torque_set.rr.clone())).name("set"),
                                Line::new(Values::from_values(d.torque_real.rr.clone()))
                                    .name("real"),
                            ],
                        );
                    }
                }
            } else {
                ui.label("Open or drag and drop a file");
            }
        });

        self.detect_files_being_dropped(ctx);
    }
}

trait FormatLabel {
    fn format_label(name: &str, value: &Value) -> String;
}

struct Power;
impl FormatLabel for Power {
    fn format_label(_name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}s\np = {y}W")
    }
}

struct Speed;
impl FormatLabel for Speed {
    fn format_label(_name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}s\nv = {y}km/h")
    }
}

struct Torque;
impl FormatLabel for Torque {
    fn format_label(name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("{name}\nt = {x}s\nM = {y}Nm")
    }
}

fn ratio_slider(ui: &mut Ui, value: &mut f32, default_ratio: f32) {
    let min = default_ratio / 100.0;
    let max = default_ratio * 100.0;
    ui.add(Slider::new(value, min..=max).logarithmic(true));
}

fn motor_plot<T: FormatLabel, const COUNT: usize>(
    ui: &mut Ui,
    _: T,
    data_aspect: f32,
    lines_fl: [Line; COUNT],
    lines_fr: [Line; COUNT],
    lines_rl: [Line; COUNT],
    lines_rr: [Line; COUNT],
) {
    let h = ui.available_height() / 2.0
        - ui.fonts().row_height(TextStyle::Body)
        - ui.style().spacing.item_spacing.y;

    ui.columns(2, |uis| {
        let ui = &mut uis[0];
        ui.label("front left");
        Plot::new("fl_motor")
            .height(h)
            .data_aspect(data_aspect)
            .custom_label_func(move |n, v| T::format_label(n, v))
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in lines_fl {
                    ui.line(l);
                }
            })
            .response;
        ui.label("rear left");
        Plot::new("rl_motor")
            .height(h)
            .data_aspect(data_aspect)
            .custom_label_func(move |n, v| T::format_label(n, v))
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in lines_rl {
                    ui.line(l);
                }
            });

        let ui = &mut uis[1];
        ui.label("front right");
        Plot::new("fr_motor")
            .height(h)
            .data_aspect(data_aspect)
            .custom_label_func(move |n, v| T::format_label(n, v))
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in lines_fr {
                    ui.line(l);
                }
            });
        ui.label("rear right");
        Plot::new("rr_motor")
            .height(h)
            .data_aspect(data_aspect)
            .custom_label_func(move |n, v| T::format_label(n, v))
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in lines_rr {
                    ui.line(l);
                }
            });
    })
}

impl PlotApp {
    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.try_open(path);
        }
    }

    fn try_open(&mut self, path: PathBuf) {
        match Self::open(&path) {
            Ok(d) => {
                self.current_path = Some(path);
                let power = QuadValues {
                    fl: d.power_fl().collect(),
                    fr: d.power_fr().collect(),
                    rl: d.power_rl().collect(),
                    rr: d.power_rr().collect(),
                };
                let speed = QuadValues {
                    fl: d.speed_fl().collect(),
                    fr: d.speed_fr().collect(),
                    rl: d.speed_rl().collect(),
                    rr: d.speed_rr().collect(),
                };
                let torque_set = QuadValues {
                    fl: d.torque_set_fl().collect(),
                    fr: d.torque_set_fr().collect(),
                    rl: d.torque_set_rl().collect(),
                    rr: d.torque_set_rr().collect(),
                };
                let torque_real = QuadValues {
                    fl: d.torque_real_fl().collect(),
                    fr: d.torque_real_fr().collect(),
                    rl: d.torque_real_rl().collect(),
                    rr: d.torque_real_rr().collect(),
                };
                self.data = Some(PlotData {
                    raw: d,
                    power,
                    speed,
                    torque_set,
                    torque_real,
                });
            }
            Err(_) => {
                self.current_path = None;
                self.data = None;
            }
        }
    }

    fn open(path: &Path) -> anyhow::Result<Data> {
        let mut reader = BufReader::new(File::open(path)?);
        Ok(Data::read(&mut reader)?)
    }

    fn detect_files_being_dropped(&mut self, ctx: &CtxRef) {
        // Preview hovering files
        if !ctx.input().raw.hovered_files.is_empty() {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            let screen_rect = ctx.input().screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                "Dropping files",
                TextStyle::Body,
                Color32::WHITE,
            );
        }

        // Collect dropped files
        if !ctx.input().raw.dropped_files.is_empty() {
            if let Some(p) = ctx
                .input()
                .raw
                .dropped_files
                .first()
                .and_then(|f| f.path.as_ref())
            {
                self.try_open(p.clone());
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let app = PlotApp::default();
    let options = NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
