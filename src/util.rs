use egui::{Slider, Ui};

use crate::motor::Mode;

pub fn ratio_slider(ui: &mut Ui, value: &mut f32, default_ratio: f32, range: f32) {
    ui.label("aspect ratio");

    let min = default_ratio / range;
    let max = default_ratio * range;
    ui.add(Slider::new(value, min..=max).logarithmic(true));
}

pub fn mode_toggle(ui: &mut Ui, mode: &mut Mode) {
    ui.label("grid mode");

    let mut checked = mode.is_split();
    ui.checkbox(&mut checked, "");
    *mode = Mode::from(checked);
}
