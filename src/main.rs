use std::collections::HashMap;

use components::sidebar;
use desmoxide::graph::expressions::{CompiledEquations, ExpressionId, Expressions};
use graph::GraphRenderer;
use iced::{
    alignment::Horizontal,
    overlay,
    widget::{
        canvas::Cache,
        container,
        pane_grid::{self, Axis, Content, Pane, ResizeEvent},
        row,
        text_input::{self, focus, Id},
        Canvas, TextInput,
    },
    Application, Length, Settings, Task, Vector,
};

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

mod components;
mod graph;

static DCG_FONT: &[u8; 45324] = include_bytes!("./dcg-icons-2024-08-02.ttf");

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    iced::application("Somsed", Somsed::update, Somsed::view)
        .font(DCG_FONT)
        .antialiasing(true)
        .run()
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

fn main() -> iced::Result {
    iced::application("Somsed", Somsed::update, Somsed::view)
        .font(DCG_FONT)
        .antialiasing(true)
        .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    Moved(Vector),
    Scaled(f32, Option<Vector>),
    EquationChanged(ExpressionId, String),
    EquationAdded(String),
    ShowError(Option<ExpressionId>),
    FocusExpr(usize),
    Resized(pane_grid::ResizeEvent),
}

enum PaneType {
    Graph,
    Sidebar,
}

struct Somsed {
    panes: pane_grid::State<PaneType>,
    graph_caches: HashMap<ExpressionId, Cache>,
    errors: HashMap<ExpressionId, String>,
    expressions: Expressions,

    compiled_eqs: CompiledEquations,

    shown_error: Option<ExpressionId>,

    url: String,

    sidebar_width: f32,

    scale: f32,
    mid: Vector,
    resolution: u32,
}

impl Default for Somsed {
    fn default() -> Self {
        let expressions = Expressions::new(HashMap::new());

        let (mut panes, pane) = pane_grid::State::new(PaneType::Sidebar);

        panes.split(Axis::Vertical, pane, PaneType::Graph);

        Self {
            panes,
            errors: HashMap::new(),
            compiled_eqs: CompiledEquations::default(),
            scale: 100.0,
            mid: Vector { x: 0.0, y: 0.0 },
            resolution: 1000,
            sidebar_width: 300.0,
            url: "".to_string(),

            graph_caches: HashMap::new(),
            expressions,

            shown_error: None,
        }
    }
}

impl Somsed {
    pub fn clear_caches(&mut self) {
        for (_, v) in &mut self.graph_caches {
            v.clear();
        }
    }
}

impl Somsed {
    fn view(&self) -> pane_grid::PaneGrid<'_, Message> {
        pane_grid::PaneGrid::new(&self.panes, move |_, id, _| match id {
            PaneType::Graph => Content::new(
                Canvas::new(GraphRenderer::new(
                    &self.compiled_eqs,
                    &self.graph_caches,
                    self.scale,
                    self.mid,
                    self.resolution,
                ))
                .width(Length::Fill)
                .height(Length::Fill),
            ),
            PaneType::Sidebar => pane_grid::Content::new(sidebar::view(
                &self.expressions.storage,
                &self.errors,
                &self.shown_error,
                self.sidebar_width,
            )),
        })
        .on_resize(10, Message::Resized)
        .width(Length::Fill)
        .height(Length::Fill)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Moved(p) => {
                self.mid = self.mid + p;
                self.clear_caches();
            }
            Message::EquationChanged(i, s) => {
                self.expressions.set_equation(i, s);
                self.errors = self.expressions.parse_all();

                self.compiled_eqs = self.expressions.compile_all(&mut self.errors);

                self.graph_caches[&i].clear();
            }
            Message::EquationAdded(s) => {
                self.expressions.add_equation(s);

                self.graph_caches
                    .insert(ExpressionId(self.expressions.max_id - 1), Cache::new());
                self.errors = self.expressions.parse_all();
                self.compiled_eqs = self.expressions.compile_all(&mut self.errors);
                return focus(Id::new(format!("equation_{}", self.expressions.max_id - 1)));
            }
            Message::Scaled(scale, mid) => {
                self.scale = scale;
                if let Some(mid) = mid {
                    self.mid = mid;
                }
                self.clear_caches();
            }
            Message::ShowError(i) => {
                self.shown_error = i;
            }
            Message::FocusExpr(i) => return focus(Id::new(format!("equation_{}", i))),
            Message::Resized(ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
            }
        };
        Task::none()
    }

    fn title(&self) -> String {
        "Somsed".to_string()
    }
}
