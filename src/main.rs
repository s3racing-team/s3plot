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

fn main() {
    let options = NativeOptions::default();
    let res = eframe::run_native(
        APP_NAME,
        options,
        Box::new(|c| Ok(Box::new(PlotApp::new(c)))),
    );
    if let Err(err) = res {
        println!("{err}");
    }
}
