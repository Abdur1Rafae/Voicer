#![allow(warnings)]

use eframe::Theme;
use egui::Vec2;
use frontend::Gui;

use tokio::{io, time::Instant};
pub use eframe::{run_native, App, egui::{self}};

pub mod frontend;
pub mod backend;

fn main() {
    let app = Gui::new();
    let mut win_options = eframe::NativeOptions::default();
    win_options.initial_window_size = Some(egui::Vec2::new(1200.0, 800.0));
    win_options.centered = true;
    win_options.resizable = false;
    run_native("Voicer",  win_options, Box::new(|cc| Box::new(app)));
}