#![windows_subsystem = "windows"]
use app::PlotApp;

use eframe::NativeOptions;

mod app;
mod data;
mod eval;
mod fs;
mod plot;
mod util;

const APP_NAME: &str = "s3plot";

fn main() -> anyhow::Result<()> {
    let options = NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(APP_NAME, options, Box::new(|c| Box::new(PlotApp::new(c))));
}
