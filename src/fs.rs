use std::fmt::Write as _;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use egui::{Align2, Color32, Context, Id, LayerId, Order, Pos2, Rect, TextStyle, Vec2};
use serde::{Deserialize, Serialize};

use crate::data::{self, LogStream, SanityError};
use crate::PlotApp;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Files {
    pub dir: PathBuf,
    pub items: Vec<PathBuf>,
}

pub struct SelectableFiles {
    pub dir: PathBuf,
    pub by_header: Vec<Vec<SelectableFile>>,
    pub with_error: Vec<SelectableFile>,
}

pub struct SelectableFile {
    pub selected: bool,
    pub file: PathBuf,
    pub result: Result<LogStream, data::Error>,
    pub sanity_check: Result<(), SanityError>,
}

impl SelectableFile {
    pub fn new(
        seleted: bool,
        file: PathBuf,
        result: Result<LogStream, data::Error>,
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
            self.selectable_files = Some(open_files(files));
        }
    }

    pub fn try_open_files(&mut self, files: Files, always_show_dialog: bool) {
        let selectable_files = open_files(files);

        let all_succeeded = selectable_files.with_error.is_empty();
        let sanity_check_passed = selectable_files
            .by_header
            .iter()
            .all(|g| g.iter().all(|f| f.sanity_check.is_ok()));

        if all_succeeded && !always_show_dialog {
            self.concat_and_show(selectable_files);
        } else {
            self.selectable_files = Some(selectable_files);
        }
    }

    pub fn concat_and_show(&mut self, selectable_files: SelectableFiles) {
        let mut streams = Vec::with_capacity(selectable_files.by_header.len());
        let mut files = Vec::new();
        for group in selectable_files.by_header.into_iter() {
            let mut group = group
                .into_iter()
                .filter(|f| f.selected)
                .filter_map(|f| f.result.ok().map(|s| (f.file, s)))
                .collect::<Vec<_>>();

            let (first_file, first) = group.remove(0);
            files.push(first_file);

            let additional = group.iter().map(|(_, d)| d.len()).sum();
            first.reserve(additional);

            for (f, s) in group.into_iter() {
                first.extend(&s);
                files.push(f);
            }

            streams.push(group.remove(0));
        }

        let files = Files {
            dir: selectable_files.dir,
            items: files,
        };

        self.selectable_files = None;
        self.files = Some(files);
        self.data = Some(data::process_data(files, &self.custom.plots));
    }
}

fn find_files(dir: PathBuf) -> Result<Files, data::Error> {
    let mut items = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if path.extension().map_or(false, |e| e == "s3lg") {
            items.push(path);
        }
    }

    items.sort();

    Ok(Files { dir, items })
}

fn open_files(files: Files) -> SelectableFiles {
    let mut by_header: Vec<Vec<SelectableFile>> = Vec::new();
    let mut with_error = Vec::new();
    for f in files.items.iter() {
        let selectable_file = open_file(f);
        match &selectable_file.result {
            Ok(log_file) => {
                for group in by_header.iter_mut() {
                    if log_file.header_matches(group[0].result.as_ref().unwrap()) {
                        group.push(selectable_file);
                    }
                }
            }
            Err(_) => with_error.push(selectable_file),
        }
    }

    SelectableFiles {
        dir: files.dir,
        by_header,
        with_error,
    }
}

fn open_file(path: &Path) -> SelectableFile {
    let result = File::open(path).map_err(From::from).and_then(|f| {
        let mut reader = BufReader::new(f);
        data::read_file(&mut reader)
    });
    let sanity_check = match &result {
        Ok(l) => data::sanity_check(&l.entries),
        Err(_) => Ok(()),
    };
    SelectableFile {
        selected: sanity_check.is_ok(),
        file: path.to_path_buf(),
        result,
        sanity_check,
    }
}
