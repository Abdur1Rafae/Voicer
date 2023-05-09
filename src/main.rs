#![allow(warnings)]

use UI::Gui;

use tokio::{io, time::Instant};
use eframe::{run_native, epi::App, egui::{self}};

pub mod UI;
pub mod db_config;

fn main() {
    let app = Gui::new();
    let win_options = eframe::NativeOptions::default();
    // win_options.initial_window_size = Some(egui::Vec2::new(1000.0, 800.0));
    run_native(Box::new(app), win_options);
}