use egui::plot::Value;
use egui::{menu, CentralPanel, Key, RichText, TopBottomPanel};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::data::{Data, Temp, Version};
use crate::eval::ExprError;
use crate::fs::Files;
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
    pub raw_data: Data,
    pub raw_temp: Temp,
    pub power: WheelValues,
    pub velocity: WheelValues,
    pub torque_set: WheelValues,
    pub torque_real: WheelValues,
    pub temp: WheelValues,
    pub room_temp: WheelValues,
    pub heatsink_temp: WheelValues,
    pub ams_temp_max: Vec<Value>,
    pub water_temp_converter: Vec<Value>,
    pub water_temp_motor: Vec<Value>,
    pub custom: Vec<CustomValues>,
}

pub struct WheelValues {
    pub fl: Vec<Value>,
    pub fr: Vec<Value>,
    pub rl: Vec<Value>,
    pub rr: Vec<Value>,
}

#[derive(Default)]
pub struct CustomValues {
    pub error: ExprError,
    pub values: Vec<Value>,
}

impl CustomValues {
    pub fn from_values(values: Vec<Value>) -> Self {
        Self {
            error: ExprError::default(),
            values,
        }
    }

    pub fn from_error(error: ExprError) -> Self {
        Self {
            error,
            values: Vec::new(),
        }
    }

    pub fn from_result(r: Result<Vec<Value>, ExprError>) -> Self {
        match r {
            Ok(v) => CustomValues::from_values(v),
            Err(e) => CustomValues::from_error(e),
        }
    }
}

impl Default for PlotApp {
    fn default() -> Self {
        Self {
            files: None,
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
                    if ui.button("Open").clicked() {
                        self.open_dir_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Reopen").clicked() {
                        if let Some(files) = self.files.clone() {
                            self.try_open(files);
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
                    let prefix = match util::common_parent_dir(files) {
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
                    if let Some(p) = &files.temp {
                        let text = p.strip_prefix(prefix).unwrap().display().to_string();
                        ui.label(RichText::new(text).strong());
                    }
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(d) = &mut self.data {
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
                ui.label("Open or drag and drop a file");
            }
        });

        self.detect_files_being_dropped(ctx);
    }
}

impl PlotApp {
    pub fn new(context: &eframe::CreationContext) -> Self {
        let mut app = context
            .storage
            .and_then(|s| eframe::get_value::<PlotApp>(s, eframe::APP_KEY))
            .unwrap_or_default();

        if let Some(f) = app.files.clone() {
            app.try_open(f);
        }
        app
    }
}
