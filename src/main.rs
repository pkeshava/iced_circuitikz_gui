// src/main.rs

use iced::{Application, Settings};

mod pdf_generator;
mod ui;

fn main() -> iced::Result {
    ui::CircuitikzApp::run(Settings::default())
}