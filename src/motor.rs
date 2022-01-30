use eframe::egui::plot::{Legend, Line, Plot, Value, Values};
use eframe::egui::{TextStyle, Ui};

use crate::app::PlotData;

pub trait FormatLabel {
    fn format_label(name: &str, value: &Value) -> String;
}

pub struct QuadValues {
    pub fl: Vec<Value>,
    pub fr: Vec<Value>,
    pub rl: Vec<Value>,
    pub rr: Vec<Value>,
}

pub struct Power;
impl FormatLabel for Power {
    fn format_label(_name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}s\np = {y}W")
    }
}

pub struct Speed;
impl FormatLabel for Speed {
    fn format_label(_name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("t = {x}s\nv = {y}km/h")
    }
}

pub struct Torque;
impl FormatLabel for Torque {
    fn format_label(name: &str, val: &Value) -> String {
        let x = (val.x * 1000.0).round() / 1000.0;
        let y = (val.y * 1000.0).round() / 1000.0;
        format!("{name}\nt = {x}s\nM = {y}Nm")
    }
}

pub fn plot_power(ui: &mut Ui, data: &PlotData, aspect_ratio: f32) {
    plot(
        ui,
        Power,
        aspect_ratio,
        [Line::new(Values::from_values(data.power.fl.clone())).name("power")],
        [Line::new(Values::from_values(data.power.fr.clone())).name("power")],
        [Line::new(Values::from_values(data.power.rl.clone())).name("power")],
        [Line::new(Values::from_values(data.power.rr.clone())).name("power")],
    );
}

pub fn plot_velocity(ui: &mut Ui, data: &PlotData, aspect_ratio: f32) {
    plot(
        ui,
        Speed,
        aspect_ratio,
        [Line::new(Values::from_values(data.velocity.fl.clone())).name("speed")],
        [Line::new(Values::from_values(data.velocity.fr.clone())).name("speed")],
        [Line::new(Values::from_values(data.velocity.rl.clone())).name("speed")],
        [Line::new(Values::from_values(data.velocity.rr.clone())).name("speed")],
    );
}

pub fn plot_torque(ui: &mut Ui, data: &PlotData, aspect_ratio: f32) {
    plot(
        ui,
        Torque,
        aspect_ratio,
        [
            Line::new(Values::from_values(data.torque_set.fl.clone())).name("set"),
            Line::new(Values::from_values(data.torque_real.fl.clone())).name("real"),
        ],
        [
            Line::new(Values::from_values(data.torque_set.fr.clone())).name("set"),
            Line::new(Values::from_values(data.torque_real.fr.clone())).name("real"),
        ],
        [
            Line::new(Values::from_values(data.torque_set.rl.clone())).name("set"),
            Line::new(Values::from_values(data.torque_real.rl.clone())).name("real"),
        ],
        [
            Line::new(Values::from_values(data.torque_set.rr.clone())).name("set"),
            Line::new(Values::from_values(data.torque_real.rr.clone())).name("real"),
        ],
    );
}

pub fn plot<T: FormatLabel, const COUNT: usize>(
    ui: &mut Ui,
    _: T,
    data_aspect: f32,
    lines_fl: [Line; COUNT],
    lines_fr: [Line; COUNT],
    lines_rl: [Line; COUNT],
    lines_rr: [Line; COUNT],
) {
    let h = ui.available_height() / 2.0
        - ui.fonts().row_height(TextStyle::Body)
        - ui.style().spacing.item_spacing.y;

    ui.columns(2, |uis| {
        let ui = &mut uis[0];
        ui.label("front left");
        Plot::new("fl_motor")
            .height(h)
            .data_aspect(data_aspect)
            .custom_label_func(move |n, v| T::format_label(n, v))
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in lines_fl {
                    ui.line(l);
                }
            });
        ui.label("rear left");
        Plot::new("rl_motor")
            .height(h)
            .data_aspect(data_aspect)
            .custom_label_func(move |n, v| T::format_label(n, v))
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in lines_rl {
                    ui.line(l);
                }
            });

        let ui = &mut uis[1];
        ui.label("front right");
        Plot::new("fr_motor")
            .height(h)
            .data_aspect(data_aspect)
            .custom_label_func(move |n, v| T::format_label(n, v))
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in lines_fr {
                    ui.line(l);
                }
            });
        ui.label("rear right");
        Plot::new("rr_motor")
            .height(h)
            .data_aspect(data_aspect)
            .custom_label_func(move |n, v| T::format_label(n, v))
            .legend(Legend::default())
            .show(ui, |ui| {
                for l in lines_rr {
                    ui.line(l);
                }
            });
    })
}
