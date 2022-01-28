use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use eframe::egui::plot::{Legend, Line, Plot, Values};
use eframe::egui::{menu, CentralPanel, Slider, TopBottomPanel, LayerId, Order, Id, Color32, Align2, TextStyle, CtxRef, Ui};
use eframe::{epi, NativeOptions};

use s3plot::Data;

struct PlotApp {
    current_path: Option<PathBuf>,
    data: Option<Data>,
    data_aspect: f32,
    selected_mode: Mode,
}

#[derive(PartialEq, Eq)]
enum Mode {
    Power,
    Speed,
    Torque,
}

impl Default for PlotApp {
    fn default() -> Self {
        Self {
            current_path: None,
            data: None,
            selected_mode: Mode::Power,
            data_aspect: 0.005,
        }
    }
}

impl epi::App for PlotApp {
    fn name(&self) -> &str {
        "S3 Plot"
    }

    fn update(&mut self, ctx: &CtxRef, _: &epi::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                menu::menu_button(ui, "File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.try_open(path);
                        }
                    }
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(d) = &self.data {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_mode, Mode::Power, "Power");
                    ui.selectable_value(&mut self.selected_mode, Mode::Speed, "Speed");
                    ui.selectable_value(&mut self.selected_mode, Mode::Torque, "Torque");
                    ui.add_space(40.0);

                    ui.add(Slider::new(&mut self.data_aspect, 0.00001..=1.0).logarithmic(true));

                    if let Some(p) = &self.current_path {
                        ui.label(format!("{}", p.display()));
                    }
                });

                match self.selected_mode {
                    Mode::Power => {
                        motor_plot(
                            ui,
                            [
                                [Line::new(Values::from_values_iter(d.power_fl())).name("power")],
                                [Line::new(Values::from_values_iter(d.power_fr())).name("power")],
                                [Line::new(Values::from_values_iter(d.power_rl())).name("power")],
                                [Line::new(Values::from_values_iter(d.power_rr())).name("power")],
                            ],
                            self.data_aspect,
                        );
                    }
                    Mode::Speed => {
                        motor_plot(
                            ui,
                            [
                                [Line::new(Values::from_values_iter(d.speed_fl())).name("speed")],
                                [Line::new(Values::from_values_iter(d.speed_fr())).name("speed")],
                                [Line::new(Values::from_values_iter(d.speed_rl())).name("speed")],
                                [Line::new(Values::from_values_iter(d.speed_rr())).name("speed")],
                            ],
                            self.data_aspect,
                        );
                    }
                    Mode::Torque => {
                        motor_plot(
                            ui,
                            [
                                [
                                    Line::new(Values::from_values_iter(d.torque_set_fl()))
                                        .name("set"),
                                    Line::new(Values::from_values_iter(d.torque_real_fl()))
                                        .name("real"),
                                ],
                                [
                                    Line::new(Values::from_values_iter(d.torque_set_fr()))
                                        .name("set"),
                                    Line::new(Values::from_values_iter(d.torque_real_fr()))
                                        .name("real"),
                                ],
                                [
                                    Line::new(Values::from_values_iter(d.torque_set_rl()))
                                        .name("set"),
                                    Line::new(Values::from_values_iter(d.torque_real_rl()))
                                        .name("real"),
                                ],
                                [
                                    Line::new(Values::from_values_iter(d.torque_set_rr()))
                                        .name("set"),
                                    Line::new(Values::from_values_iter(d.torque_real_rr()))
                                        .name("real"),
                                ],
                            ],
                            self.data_aspect,
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

fn motor_plot<const COUNT: usize>(ui: &mut Ui, lines: [[Line; COUNT]; 4], data_aspect: f32) {
    let h = ui.available_height() / 2.0
        - ui.fonts().row_height(TextStyle::Body)
        - ui.style().spacing.item_spacing.y;

    let [fl, fr, rl, rr] = lines;

    ui.columns(2, |uis| {
        let ui = &mut uis[0];
        ui.label("front left");
        Plot::new("fl_motor")
            .height(h)
            .data_aspect(data_aspect)
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in fl {
                    ui.line(l);
                }
            })
            .response;
        ui.label("rear left");
        Plot::new("rl_motor")
            .height(h)
            .data_aspect(data_aspect)
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in rl {
                    ui.line(l);
                }
            });

        let ui = &mut uis[1];
        ui.label("front right");
        Plot::new("fr_motor")
            .height(h)
            .data_aspect(data_aspect)
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in fr {
                    ui.line(l);
                }
            });
        ui.label("rear right");
        Plot::new("rr_motor")
            .height(h)
            .data_aspect(data_aspect)
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in rr {
                    ui.line(l);
                }
            });
    })
}

impl PlotApp {
    fn try_open(&mut self, path: PathBuf) {
        match Self::open(&path) {
            Ok(d) => {
                self.current_path = Some(path);
                self.data = Some(d);
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
        // Preview hovering files:
        if !ctx.input().raw.hovered_files.is_empty() {
            let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
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

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            if let Some(p) = ctx.input().raw.dropped_files.first().and_then(|f| f.path.as_ref()) {
                self.try_open(p.clone());
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let app = PlotApp::default();
    eframe::run_native(Box::new(app), NativeOptions::default());
}
