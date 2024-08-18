use std::{collections::HashMap, sync::Arc};

use components::equations;
use desmoxide::{graph::expressions::{EquationType, ExpressionType, Expressions}, lang::{ast::{Ident, AST}, compiler::ir::{IRSegment, IRType}, expression_provider::ExpressionId}};
use graph::GraphRenderer;
use iced::{
    widget::{
        canvas::Cache,
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
    ShowError(Option<usize>),
    FocusGraph(usize),
}



struct Somsed<'a> {
    graph_caches: HashMap<ExpressionId, Cache>,
    expressions: Expressions<'a>,


    shown_error: Option<usize>,

    sidebar_width: f32,

    scale: f32,
    mid: Vector,
    resolution: u32,
}

impl<'a> Application for Somsed<'a> {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                scale: 100.0,
                mid: Vector { x: 0.0, y: 0.0 },
                resolution: 1000,
                sidebar_width: 300.0,

                graph_caches: HashMap::new(),
                expressions: Expressions::new(HashMap::new()),

                shown_error: None,
            },
            Command::none(),
        )
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let content = Canvas::new(&GraphRenderer::new(&, graph_caches, scale, mid, resolution))
            .width(Length::Fill)
            .height(Length::Fill);

        row![
            equations::view(
                &self.expressions,
                &self.parsed_expr,
                &mut self.shown_error,
                self.sidebar_width
            ),
            content
        ]
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
            Message::ShowError(i) => {
                self.shown_error = i;
                Command::none()
            }
            Message::FocusGraph(i) => focus(Id::new(format!("equation_{}", i))),
        }
    }

    fn title(&self) -> String {
        "Somsed".to_string()
    }
}
