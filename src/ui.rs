// src/ui.rs

use iced::{
    executor, Application, Command, Element, Length, Theme, Alignment,
};
use iced::widget::{Button, Column, Row, Text, TextInput, PickList};

use crate::pdf_generator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Nmos,
}

impl ComponentType {
    const ALL: [ComponentType; 1] = [ComponentType::Nmos];
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::Nmos => write!(f, "nmos"),
        }
    }
}

pub struct CircuitikzApp {
    x_input: String,
    y_input: String,
    message: String,
    is_generating: bool,
    // New fields
    component_type: Option<ComponentType>,
    component_x_input: String,
    component_y_input: String,
    components: Vec<(ComponentType, usize, usize)>,
}

#[derive(Debug, Clone)]
pub enum Message {
    XInputChanged(String),
    YInputChanged(String),
    GeneratePressed,
    GenerationComplete(Result<(), String>),
    ComponentTypeSelected(ComponentType),
    ComponentXInputChanged(String),
    ComponentYInputChanged(String),
    AddComponentPressed,
}

impl Application for CircuitikzApp {
    type Executor = executor::Default; // Use Tokio executor
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
                component_type: Some(ComponentType::Nmos),
                component_x_input: "5".to_string(),
                component_y_input: "5".to_string(),
                components: Vec::new(),
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
            Message::ComponentTypeSelected(component) => {
                self.component_type = Some(component);
                Command::none()
            }
            Message::ComponentXInputChanged(value) => {
                self.component_x_input = value;
                Command::none()
            }
            Message::ComponentYInputChanged(value) => {
                self.component_y_input = value;
                Command::none()
            }
            Message::AddComponentPressed => {
                if let Some(component_type) = self.component_type {
                    let x_result = self.component_x_input.parse::<usize>();
                    let y_result = self.component_y_input.parse::<usize>();
                    if let (Ok(x), Ok(y)) = (x_result, y_result) {
                        self.components.push((component_type, x, y));
                        self.message = format!(
                            "Added component {:?} at ({}, {})",
                            component_type, x, y
                        );
                    } else {
                        self.message = "Invalid component coordinates".to_string();
                    }
                } else {
                    self.message = "No component type selected".to_string();
                }
                Command::none()
            }
            Message::GeneratePressed => {
                self.is_generating = true;
                self.message = "Generating PDF...".to_string();

                let x = self.x_input.clone();
                let y = self.y_input.clone();
                let components = self.components.clone();

                Command::perform(
                    generate_grid_pdf(x, y, components),
                    Message::GenerationComplete,
                )
            }
            Message::GenerationComplete(result) => {
                self.is_generating = false;
                match result {
                    Ok(_) => {
                        self.message = "PDF generated and opened successfully.".to_string();
                        // Clear components after generation
                        self.components.clear();
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
        // Grid input fields
        let x_input = TextInput::new(
            "Grid Width (X)",
            &self.x_input,
        )
        .padding(10)
        .size(20)
        .width(Length::Fixed(100.0))
        .on_input(Message::XInputChanged);

        let y_input = TextInput::new(
            "Grid Height (Y)",
            &self.y_input,
        )
        .padding(10)
        .size(20)
        .width(Length::Fixed(100.0))
        .on_input(Message::YInputChanged);

        let grid_row = Row::new()
            .spacing(10)
            .push(x_input)
            .push(y_input);

        // Component selection fields
        let component_pick_list = PickList::new(
            &ComponentType::ALL[..],
            self.component_type,
            Message::ComponentTypeSelected,
        )
        .width(Length::Fixed(100.0));

        let component_x_input = TextInput::new(
            "Comp X",
            &self.component_x_input,
        )
        .padding(10)
        .size(20)
        .width(Length::Fixed(80.0))
        .on_input(Message::ComponentXInputChanged);

        let component_y_input = TextInput::new(
            "Comp Y",
            &self.component_y_input,
        )
        .padding(10)
        .size(20)
        .width(Length::Fixed(80.0))
        .on_input(Message::ComponentYInputChanged);

        let add_component_button = Button::new(
            Text::new("Add Component").size(16),
        )
        .padding(10)
        .on_press(Message::AddComponentPressed);

        let component_row = Row::new()
            .spacing(10)
            .push(component_pick_list)
            .push(component_x_input)
            .push(component_y_input)
            .push(add_component_button);

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
            .push(grid_row)
            .push(component_row)
            .push(generate_button)
            .push(Text::new(&self.message).size(16));

        content.into()
    }
}

// Place `generate_grid_pdf` function here
async fn generate_grid_pdf(
    x: String,
    y: String,
    components: Vec<(ComponentType, usize, usize)>,
) -> Result<(), String> {
    // Validate input
    let x: usize = x.parse().map_err(|_| "Invalid grid X coordinate")?;
    let y: usize = y.parse().map_err(|_| "Invalid grid Y coordinate")?;

    // Define LaTeX preamble and body
    let preamble = r#"\documentclass{standalone}
\usepackage{tikz}
\usepackage{circuitikz}"#;

    let mut components_code = String::new();

    for (component_type, comp_x, comp_y) in components {
        let code = match component_type {
            ComponentType::Nmos => format!(
                r#"    \draw ({},{}) node[nmos] {{}};"#, comp_x, comp_y
            ),
        };
        components_code.push_str(&code);
        components_code.push('\n');
    }

    let body = format!(
        r#"\begin{{circuitikz}}
    \draw[step=1cm, gray, very thin] (0,0) grid ({},{});
{}
\end{{circuitikz}}"#,
        x, y, components_code
    );

    // Call the generate_pdf function from pdf_generator module
    pdf_generator::generate_pdf(preamble, &body, "grid")
        .await
}