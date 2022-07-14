use std::path::{Path, PathBuf};

use egui::{Slider, Ui};

pub fn ratio_slider(ui: &mut Ui, value: &mut f32, default_ratio: f32, range: f32) {
    let min = default_ratio / range;
    let max = default_ratio * range;
    ui.add(
        Slider::new(value, min..=max)
            .logarithmic(true)
            .text("aspect ratio"),
    );
}

pub fn format_time(t: f64) -> String {
    let sub_sec = (t.fract() * 100.0).round() as usize;

    let secs = t as usize;
    let s = secs % 60;
    let m = secs / 60 % 60;
    let h = secs / (60 * 60);
    if h == 0 {
        format!("{m:02}:{s:02}.{sub_sec}")
    } else {
        format!("{h:02}:{m:02}:{s:02}.{sub_sec}")
    }
}

pub fn common_parent_dir<'a>(mut files: impl Iterator<Item = &'a PathBuf>) -> Option<&'a Path> {
    let first = files.next()?;
    let parent = first.parent()?;

    for f in files {
        if f.parent()? != parent {
            return None;
        }
    }

    Some(parent)
}
