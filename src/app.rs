use std::path::PathBuf;

use eframe::egui::plot::Value;
use eframe::egui::{menu, CentralPanel, CtxRef, Key, TopBottomPanel};
use eframe::epi::{self, App, Frame};
use serde::{Deserialize, Serialize};

use crate::{util, custom};
use crate::custom::CustomConfig;
use crate::data::Data;
use crate::motor::{self, QuadValues};

const APP_NAME: &str = "s3plot";

const POWER_ASPECT_RATIO: f32 = 0.005;
const VELOCITY_ASPECT_RATIO: f32 = 0.5;
const TORQUE_ASPECT_RATIO: f32 = 0.04;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct PlotApp {
    pub current_path: Option<PathBuf>,
    selected_tab: Tab,
    pub power_aspect_ratio: f32,
    pub velocity_aspect_ratio: f32,
    pub torque_aspect_ratio: f32,
    pub custom: CustomConfig,
    #[serde(skip)]
    pub data: Option<PlotData>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum Tab {
    Power,
    Speed,
    Torque,
    Custom,
}

pub struct PlotData {
    pub raw: Data,
    pub power: QuadValues,
    pub velocity: QuadValues,
    pub torque_set: QuadValues,
    pub torque_real: QuadValues,
    pub custom: Vec<Value>,
}

impl Default for PlotApp {
    fn default() -> Self {
        Self {
            current_path: None,
            data: None,
            selected_tab: Tab::Power,
            power_aspect_ratio: POWER_ASPECT_RATIO,
            velocity_aspect_ratio: VELOCITY_ASPECT_RATIO,
            torque_aspect_ratio: TORQUE_ASPECT_RATIO,
            custom: Default::default(),
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
            if let Some(d) = &mut self.data {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_tab, Tab::Power, "Power");
                    ui.selectable_value(&mut self.selected_tab, Tab::Speed, "Speed");
                    ui.selectable_value(&mut self.selected_tab, Tab::Torque, "Torque");
                    ui.selectable_value(&mut self.selected_tab, Tab::Custom, "Custom");
                    ui.add_space(40.0);

                    ui.label("aspect ratio");
                    match self.selected_tab {
                        Tab::Power => {
                            util::ratio_slider(
                                ui,
                                &mut self.power_aspect_ratio,
                                POWER_ASPECT_RATIO,
                                100.0,
                            );
                        }
                        Tab::Speed => {
                            util::ratio_slider(
                                ui,
                                &mut self.velocity_aspect_ratio,
                                VELOCITY_ASPECT_RATIO,
                                100.0,
                            );
                        }
                        Tab::Torque => {
                            util::ratio_slider(
                                ui,
                                &mut self.torque_aspect_ratio,
                                TORQUE_ASPECT_RATIO,
                                100.0,
                            );
                        }
                        Tab::Custom => {
                            custom::ratio_slider(ui, &mut self.custom);
                        }
                    }
                    ui.add_space(40.0);

                    if let Some(p) = &self.current_path {
                        ui.label(format!("{}", p.display()));
                    }
                });

                match self.selected_tab {
                    Tab::Power => {
                        motor::plot_power(ui, d, self.power_aspect_ratio);
                    }
                    Tab::Speed => {
                        motor::plot_velocity(ui, d, self.velocity_aspect_ratio);
                    }
                    Tab::Torque => {
                        motor::plot_torque(ui, d, self.torque_aspect_ratio);
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
