use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use egui::{Align2, Color32, Context, Id, LayerId, Order, Pos2, Rect, TextStyle, Vec2};
use serde::{Deserialize, Serialize};

use crate::app::{CustomValues, PlotData, WheelValues};
use crate::data::{Data, DataEntry, MapOverTime, Temp, TempEntry};
use crate::{eval, PlotApp};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Files {
    pub data: Vec<PathBuf>,
    pub temp: Option<PathBuf>,
}

impl PlotApp {
    pub fn open_dir_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            if let Ok(files) = find_files(&path) {
                self.try_open(files);
            }
        }
    }

    pub fn detect_files_being_dropped(&mut self, ctx: &Context) {
        // Preview hovering files
        if !ctx.input().raw.hovered_files.is_empty() {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            let screen_rect = ctx.input().screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));

            // Draw plus
            let w = screen_rect.width();
            let h = screen_rect.height();
            let center = screen_rect.center();

            // Background box
            let box_size = f32::min(w, h) * 0.04;
            let box_rect = Rect {
                min: Pos2::new(center.x - box_size, center.y - box_size),
                max: Pos2::new(center.x + box_size, center.y + box_size),
            };
            painter.rect_filled(box_rect, box_size * 0.3, Color32::from_white_alpha(50));

            // Forground
            let long_extend = box_size * 0.6;
            let short_extend = long_extend * 0.1;
            let color = Color32::from_gray(0);
            let rect = Rect {
                min: Pos2::new(center.x - long_extend, center.y - short_extend),
                max: Pos2::new(center.x + long_extend, center.y + short_extend),
            };
            painter.rect_filled(rect, 0.0, color);
            let rect = Rect {
                min: Pos2::new(center.x - short_extend, center.y - long_extend),
                max: Pos2::new(center.x + short_extend, center.y + long_extend),
            };
            painter.rect_filled(rect, 0.0, color);

            // File names
            let pos = center + Vec2::new(0.0, box_size * 2.0);
            let mut text = String::new();
            for f in ctx.input().raw.hovered_files.iter() {
                if let Some(p) = &f.path {
                    write!(&mut text, "\n{}", p.display()).ok();
                }
            }
            painter.text(
                pos,
                Align2::CENTER_TOP,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::from_white_alpha(160),
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
                if let Ok(files) = find_files(p) {
                    self.try_open(files);
                }
            }
        }
    }

    pub fn try_open(&mut self, files: Files) {
        fn open_data(files: &Files) -> anyhow::Result<Data> {
            let mut data = Data::default();
            for p in files.data.iter() {
                let mut reader = BufReader::new(File::open(p)?);
                data.read_extend(&mut reader)?;
            }
            Ok(data)
        }
        fn open_temp(files: &Files) -> anyhow::Result<Temp> {
            let mut temp = Temp::default();
            if let Some(p) = &files.temp {
                let mut reader = BufReader::new(File::open(p)?);
                temp.read_extend(&mut reader)?;
            }
            Ok(temp)
        }

        match (open_data(&files), open_temp(&files)) {
            (Ok(d), Ok(t)) => {
                let power = WheelValues {
                    fl: d.iter().map_over_time(DataEntry::power_fl),
                    fr: d.iter().map_over_time(DataEntry::power_fr),
                    rl: d.iter().map_over_time(DataEntry::power_rl),
                    rr: d.iter().map_over_time(DataEntry::power_rr),
                };
                let velocity = WheelValues {
                    fl: d.iter().map_over_time(DataEntry::velocity_fl),
                    fr: d.iter().map_over_time(DataEntry::velocity_fr),
                    rl: d.iter().map_over_time(DataEntry::velocity_rl),
                    rr: d.iter().map_over_time(DataEntry::velocity_rr),
                };
                let torque_set = WheelValues {
                    fl: d.iter().map_over_time(DataEntry::torque_set_fl),
                    fr: d.iter().map_over_time(DataEntry::torque_set_fr),
                    rl: d.iter().map_over_time(DataEntry::torque_set_rl),
                    rr: d.iter().map_over_time(DataEntry::torque_set_rr),
                };
                let torque_real = WheelValues {
                    fl: d.iter().map_over_time(DataEntry::torque_real_fl),
                    fr: d.iter().map_over_time(DataEntry::torque_real_fr),
                    rl: d.iter().map_over_time(DataEntry::torque_real_rl),
                    rr: d.iter().map_over_time(DataEntry::torque_real_rr),
                };
                let temp = WheelValues {
                    fl: t.iter().map_over_time(TempEntry::temp_fl),
                    fr: t.iter().map_over_time(TempEntry::temp_fr),
                    rl: t.iter().map_over_time(TempEntry::temp_rl),
                    rr: t.iter().map_over_time(TempEntry::temp_rr),
                };
                let room_temp = WheelValues {
                    fl: t.iter().map_over_time(TempEntry::room_temp_fl),
                    fr: t.iter().map_over_time(TempEntry::room_temp_fr),
                    rl: t.iter().map_over_time(TempEntry::room_temp_rl),
                    rr: t.iter().map_over_time(TempEntry::room_temp_rr),
                };
                let heatsink_temp = WheelValues {
                    fl: t.iter().map_over_time(TempEntry::heatsink_temp_fl),
                    fr: t.iter().map_over_time(TempEntry::heatsink_temp_fr),
                    rl: t.iter().map_over_time(TempEntry::heatsink_temp_rl),
                    rr: t.iter().map_over_time(TempEntry::heatsink_temp_rr),
                };
                let ams_temp_max = t.iter().map_over_time(TempEntry::ams_temp_max);
                let water_temp_converter = t.iter().map_over_time(TempEntry::water_temp_converter);
                let water_temp_motor = t.iter().map_over_time(TempEntry::water_temp_motor);
                let custom = self
                    .custom
                    .plots
                    .iter()
                    .map(|p| {
                        let r = eval::eval(&p.expr, &d, &t);
                        CustomValues::from_result(r)
                    })
                    .collect();

                self.files = Some(files);
                self.data = Some(PlotData {
                    raw_data: d,
                    raw_temp: t,
                    power,
                    velocity,
                    torque_set,
                    torque_real,
                    temp,
                    room_temp,
                    heatsink_temp,
                    ams_temp_max,
                    water_temp_converter,
                    water_temp_motor,
                    custom,
                });
            }
            _ => {
                self.files = None;
                self.data = None;
            }
        }
    }
}

fn find_files(path: &Path) -> anyhow::Result<Files> {
    fn filename(path: &Path) -> Option<&str> {
        if path.extension()? != "bin" {
            return None;
        }
        path.file_stem()?.to_str()
    }

    let mut files = Files::default();
    let mut paths = HashMap::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(name) = filename(&path) {
            if name == "temperature" {
                files.temp = Some(path);
            } else if let Ok(n) = name.parse::<usize>() {
                paths.insert(n, path);
            }
        }
    }

    for i in 1.. {
        match paths.remove(&i) {
            Some(p) => files.data.push(p),
            None => break,
        }
    }

    Ok(files)
}
