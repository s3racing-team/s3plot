use std::path::PathBuf;

use egui::plot::Value;
use egui::{menu, CentralPanel, Key, TopBottomPanel};
use serde::{Deserialize, Serialize};

use crate::custom;
use crate::custom::CustomConfig;
use crate::data::Data;
use crate::motor::{self, PowerConfig, TorqueConfig, VelocityConfig};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct PlotApp {
    pub current_path: Option<PathBuf>,
    selected_tab: Tab,
    pub power: PowerConfig,
    pub velocity: VelocityConfig,
    pub torque: TorqueConfig,
    pub custom: CustomConfig,
    #[serde(skip)]
    pub data: Option<PlotData>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum Tab {
    Power,
    Velocity,
    Torque,
    Custom,
}

pub struct PlotData {
    pub raw: Data,
    pub power: QuadValues,
    pub velocity: QuadValues,
    pub torque_set: QuadValues,
    pub torque_real: QuadValues,
    pub custom: Vec<Vec<Value>>,
}

pub struct QuadValues {
    pub fl: Vec<Value>,
    pub fr: Vec<Value>,
    pub rl: Vec<Value>,
    pub rr: Vec<Value>,
}

impl Default for PlotApp {
    fn default() -> Self {
        Self {
            current_path: None,
            data: None,
            selected_tab: Tab::Power,
            power: PowerConfig::default(),
            velocity: VelocityConfig::default(),
            torque: TorqueConfig::default(),
            custom: CustomConfig::default(),
        }
    }
}

impl PlotApp {
    pub fn new(context: &eframe::CreationContext) -> Self {
        let mut app = context
            .storage
            .and_then(|s| eframe::get_value::<PlotApp>(s, eframe::APP_KEY))
            .unwrap_or_default();

        if let Some(p) = app.current_path.clone() {
            app.try_open(p);
        }
        app
    }
}

impl eframe::App for PlotApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
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
                ui.add_space(40.0);

                if let Some(p) = &self.current_path {
                    ui.label(format!("{}", p.display()));
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(d) = &mut self.data {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_tab, Tab::Power, "Power");
                    ui.selectable_value(&mut self.selected_tab, Tab::Velocity, "Velocity");
                    ui.selectable_value(&mut self.selected_tab, Tab::Torque, "Torque");
                    ui.selectable_value(&mut self.selected_tab, Tab::Custom, "Custom");
                    ui.add_space(30.0);

                    match self.selected_tab {
                        Tab::Power => {
                            motor::config(ui, &mut self.power);
                        }
                        Tab::Velocity => {
                            motor::config(ui, &mut self.velocity);
                        }
                        Tab::Torque => {
                            motor::config(ui, &mut self.torque);
                        }
                        Tab::Custom => {
                            custom::ratio_slider(ui, &mut self.custom);
                        }
                    }
                });

                match self.selected_tab {
                    Tab::Power => {
                        motor::power_plot(ui, d, &self.power);
                    }
                    Tab::Velocity => {
                        motor::velocity_plot(ui, d, &self.velocity);
                    }
                    Tab::Torque => {
                        motor::torque_plot(ui, d, &self.torque);
                    }
                    Tab::Custom => {
                        custom::plot(ui, d, &mut self.custom);
                    }
                }
            } else {
                ui.label("Open or drag and drop a file");
            }
        });

        self.detect_files_being_dropped(ctx);
    }
}
