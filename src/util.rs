use eframe::egui::{Slider, Ui};

pub fn ratio_slider(ui: &mut Ui, value: &mut f32, default_ratio: f32, range: f32) {
    let min = default_ratio / range;
    let max = default_ratio * range;
    ui.add(Slider::new(value, min..=max).logarithmic(true));
}
