use std::collections::HashMap;

use components::sidebar;
use desmoxide::{
    graph::expressions::{CompiledEquations, ExpressionId, Expressions},
    interop::{Expression, Graph},
};
use graph::GraphRenderer;
use iced::{
    alignment::Horizontal,
    overlay,
    widget::{
        self,
        canvas::Cache,
        container, mouse_area, opaque,
        pane_grid::{self, Axis, Content, Pane, ResizeEvent},
        row,
        text_input::{self, focus, Id},
        Canvas, Stack, TextInput,
    },
    Application, Color, Length, Padding, Settings, Task, Vector,
};

use clap::Parser;

use reqwest::header::{ACCEPT, CONTENT_TYPE};
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
    let options = Options::parse();
    iced::application("Somsed", Somsed::update, Somsed::view)
        .font(DCG_FONT)
        .antialiasing(true)
        .run_with(move || (Somsed::new(options), Task::none()))
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

    scale: f32,
    mid: Vector,
    resolution: u32,
}

#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
struct Options {
    /// url of the program to fetch
    #[arg(short, long)]
    url: Option<String>,
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

            graph_caches: HashMap::new(),
            expressions,

            shown_error: None,
        }
    }
}

impl Somsed {
    async fn get_url(url: &str) -> Graph {
        let res = reqwest::Client::new()
            .get(url)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .expect("Should send the request");

        let text = res.text().await.expect("Should get response text");

        serde_json::from_str(&text).expect("failed to deserealize graph")
    }
    fn new(options: Options) -> Self {
        let mut graph_caches = HashMap::new();
        let mut expressions = Expressions::new(if let Some(url) = options.url {
            let graph = futures_lite::future::block_on(Self::get_url(&url));
            graph
                .exprs()
                .into_iter()
                .filter_map(|expr| match expr {
                    Expression::Expression { id, latex, .. } => latex.as_ref().map(|latex| {
                        graph_caches.insert(ExpressionId(*id), Cache::new());
                        (ExpressionId(*id), latex.clone())
                    }),
                    _ => None,
                })
                .collect()
        } else {
            HashMap::new()
        });

        expressions.parse_all();

        let mut errors = HashMap::new();
        let compiled_eqs = expressions.compile_all(&mut errors);

        let (mut panes, pane) = pane_grid::State::new(PaneType::Sidebar);

        panes.split(Axis::Vertical, pane, PaneType::Graph);

        Self {
            panes,
            errors,
            compiled_eqs,
            scale: 100.0,
            mid: Vector { x: 0.0, y: 0.0 },
            resolution: 1000,

            expressions,
            graph_caches,

            shown_error: None,
        }
    }

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
            )),
        })
        .on_resize(10, Message::Resized)
        .width(Length::Fill)
        .height(Length::Fill)
    }

    pub fn clear_caches(&mut self) {
        for (_, v) in &mut self.graph_caches {
            v.clear();
        }
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
}
