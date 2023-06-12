#![allow(warnings)]

use frontend::Gui;

use tokio::{io, time::Instant};
pub use eframe::{run_native, App, egui::{self}};

pub mod frontend;
pub mod backend;

fn main() {
    let app = Gui::new();
    let win_options = eframe::NativeOptions::default();
    // win_options.initial_window_size = Some(egui::Vec2::new(1000.0, 800.0));
    run_native("Voicer",  win_options, Box::new(|cc| Box::new(app)));
}