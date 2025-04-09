use std::path::Path;
use std::sync::Arc;
use std::thread::JoinHandle;

use chrono::{Duration, Local, TimeZone};
use egui::{
    Align2, CentralPanel, Color32, Key, Modifiers, RichText, TopBottomPanel, Ui, Vec2, Window, menu,
};
use egui_extras::{Column, TableBuilder};
use egui_plot::PlotPoint;
use serde::{Deserialize, Serialize};

use crate::data::LogStream;
use crate::eval::{self, Expr, ExprError};
use crate::fs::{ErrorFile, Files, SelectableFile, SelectableFiles};
use crate::plot::{self, Config};
use crate::util;

const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PlotApp {
    pub config: Config,
    pub files: Option<Files>,
    #[serde(skip)]
    pub selectable_files: Option<SelectableFiles>,
    #[serde(skip)]
    pub data: Option<PlotData>,
}

pub struct PlotData {
    pub streams: Arc<[LogStream]>,
    pub plots: Vec<Vec<PlotValues>>,
}

pub enum PlotValues {
    Job(Job),
    Result(Result<Vec<PlotPoint>, Box<ExprError>>),
}

impl PlotValues {
    pub const fn empty() -> Self {
        Self::Result(Ok(Vec::new()))
    }

    pub fn into_job(self) -> Option<Job> {
        match self {
            Self::Job(v) => Some(v),
            _ => None,
        }
    }

    pub fn x_err(&self) -> Option<&cods::Error> {
        match self {
            PlotValues::Result(Err(e)) => e.x.as_ref(),
            _ => None,
        }
    }

    pub fn y_err(&self) -> Option<&cods::Error> {
        match self {
            PlotValues::Result(Err(e)) => e.y.as_ref(),
            _ => None,
        }
    }
}

pub struct Job {
    handle: JoinHandle<Result<Vec<PlotPoint>, Box<ExprError>>>,
}

impl Job {
    pub fn start(expr: Expr, data: Arc<[LogStream]>) -> Self {
        let handle = std::thread::spawn(move || eval::eval(&expr, data));
        Self { handle }
    }

    pub fn is_done(&self) -> bool {
        self.handle.is_finished()
    }

    pub fn join(self) -> Result<Vec<PlotPoint>, Box<ExprError>> {
        self.handle.join().expect("failed to join worker thread")
    }
}

impl eframe::App for PlotApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::O)) {
            self.open_dir_dialog();
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::O)) {
            if let Some(files) = &self.files {
                self.try_open_dir(files.dir.clone());
            }
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

                ui.add_space(40.0);

                if let Some(files) = &self.files {
                    let files_iter = files.items.iter();
                    let prefix = match util::common_parent_dir(files_iter) {
                        Some(p) => {
                            ui.label(format!("{}/", p.display()));
                            ui.add_space(20.0);
                            p
                        }
                        None => "".as_ref(),
                    };

                    for p in files.items.iter() {
                        let text = p.strip_prefix(prefix).unwrap().display().to_string();
                        ui.label(RichText::new(text).strong());
                    }
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if self.selectable_files.is_some() {
                ui.label("...");
            } else if let Some(data) = &mut self.data {
                plot::keybindings(ui, data, &mut self.config);
                plot::tab_bar(ui, data, &mut self.config);
                plot::tab_plot(ui, data, &mut self.config);
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
                        self.concat_and_show(files);
                    }
                }
                _ => self.selectable_files = None,
            }
        }

        self.detect_files_being_dropped(ctx);
    }
}

pub fn select_files_dialog(ui: &mut Ui, opened_files: &mut SelectableFiles) -> bool {
    let common_prefix = opened_files.dir.as_path();

    for (i, group) in opened_files.by_header.iter_mut().enumerate() {
        ui.push_id(i, |ui| {
            select_files_table(ui, group, common_prefix);
        });
        ui.add_space(20.0);
    }

    error_files_table(ui, &opened_files.with_error, common_prefix);

    ui.add_space(20.0);

    ui.horizontal(|ui| ui.button("Ok").clicked()).inner
}

enum MoveDirection {
    Up(usize),
    Down(usize),
}

fn select_files_table(ui: &mut Ui, files: &mut Vec<SelectableFile>, common_prefix: &Path) {
    let mut move_row = None;

    TableBuilder::new(ui)
        .column(Column::exact(50.0)) // arrows
        .column(Column::exact(50.0)) // select/deselect
        .column(Column::exact(80.0)) // version
        .column(Column::exact(200.0)) // file name
        .column(Column::exact(400.0)) // sanity check
        .column(Column::exact(300.0)) // start end
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
                ui.heading("Version");
            });
            header.col(|ui| {
                ui.heading("File");
            });
            header.col(|ui| {
                ui.heading("Sanity check");
            });
            header.col(|ui| {
                ui.heading("Time");
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
                            ui.checkbox(&mut f.selected, "");
                        });
                    });
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            ui.label(f.stream.version.to_string());
                        });
                    });
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            let name = f.file.strip_prefix(common_prefix).unwrap();
                            ui.label(name.display().to_string());
                        });
                    });
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| match &f.sanity_check {
                            Ok(_) => ui.label("ok"),
                            Err(e) => ui.colored_label(Color32::YELLOW, &e.0),
                        });
                    });
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            if let (Some(first), Some(last)) =
                                (f.stream.time.first(), f.stream.time.last())
                            {
                                match f.stream.start {
                                    Some(start) => {
                                        let start = start + Duration::milliseconds(*first as i64);
                                        let end = start + Duration::milliseconds(*last as i64);

                                        let local_start = Local
                                            .from_utc_datetime(&start)
                                            .format(DATE_TIME_FORMAT);
                                        let local_end =
                                            Local.from_utc_datetime(&end).format(DATE_TIME_FORMAT);

                                        ui.label(format!("{local_start} - {local_end}"));
                                    }
                                    None => {
                                        let start = util::format_time(*first as f64 / 1000.0);
                                        let end = util::format_time(*last as f64 / 1000.0);
                                        ui.label(format!("{start} - {end}"));
                                    }
                                }
                            } else {
                                ui.label("File is empty");
                            }
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

fn error_files_table(ui: &mut Ui, files: &[ErrorFile], common_prefix: &Path) {
    TableBuilder::new(ui)
        .column(Column::exact(400.0)) // file name
        .column(Column::exact(500.0)) // error
        .resizable(false)
        .striped(true)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("File");
            });
            header.col(|ui| {
                ui.heading("Error");
            });
        })
        .body(|mut body| {
            for e in files.iter() {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            let name = e.file.strip_prefix(common_prefix).unwrap();
                            ui.label(name.display().to_string());
                        });
                    });
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            ui.label(RichText::new(e.error.to_string()).color(Color32::RED));
                        });
                    });
                });
            }
        });
}

impl PlotApp {
    pub fn new(context: &eframe::CreationContext) -> Self {
        let mut app = context
            .storage
            .and_then(|s| eframe::get_value::<PlotApp>(s, eframe::APP_KEY))
            .unwrap_or_default();

        if let Some(f) = app.files.clone() {
            app.try_open_files(f, false);
        }
        app
    }
}
