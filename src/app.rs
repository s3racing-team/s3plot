use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::JoinHandle;

use egui::plot::PlotPoint;
use egui::{menu, Align2, CentralPanel, Color32, Key, RichText, TopBottomPanel, Ui, Vec2, Window};
use egui_extras::{Size, TableBuilder};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::data::{DataEntry, TempEntry, TimeStamped, Version};
use crate::eval::{self, Expr, ExprError};
use crate::fs::{Files, SelectableFile, SelectableFiles};
use crate::plot::{
    self, CustomConfig, PowerConfig, Temp1Config, Temp2Config, TorqueConfig, VelocityConfig,
};
use crate::util;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct PlotApp {
    pub files: Option<Files>,
    selected_tab: Tab,
    pub version: Version,
    pub power: PowerConfig,
    pub velocity: VelocityConfig,
    pub torque: TorqueConfig,
    pub temp1: Temp1Config,
    pub temp2: Temp2Config,
    pub custom: CustomConfig,
    #[serde(skip)]
    pub selectable_files: Option<SelectableFiles>,
    #[serde(skip)]
    pub data: Option<PlotData>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum Tab {
    Power,
    Velocity,
    Torque,
    Temp1,
    Temp2,
    Custom,
}

pub struct PlotData {
    pub raw_data: Arc<[DataEntry]>,
    pub raw_temp: Arc<[TempEntry]>,
    pub power: WheelValues,
    pub velocity: WheelValues,
    pub torque_set: WheelValues,
    pub torque_real: WheelValues,
    pub temp: WheelValues,
    pub room_temp: WheelValues,
    pub heatsink_temp: WheelValues,
    pub ams_temp_max: Vec<PlotPoint>,
    pub water_temp_converter: Vec<PlotPoint>,
    pub water_temp_motor: Vec<PlotPoint>,
    pub custom: Vec<CustomValues>,
}

pub enum CustomValues {
    Job(Job),
    Result(Result<Vec<PlotPoint>, Box<ExprError>>),
}

impl CustomValues {
    pub const fn empty() -> Self {
        Self::Result(Ok(Vec::new()))
    }

    pub fn into_job(self) -> Option<Job> {
        match self {
            Self::Job(v) => Some(v),
            _ => None,
        }
    }
}

pub struct Job {
    handle: JoinHandle<Result<Vec<PlotPoint>, Box<ExprError>>>,
}

impl Job {
    pub fn start(expr: Expr, data: Arc<[DataEntry]>, temp: Arc<[TempEntry]>) -> Self {
        let handle = std::thread::spawn(move || eval::eval(&expr, data, temp));
        Self { handle }
    }

    pub fn is_done(&self) -> bool {
        self.handle.is_finished()
    }

    pub fn join(self) -> Result<Vec<PlotPoint>, Box<ExprError>> {
        self.handle.join().expect("failed to join worker thread")
    }
}

pub struct WheelValues {
    pub fl: Vec<PlotPoint>,
    pub fr: Vec<PlotPoint>,
    pub rl: Vec<PlotPoint>,
    pub rr: Vec<PlotPoint>,
}

impl Default for PlotApp {
    fn default() -> Self {
        Self {
            files: None,
            selectable_files: None,
            data: None,
            selected_tab: Tab::Power,
            version: Version::default(),
            power: PowerConfig::default(),
            velocity: VelocityConfig::default(),
            torque: TorqueConfig::default(),
            temp1: Temp1Config::default(),
            temp2: Temp2Config::default(),
            custom: CustomConfig::default(),
        }
    }
}

impl eframe::App for PlotApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if ctx.input().modifiers.ctrl && ctx.input().key_pressed(Key::O) {
            self.open_dir_dialog();
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open dir").clicked() {
                        self.open_dir_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Reopen dir").clicked() {
                        if let Some(files) = &self.files {
                            self.try_open_dir(files.dir.clone());
                        }
                        ui.close_menu();
                    }
                    if ui.button("Reopen files").clicked() {
                        if let Some(files) = self.files.clone() {
                            self.try_open_files(files, true);
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button(format!("Version ( {} )", self.version), |ui| {
                    let mut clicked = false;
                    for v in Version::iter() {
                        clicked |= ui
                            .selectable_value(&mut self.version, v, v.to_string())
                            .clicked();
                    }
                    if clicked {
                        ui.close_menu();
                    }
                });

                ui.add_space(40.0);

                if let Some(files) = &self.files {
                    let files_iter = files.data.iter().chain(files.temp.iter());
                    let prefix = match util::common_parent_dir(files_iter) {
                        Some(p) => {
                            ui.label(format!("{}/", p.display()));
                            ui.add_space(20.0);
                            p
                        }
                        None => "".as_ref(),
                    };

                    for p in files.data.iter() {
                        let text = p.strip_prefix(prefix).unwrap().display().to_string();
                        ui.label(RichText::new(text).strong());
                    }
                    for p in files.temp.iter() {
                        let text = p.strip_prefix(prefix).unwrap().display().to_string();
                        ui.label(RichText::new(text).strong());
                    }
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if self.selectable_files.is_some() {
                ui.label("...");
            } else if let Some(d) = &mut self.data {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_tab, Tab::Power, "Power");
                    ui.selectable_value(&mut self.selected_tab, Tab::Velocity, "Velocity");
                    ui.selectable_value(&mut self.selected_tab, Tab::Torque, "Torque");
                    ui.selectable_value(&mut self.selected_tab, Tab::Temp1, "Temp 1");
                    ui.selectable_value(&mut self.selected_tab, Tab::Temp2, "Temp 2");
                    ui.selectable_value(&mut self.selected_tab, Tab::Custom, "Custom");
                    ui.add_space(30.0);

                    match self.selected_tab {
                        Tab::Power => plot::wheel_config(ui, &mut self.power),
                        Tab::Velocity => plot::wheel_config(ui, &mut self.velocity),
                        Tab::Torque => plot::wheel_config(ui, &mut self.torque),
                        Tab::Temp1 => plot::wheel_config(ui, &mut self.temp1),
                        Tab::Temp2 => plot::temp2_config(ui, &mut self.temp2),
                        Tab::Custom => plot::custom_config(ui, &mut self.custom),
                    }
                });

                match self.selected_tab {
                    Tab::Power => plot::power_plot(ui, d, &self.power),
                    Tab::Velocity => plot::velocity_plot(ui, d, &self.velocity),
                    Tab::Torque => plot::torque_plot(ui, d, &self.torque),
                    Tab::Temp1 => plot::temp1_plot(ui, d, &self.temp1),
                    Tab::Temp2 => plot::temp2_plot(ui, d, &self.temp2),
                    Tab::Custom => plot::custom_plot(ui, d, &mut self.custom),
                }
            } else {
                ui.label("Open or drag and drop a directory");
            }
        });

        if let Some(files) = &mut self.selectable_files {
            let mut open = true;
            let r = Window::new("Select files")
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .fixed_size(Vec2::new(800.0, 600.0))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| select_files_dialog(ui, files));

            match r {
                Some(r) if open => {
                    if let Some(true) = r.inner {
                        let files = self.selectable_files.take().unwrap();
                        self.concat_and_open(files);
                    }
                }
                _ => self.selectable_files = None,
            }
        }

        self.detect_files_being_dropped(ctx);
    }
}

pub fn select_files_dialog(ui: &mut Ui, opened_files: &mut SelectableFiles) -> bool {
    let data_files_iter = opened_files.data.iter().map(|f| &f.file);
    let temp_files_iter = opened_files.temp.iter().map(|f| &f.file);
    let common_prefix = match util::common_parent_dir(data_files_iter.chain(temp_files_iter)) {
        Some(p) => p.to_owned(),
        None => PathBuf::new(),
    };

    ui.push_id("data files table", |ui| {
        select_files_table(ui, &mut opened_files.data, &common_prefix);
    });
    ui.add_space(20.0);

    ui.push_id("temp files table", |ui| {
        select_files_table(ui, &mut opened_files.temp, &common_prefix);
    });
    ui.add_space(20.0);

    ui.horizontal(|ui| ui.button("Ok").clicked()).inner
}

enum MoveDirection {
    Up(usize),
    Down(usize),
}

fn select_files_table(
    ui: &mut Ui,
    files: &mut Vec<SelectableFile<impl TimeStamped>>,
    common_prefix: &Path,
) {
    let mut move_row = None;

    TableBuilder::new(ui)
        .column(Size::exact(50.0)) // arrows
        .column(Size::exact(60.0)) // select/deselect
        .column(Size::exact(400.0)) // file name
        .column(Size::exact(500.0)) // start end or error
        .resizable(false)
        .striped(true)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("Move");
            });
            header.col(|ui| {
                ui.heading("Select");
            });
            header.col(|ui| {
                ui.heading("File");
            });
            header.col(|ui| {
                ui.heading("Info");
            });
        })
        .body(|mut body| {
            for (i, f) in files.iter_mut().enumerate() {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            if ui.button("⏶").clicked() {
                                move_row = Some(MoveDirection::Up(i))
                            }
                            if ui.button("⏷").clicked() {
                                move_row = Some(MoveDirection::Down(i))
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            match f.result {
                                Ok(_) => ui.checkbox(&mut f.selected, ""),
                                Err(_) => ui.label("(ignored)"),
                            };
                        });
                    });
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            let name = f.file.strip_prefix(common_prefix).unwrap();
                            ui.label(name.display().to_string());
                        });
                    });
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            match &f.result {
                                Ok(d) => {
                                    if let (Some(first), Some(last)) = (d.first(), d.last()) {
                                        let start = util::format_time(first.time());
                                        let end = util::format_time(last.time());
                                        ui.label(format!("{start} - {end}"));
                                    } else {
                                        ui.label("File is empty");
                                    }
                                }
                                Err(e) => {
                                    ui.label(RichText::new(e.to_string()).color(Color32::RED));
                                }
                            };
                        });
                    });
                });
            }
        });

    match move_row {
        Some(MoveDirection::Up(idx)) => {
            if idx == 0 {
                let first = files.remove(0);
                files.push(first);
            } else {
                files.swap(idx, idx - 1);
            }
        }
        Some(MoveDirection::Down(idx)) => {
            if idx == files.len() - 1 {
                let first = files.remove(files.len() - 1);
                files.insert(0, first);
            } else {
                files.swap(idx, idx + 1);
            }
        }
        None => (),
    }
}

impl PlotApp {
    pub fn new(context: &eframe::CreationContext) -> Self {
        let mut app = context
            .storage
            .and_then(|s| eframe::get_value::<PlotApp>(s, eframe::APP_KEY))
            .unwrap_or_default();

        if let Some(f) = app.files.clone() {
            // TODO: don't show selection dialog if all files are opened successfully
            app.try_open_files(f, false);
        }
        app
    }
}
