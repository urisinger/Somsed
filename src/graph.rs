use std::{collections::HashMap, fmt::Debug};

use iced::{
    event::Status,
    mouse::{self, Cursor},
    widget::canvas::{event, Cache, Event, Frame, Geometry, Path, Program, Stroke},
    Color, Point, Theme, Vector,
};

use crate::{
    latex::SubscriptSymbol,
    parser::{tokenizer::tokenize, EvalError, Node},
    Message,
};

pub struct Graph {
    pub eq: Node,
    graph_cache: Cache,
}

pub struct GraphRenderer {
    pub scale: f32,
    pub mid: Vector,
    pub resolution: u32,

    pub graphs: Vec<Option<Graph>>,

    symbols: HashMap<SubscriptSymbol, f32>,
}

impl GraphRenderer {
    pub fn new(scale: f32, mid: Vector, resolution: u32) -> Self {
        Self {
            scale,
            mid,
            resolution,
            graphs: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn add_equation(&mut self, eq: &str) -> Result<(), String> {
        self.graphs.push(None);
        self.set_equation(self.graphs.len() - 1, eq)
    }

    pub fn set_equation(&mut self, i: usize, eq: &str) -> Result<(), String> {
        let tokens = tokenize(eq.chars()).map_err(|e| e.to_string())?;
        let parsed_eq = Node::parse(tokens).map_err(|e| e.to_string())?;
        self.graphs[i] = Some(Graph::new(parsed_eq));
        Ok(())
    }

    pub fn clear_caches(&self) {
        for graph in &self.graphs {
            graph.as_ref().map(|g| g.clear_cache());
        }
    }
}

impl Graph {
    pub fn new(eq: Node) -> Self {
        Self {
            eq,
            graph_cache: Cache::new(),
        }
    }
    pub fn points(
        &self,
        range: f32,
        mid: Vector,
        resolution: u32,
        vars: &mut HashMap<SubscriptSymbol, f32>,
    ) -> Result<Vec<Option<Vector>>, String> {
        let mut points = Vec::new();

        let min = mid.x - range / 2.0;
        let dx = range / resolution as f32;
        vars.insert(SubscriptSymbol::from('x'), 0.0);

        for i in 0..resolution {
            let x = i as f32 * dx + min;
            vars.get_mut(&SubscriptSymbol::from('x')).map(|v| *v = x);
            let point = match self.eq.evaluate(vars) {
                Ok(y) => Some(Vector { x, y }),
                Err(EvalError::DivideByZeroError) => None,
                Err(EvalError::UnknownSymbol(s)) => return Err(format!("unknown symbol: {:?}", s)),
            };
            points.push(point);
        }

        Ok(points)
    }

    pub fn clear_cache(&self) {
        self.graph_cache.clear()
    }
}

pub enum GraphState {
    None,
    Moving { start: Point },
}

impl Default for GraphState {
    fn default() -> Self {
        Self::None
    }
}

impl Program<Message> for GraphRenderer {
    type State = GraphState;
    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        let graphs = self.graphs.iter().filter_map(|g| g.as_ref()).map(|graph| {
            graph.graph_cache.draw(renderer, bounds.size(), |frame| {
                let mut symbols = self.symbols.clone();

                match graph.points(
                    bounds.width / self.scale,
                    self.mid,
                    self.resolution,
                    &mut symbols,
                ) {
                    Ok(points) => {
                        for i in 0..points.len() - 1 {
                            match (points[i], points[i + 1]) {
                                (Some(point), Some(next_point)) => {
                                    frame.stroke(
                                        &Path::line(
                                            Point {
                                                x: (point.x - self.mid.x) * self.scale as f32
                                                    + bounds.width / 2.0,
                                                y: (point.y - self.mid.y) * self.scale as f32
                                                    + bounds.height / 2.0,
                                            },
                                            Point {
                                                x: (next_point.x - self.mid.x) * self.scale as f32
                                                    + bounds.width / 2.0,
                                                y: (next_point.y - self.mid.y) * self.scale as f32
                                                    + bounds.height / 2.0,
                                            },
                                        ),
                                        Stroke::default().with_width(4.0),
                                    );
                                }
                                _ => (),
                            }
                        }
                    }
                    Err(e) => eprintln!("{}", e),
                }
            })
        });

        let graphs = graphs.collect();

        graphs
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: event::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (Status, Option<Message>) {
        let Some(cursor_position) = cursor.position_in(bounds) else {
            return (event::Status::Ignored, None);
        };
        match event {
            Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    match *state {
                        GraphState::None => {
                            *state = GraphState::Moving {
                                start: cursor_position,
                            };
                        }
                        _ => {}
                    }
                    (event::Status::Captured, None)
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    *state = GraphState::None;
                    (event::Status::Captured, None)
                }
                mouse::Event::CursorMoved { .. } => match *state {
                    GraphState::Moving { start } => {
                        let diff = (start - cursor_position) * (1.0 / self.scale);
                        *state = GraphState::Moving {
                            start: cursor_position,
                        };
                        (event::Status::Captured, Some(Message::Moved(diff)))
                    }
                    GraphState::None => (event::Status::Ignored, None),
                },
                mouse::Event::WheelScrolled { delta } => match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                        let scaling = self.scale * (1.0 + y / 30.0);
                        let mid =
                            if let Some(cursor_to_center) = cursor.position_from(bounds.center()) {
                                let factor = scaling - self.scale;

                                Some(
                                    self.mid
                                        - Vector::new(
                                            cursor_to_center.x * factor / (self.scale * self.scale),
                                            cursor_to_center.y * factor / (self.scale * self.scale),
                                        ),
                                )
                            } else {
                                None
                            };
                        (event::Status::Captured, Some(Message::Scaled(scaling, mid)))
                    }
                },
                _ => (event::Status::Ignored, None),
            },
            _ => (event::Status::Ignored, None),
        }
    }
}
