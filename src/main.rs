use std::{collections::HashMap, sync::Arc};

use components::sidebar;
use desmoxide::{
    graph::expressions::{
        CompiledEquations, EquationType, ExpressionId, ExpressionType, Expressions,
    },
    lang::{
        ast::{Ident, AST},
        compiler::ir::{IRSegment, IRType},
    },
};
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

static DCG_FONT: &[u8; 45324] = include_bytes!("./dcg-icons-2024-08-02.ttf");

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let mut settings = Settings {
        antialiasing: true,
        ..Default::default()
    };

    settings.fonts.push(DCG_FONT.into());
    Somsed::run(settings).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn main() {
    let mut settings = Settings {
        antialiasing: true,
        ..Default::default()
    };

    settings.fonts.push(DCG_FONT.into());
    Somsed::run(settings).unwrap();
}

#[derive(Debug, Clone)]
pub enum Message {
    Moved(Vector),
    Scaled(f32, Option<Vector>),
    EquationChanged(ExpressionId, String),
    EquationAdded(String),
    ShowError(Option<ExpressionId>),
    FocusGraph(usize),
}

struct Somsed {
    graph_caches: HashMap<ExpressionId, Cache>,
    errors: HashMap<ExpressionId, String>,
    expressions: Expressions,

    compiled_eqs: CompiledEquations,

    shown_error: Option<ExpressionId>,

    sidebar_width: f32,

    scale: f32,
    mid: Vector,
    resolution: u32,
}

impl Somsed {
    pub fn clear_caches(&mut self) {
        for (_, v) in &mut self.graph_caches {
            v.clear();
        }
    }
}

impl Application for Somsed {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Message>) {
        let expressions = Expressions::new(HashMap::new());
        (
            Self {
                errors: HashMap::new(),
                compiled_eqs: CompiledEquations::default(),
                scale: 100.0,
                mid: Vector { x: 0.0, y: 0.0 },
                resolution: 1000,
                sidebar_width: 300.0,

                graph_caches: HashMap::new(),
                expressions,

                shown_error: None,
            },
            Command::none(),
        )
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let content = Canvas::new(GraphRenderer::new(
            &self.compiled_eqs,
            &self.graph_caches,
            self.scale,
            self.mid,
            self.resolution,
        ))
        .width(Length::Fill)
        .height(Length::Fill);

        row![
            sidebar::view(
                &self.expressions.storage,
                &self.errors,
                &self.shown_error,
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
                self.mid = self.mid + p;
                self.clear_caches();
                iced::Command::none()
            }
            Message::EquationChanged(i, s) => {
                self.expressions.set_equation(i, s);
                self.errors = self.expressions.parse_all();

                self.compiled_eqs = self.expressions.compile_all(&mut self.errors);

                self.graph_caches[&i].clear();
                iced::Command::none()
            }
            Message::EquationAdded(s) => {
                self.expressions.add_equation(s);

                self.graph_caches
                    .insert(ExpressionId(self.expressions.max_id - 1), Cache::new());
                self.errors = self.expressions.parse_all();
                self.compiled_eqs = self.expressions.compile_all(&mut self.errors);
                focus(Id::new(format!("equation_{}", self.expressions.max_id - 1)))
            }
            Message::Scaled(scale, mid) => {
                self.scale = scale;
                if let Some(mid) = mid {
                    self.mid = mid;
                }
                self.clear_caches();
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
