use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use eframe::egui::plot::{Line, Plot, Values};
use eframe::egui::{menu, CentralPanel, TopBottomPanel};
use eframe::{egui, epi, NativeOptions};

use s3plot::Data;

struct PlotApp {
    picked_path: Option<String>,
    data: Option<Data>,
}

impl Default for PlotApp {
    fn default() -> Self {
        Self {
            picked_path: None,
            data: None,
        }
    }
}

impl epi::App for PlotApp {
    fn name(&self) -> &str {
        "S3 Plot"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _: &epi::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                menu::menu_button(ui, "File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            let path = Some(path.display().to_string());
                            if let Some(p) = path {
                                if let Ok(_) = self.open(&p) {
                                    self.picked_path = Some(p);
                                } else {
                                    self.picked_path = None;
                                }
                            }
                        }
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(d) = &self.data {
                if let Some(p) = &self.picked_path {
                    ui.label(p);
                }

                let h = ui.available_height() / 2.0 - ui.fonts().row_height(egui::TextStyle::Body);

                ui.columns(2, |uis| {
                    let ui = &mut uis[0];
                    let values = Values::from_values_iter(d.pmotor_fl());
                    ui.label("fl motor");
                    Plot::new("fl_motor")
                        .height(h)
                        .show(ui, |ui| ui.line(Line::new(values)));
                    let values = Values::from_values_iter(d.pmotor_fr());
                    ui.label("fr motor");
                    Plot::new("fr_motor")
                        .height(h)
                        .show(ui, |ui| ui.line(Line::new(values)));

                    let ui = &mut uis[1];
                    let values = Values::from_values_iter(d.pmotor_rl());
                    ui.label("hl motor");
                    Plot::new("hl_motor")
                        .height(h)
                        .show(ui, |ui| ui.line(Line::new(values)));
                    let values = Values::from_values_iter(d.pmotor_rr());
                    ui.label("hr motor");
                    Plot::new("hr_motor")
                        .height(h)
                        .show(ui, |ui| ui.line(Line::new(values)));
                });
            } else {
                ui.label("Open or drag and drop a file");
            }
        });
    }
}

impl PlotApp {
    fn open(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut reader = BufReader::new(File::open(path)?);
        self.data = Some(Data::read(&mut reader)?);
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let app = PlotApp::default();
    eframe::run_native(Box::new(app), NativeOptions::default());
}
