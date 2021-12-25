use std::fs::File;
use std::io::BufReader;

use eframe::egui::plot::{Line, Plot, Values};
use eframe::{egui, epi, NativeOptions};

use s3plot::Data;

#[derive(Default)]
struct PlotApp {
    data: Option<Data>,
}

impl epi::App for PlotApp {
    fn name(&self) -> &str {
        "S3 Plot"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        let _ = self.import();
    }

    fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Open").clicked() {
                        let _ = self.import();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(d) = &self.data {
                ui.horizontal(|ui| {
                    let values = Values::from_values_iter(d.pmotor_fl());
                    ui.add(Plot::new("fl motor").line(Line::new(values)));
                });
                ui.horizontal(|ui| {
                    let values = Values::from_values_iter(d.pmotor_fr());
                    ui.add(Plot::new("fr motor").line(Line::new(values)));
                });
                ui.horizontal(|ui| {
                    let values = Values::from_values_iter(d.pmotor_rl());
                    ui.add(Plot::new("rl motor").line(Line::new(values)));
                });
                ui.horizontal(|ui| {
                    let values = Values::from_values_iter(d.pmotor_rr());
                    ui.add(Plot::new("rr motor").line(Line::new(values)));
                });
            } else {
                ui.label("Open or drag and drop a file");
            }
        });
    }
}

impl PlotApp {
    fn import(&mut self) -> anyhow::Result<()> {
        let mut reader = BufReader::new(File::open("data/1.bin")?);
        self.data = Some(Data::read(&mut reader)?);
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let app = PlotApp::default();
    eframe::run_native(Box::new(app), NativeOptions::default());
}
