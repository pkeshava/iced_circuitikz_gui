// src/main.rs

use iced::{Application, Settings};

mod pdf_generator;
mod ui;

#[tokio::main]
async fn main() -> iced::Result {
    ui::CircuitikzApp::run(Settings::default())
}