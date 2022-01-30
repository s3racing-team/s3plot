use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use eframe::egui::{Align2, Color32, CtxRef, Id, LayerId, Order, TextStyle};

use crate::app::{PlotData, QuadValues};
use crate::data::{Data, MapOverTime};
use crate::{eval, PlotApp};

impl PlotApp {
    pub fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.try_open(path);
        }
    }

    pub fn detect_files_being_dropped(&mut self, ctx: &CtxRef) {
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

    pub fn try_open(&mut self, path: PathBuf) {
        fn open(path: &Path) -> anyhow::Result<Data> {
            let mut reader = BufReader::new(File::open(path)?);
            Data::read(&mut reader)
        }

        match open(&path) {
            Ok(d) => {
                self.current_path = Some(path);
                let power = QuadValues {
                    fl: d.power_fl().map_over_time(),
                    fr: d.power_fr().map_over_time(),
                    rl: d.power_rl().map_over_time(),
                    rr: d.power_rr().map_over_time(),
                };
                let velocity = QuadValues {
                    fl: d.velocity_fl().map_over_time(),
                    fr: d.velocity_fr().map_over_time(),
                    rl: d.velocity_rl().map_over_time(),
                    rr: d.velocity_rr().map_over_time(),
                };
                let torque_set = QuadValues {
                    fl: d.torque_set_fl().map_over_time(),
                    fr: d.torque_set_fr().map_over_time(),
                    rl: d.torque_set_rl().map_over_time(),
                    rr: d.torque_set_rr().map_over_time(),
                };
                let torque_real = QuadValues {
                    fl: d.torque_real_fl().map_over_time(),
                    fr: d.torque_real_fr().map_over_time(),
                    rl: d.torque_real_rl().map_over_time(),
                    rr: d.torque_real_rr().map_over_time(),
                };
                let custom = self
                    .custom
                    .plots
                    .iter()
                    .map(|p| eval::eval(&p.expr, &d).unwrap_or_default())
                    .collect();
                self.data = Some(PlotData {
                    raw: d,
                    power,
                    velocity,
                    torque_set,
                    torque_real,
                    custom,
                });
            }
            Err(_) => {
                self.current_path = None;
                self.data = None;
            }
        }
    }
}
