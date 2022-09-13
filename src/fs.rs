use std::fmt::Write as _;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use egui::{Align2, Color32, Context, Id, LayerId, Order, Pos2, Rect, TextStyle, Vec2};
use serde::{Deserialize, Serialize};

use crate::data::{self, DataEntry, LogFile, SanityError};
use crate::PlotApp;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Files {
    pub dir: PathBuf,
    pub items: Vec<PathBuf>,
}

pub struct SelectableFiles {
    pub dir: PathBuf,
    pub items: Vec<SelectableFile>,
}

pub struct SelectableFile {
    pub selected: bool,
    pub file: PathBuf,
    pub result: Result<LogFile, data::Error>,
    pub sanity_check: Result<(), SanityError>,
}

impl SelectableFile {
    pub fn new(
        seleted: bool,
        file: PathBuf,
        result: Result<LogFile, data::Error>,
        sanity_check: Result<(), SanityError>,
    ) -> Self {
        Self {
            selected: seleted,
            file,
            result,
            sanity_check,
        }
    }
}

impl PlotApp {
    pub fn open_dir_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.try_open_dir(path);
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
                .and_then(|f| f.path.clone())
            {
                self.try_open_dir(p);
            }
        }
    }

    pub fn try_open_dir(&mut self, dir: PathBuf) {
        if let Ok(files) = find_files(dir) {
            self.selectable_files = Some(open_files(files, self.version));
        }
    }

    pub fn try_open_files(&mut self, files: Files, always_show_dialog: bool) {
        let selectable_files = open_files(files, self.version);

        let all_succeeded = selectable_files.data.iter().all(|f| f.result.is_ok())
            && selectable_files.data.iter().all(|f| f.result.is_ok());

        if all_succeeded && !always_show_dialog {
            self.concat_and_open(selectable_files);
        } else {
            self.selectable_files = Some(selectable_files);
        }
    }

    pub fn concat_and_open(&mut self, selectable_files: SelectableFiles) {
        let data_len = selectable_files
            .data
            .iter()
            .filter(|f| f.selected)
            .filter_map(|f| f.result.as_ref().ok())
            .map(|d| d.len())
            .sum();
        let mut data = Vec::with_capacity(data_len);
        let mut data_files = Vec::with_capacity(data_len);
        for (p, d) in selectable_files
            .data
            .into_iter()
            .filter(|f| f.selected)
            .filter_map(|f| f.result.ok().map(|d| (f.file, d)))
        {
            data.extend_from_slice(&*d);
            data_files.push(p);
        }

        let temp_len = selectable_files
            .temp
            .iter()
            .filter(|f| f.selected)
            .filter_map(|f| f.result.as_ref().ok())
            .map(|t| t.len())
            .sum();
        let mut temp = Vec::with_capacity(temp_len);
        let mut temp_files = Vec::with_capacity(temp_len);
        for (p, t) in selectable_files
            .temp
            .into_iter()
            .filter(|f| f.selected)
            .filter_map(|f| f.result.ok().map(|d| (f.file, d)))
        {
            temp.extend_from_slice(&*t);
            temp_files.push(p);
        }

        let files = Files {
            dir: selectable_files.dir,
            data: data_files,
            temp: temp_files,
        };

        self.selectable_files = None;
        self.files = Some(files);
        self.data = Some(data::process_data(data, temp, &self.custom.plots));
    }
}

pub fn find_files(dir: PathBuf) -> Result<Files, data::Error> {
    fn filename(file: &Path) -> Option<&str> {
        if file.extension()? != "bin" {
            return None;
        }
        file.file_stem()?.to_str()
    }

    let mut data_paths: Vec<(String, PathBuf)> = Vec::new();
    let mut temp_paths: Vec<(String, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if let Some(name) = filename(&path) {
            if let Some(name) = name.strip_prefix("temperature") {
                let mut i = 0;
                for (other, _) in temp_paths.iter() {
                    if name < other.as_str() {
                        break;
                    }
                    i += 1;
                }
                temp_paths.insert(i, (name.to_string(), path));
            } else {
                let mut i = 0;
                for (other, _) in data_paths.iter() {
                    if name < other.as_str() {
                        break;
                    }
                    i += 1;
                }
                data_paths.insert(i, (name.to_string(), path));
            }
        }
    }

    Ok(Files {
        dir,
        data: data_paths.into_iter().map(|(_, p)| p).collect(),
        temp: temp_paths.into_iter().map(|(_, p)| p).collect(),
    })
}

fn open_files(files: Files) -> SelectableFiles {
    let mut data = Vec::new();
    for p in files.items.iter() {
        let result = open_file(p, version);
        data.push(SelectableFile::new(true, p.to_owned(), result));
    }

    SelectableFiles {
        dir: files.dir,
        items,
    }
}

fn open_file(path: &Path) -> Result<Vec<DataEntry>, data::Error> {
    let mut reader = BufReader::new(File::open(path)?);
    data::read_extend_data(&mut reader, &mut data)?;
    Ok(data)
}
