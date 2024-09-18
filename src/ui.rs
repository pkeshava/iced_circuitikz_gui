// src/ui.rs

use iced::{
    executor, Application, Command, Element, Length, Theme, Alignment,
};
use iced::widget::{Button, Column, Row, Text, TextInput};

use crate::pdf_generator;

pub struct CircuitikzApp {
    x_input: String,
    y_input: String,
    message: String,
    is_generating: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    XInputChanged(String),
    YInputChanged(String),
    GeneratePressed,
    GenerationComplete(Result<(), String>),
}

impl Application for CircuitikzApp {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (CircuitikzApp, Command<Message>) {
        (
            CircuitikzApp {
                x_input: "10".to_string(),
                y_input: "10".to_string(),
                message: "".to_string(),
                is_generating: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "CircuitikZ Grid Generator".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::XInputChanged(value) => {
                self.x_input = value;
                Command::none()
            }
            Message::YInputChanged(value) => {
                self.y_input = value;
                Command::none()
            }
            Message::GeneratePressed => {
                self.is_generating = true;
                self.message = "Generating PDF...".to_string();

                let x = self.x_input.clone();
                let y = self.y_input.clone();

                Command::perform(
                    generate_grid_pdf(x, y),
                    Message::GenerationComplete,
                )
            }
            Message::GenerationComplete(result) => {
                self.is_generating = false;
                match result {
                    Ok(_) => {
                        self.message = "PDF generated and opened successfully.".to_string();
                    }
                    Err(e) => {
                        self.message = format!("Error: {}", e);
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let x_input = TextInput::new(
            "X Coordinate",
            &self.x_input,
        )
        .padding(10)
        .size(20)
        .width(Length::Fixed(100.0))
        .on_input(Message::XInputChanged);

        let y_input = TextInput::new(
            "Y Coordinate",
            &self.y_input,
        )
        .padding(10)
        .size(20)
        .width(Length::Fixed(100.0))
        .on_input(Message::YInputChanged);

        let generate_button = if self.is_generating {
            Button::new(
                Text::new("Generate PDF").size(16),
            )
            .padding(10)
        } else {
            Button::new(
                Text::new("Generate PDF").size(16),
            )
            .padding(10)
            .on_press(Message::GeneratePressed)
        };

        let content = Column::new()
            .padding(20)
            .spacing(15)
            .align_items(Alignment::Center)
            .push(Text::new("CircuitikZ Grid Generator").size(30))
            .push(
                Row::new()
                    .spacing(10)
                    .push(x_input)
                    .push(y_input),
            )
            .push(generate_button)
            .push(Text::new(&self.message).size(16));

        content.into()
    }
}

// Async function to generate the grid PDF
async fn generate_grid_pdf(x: String, y: String) -> Result<(), String> {
    // Validate input
    let x: usize = x.parse().map_err(|_| "Invalid X coordinate")?;
    let y: usize = y.parse().map_err(|_| "Invalid Y coordinate")?;

    // Define LaTeX header and body
    let header = r#"\documentclass{standalone}
\usepackage{circuitikz}"#;

    let body = format!(
        r#"\begin{{circuitikz}}
    \draw[step=1cm, gray, very thin] (0,0) grid ({},{});
\end{{circuitikz}}"#,
        x, y
    );

    // Call the generate_pdf function from pdf_generator module
    pdf_generator::generate_pdf(header, &body, "grid")
        .await
}