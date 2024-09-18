use iced::{
    executor, Application, Command, Element, Length, Settings, Theme, Alignment,
};
use iced::widget::{Button, Column, Row, Text, TextInput};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

#[tokio::main]
async fn main() -> iced::Result {
    CircuitikzApp::run(Settings::default())
}

struct CircuitikzApp {
    x_input: String,
    y_input: String,
    message: String,
    is_generating: bool,
}

#[derive(Debug, Clone)]
enum Message {
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

                Command::perform(generate_pdf(x, y), Message::GenerationComplete)
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

async fn generate_pdf(x: String, y: String) -> Result<(), String> {
    // Validate input
    let x: usize = x.parse().map_err(|_| "Invalid X coordinate")?;
    let y: usize = y.parse().map_err(|_| "Invalid Y coordinate")?;

    // Generate LaTeX code
    let latex_code = format!(
        r#"\documentclass{{standalone}}
\usepackage{{circuitikz}}
\begin{{document}}
\begin{{circuitikz}}
    \draw[step=1cm, gray, very thin] (0,0) grid ({},{});
\end{{circuitikz}}
\end{{document}}
"#,
        x, y
    );

    // Write LaTeX code to a temporary file
    let latex_file_path = "grid.tex";
    tokio::fs::write(latex_file_path, latex_code)
        .await
        .map_err(|e| format!("Failed to write LaTeX file: {}", e))?;

    // Run pdflatex to generate PDF
    let pdflatex_status = TokioCommand::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg(latex_file_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| format!("Failed to run pdflatex: {}", e))?;

    if !pdflatex_status.success() {
        return Err("pdflatex failed to compile the LaTeX code.".into());
    }

    // Open the PDF with the system's default PDF viewer
    let pdf_file = "grid.pdf";

    open::that(pdf_file).map_err(|e| format!("Failed to open PDF: {}", e))?;

    // Cleanup auxiliary files (optional)
    let _ = tokio::fs::remove_file("grid.aux").await;
    let _ = tokio::fs::remove_file("grid.log").await;

    Ok(())
}
