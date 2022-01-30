use app::PlotApp;

use eframe::NativeOptions;

mod app;
mod custom;
mod data;
mod eval;
mod fs;
mod motor;
mod util;

fn main() -> anyhow::Result<()> {
    let app = PlotApp::default();
    let options = NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
