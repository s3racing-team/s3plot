use egui::plot::Value;
use egui::{menu, CentralPanel, Key, TopBottomPanel, Visuals};
use serde::{Deserialize, Serialize};

use crate::custom;
use crate::custom::CustomConfig;
use crate::data::{Data, Temp};
use crate::fs::Files;
use crate::plot::{self, PowerConfig, TempConfig, TorqueConfig, VelocityConfig};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct PlotApp {
    pub files: Option<Files>,
    selected_tab: Tab,
    pub power: PowerConfig,
    pub velocity: VelocityConfig,
    pub torque: TorqueConfig,
    pub temp: TempConfig,
    pub custom: CustomConfig,
    #[serde(skip)]
    pub data: Option<PlotData>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum Tab {
    Power,
    Velocity,
    Torque,
    Temp,
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
    pub custom: Vec<Vec<Value>>,
}

pub struct WheelValues {
    pub fl: Vec<Value>,
    pub fr: Vec<Value>,
    pub rl: Vec<Value>,
    pub rr: Vec<Value>,
}

impl Default for PlotApp {
    fn default() -> Self {
        Self {
            files: None,
            data: None,
            selected_tab: Tab::Power,
            power: PowerConfig::default(),
            velocity: VelocityConfig::default(),
            torque: TorqueConfig::default(),
            temp: TempConfig::default(),
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
                menu::menu_button(ui, "File", |ui| {
                    if ui.button("Open").clicked() {
                        self.open_dir_dialog();
                    }
                });
                ui.add_space(40.0);

                if let Some(files) = &self.files {
                    // TODO: strip common components
                    for p in files.data.iter() {
                        ui.label(format!("{}", p.display()));
                    }
                    if let Some(p) = &files.temp {
                        ui.label(format!("{}", p.display()));
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
                    ui.selectable_value(&mut self.selected_tab, Tab::Temp, "Temperature");
                    ui.selectable_value(&mut self.selected_tab, Tab::Custom, "Custom");
                    ui.add_space(30.0);

                    match self.selected_tab {
                        Tab::Power => plot::config(ui, &mut self.power),
                        Tab::Velocity => plot::config(ui, &mut self.velocity),
                        Tab::Torque => plot::config(ui, &mut self.torque),
                        Tab::Temp => plot::config(ui, &mut self.temp),
                        Tab::Custom => custom::ratio_slider(ui, &mut self.custom),
                    }
                });

                match self.selected_tab {
                    Tab::Power => plot::power_plot(ui, d, &self.power),
                    Tab::Velocity => plot::velocity_plot(ui, d, &self.velocity),
                    Tab::Torque => plot::torque_plot(ui, d, &self.torque),
                    Tab::Temp => plot::temp_plot(ui, d, &self.temp),
                    Tab::Custom => custom::plot(ui, d, &mut self.custom),
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
        context.egui_ctx.set_visuals(Visuals::dark());

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
