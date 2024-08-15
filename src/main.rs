use components::equations;
use graph::GraphRenderer;
use iced::{
    widget::{
        row,
        text_input::{focus, Id},
        Canvas,
    },
    Application, Command, Length, Settings, Vector,
};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

mod components;
mod graph;
mod latex;
mod parser;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    Somsed::run(Settings {
        antialiasing: true,
        ..Default::default()
    })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

fn main() {
    Somsed::run(Settings {
        antialiasing: true,
        ..Default::default()
    })
    .unwrap();
}

#[derive(Debug, Clone)]
pub enum Message {
    Moved(Vector),
    Scaled(f32, Option<Vector>),
    EquationChanged(usize, String),
    EquationAdded(String),
    EquationMessage(equations::Message),
}

struct Somsed {
    graph_renderer: GraphRenderer,
    equations: equations::Equations,
}

impl Application for Somsed {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                graph_renderer: GraphRenderer::new(100.0, Vector { x: 0.0, y: 0.0 }, 1000),
                equations: Default::default(),
            },
            Command::none(),
        )
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let content = Canvas::new(&self.graph_renderer)
            .width(Length::Fill)
            .height(Length::Fill);

        row![self.equations.view(), content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::Moved(p) => {
                self.graph_renderer.mid = self.graph_renderer.mid + p;
                self.graph_renderer.clear_caches();
                iced::Command::none()
            }
            Message::EquationChanged(i, s) => {
                let err = self.graph_renderer.set_equation(i, &s).err();
                self.equations.set_equation(i, s, err);

                iced::Command::none()
            }
            Message::EquationAdded(s) => {
                let err = self.graph_renderer.add_equation(&s).err();
                self.equations.add_equation(s, err);
                iced::Command::none()
            }
            Message::Scaled(scale, mid) => {
                self.graph_renderer.scale = scale;
                if let Some(mid) = mid {
                    self.graph_renderer.mid = mid;
                }
                self.graph_renderer.clear_caches();
                iced::Command::none()
            }
            Message::EquationMessage(m) => self.equations.update(m),
        }
    }

    fn title(&self) -> String {
        "Somsed".to_string()
    }
}
