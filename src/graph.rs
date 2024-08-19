use std::{collections::HashMap, fmt::Debug};

use anyhow::{anyhow, Result};
use desmoxide::{
    graph::expressions::{CompiledEquations, Expressions},
    lang::{
        ast::AST,
        compiler::{
            backends::interpreter::{eval, EvalError},
            ir::IRSegment,
            value::{IRValue, Number},
        },
        expression_provider::ExpressionId,
    },
};
use iced::{
    event::Status,
    mouse::{self, Cursor},
    widget::canvas::{event, Cache, Event, Frame, Geometry, Path, Program, Stroke},
    Color, Point, Theme, Vector,
};

use crate::Message;

pub struct GraphRenderer<'a> {
    scale: f32,
    mid: Vector,
    resolution: u32,

    exprs: &'a CompiledEquations,
    graph_caches: &'a HashMap<ExpressionId, Cache>,
}

impl<'a> GraphRenderer<'a> {
    pub fn new(
        exprs: &'a CompiledEquations,
        graph_caches: &'a HashMap<ExpressionId, Cache>,
        scale: f32,
        mid: Vector,
        resolution: u32,
    ) -> Self {
        Self {
            exprs,
            graph_caches,
            scale,
            mid,
            resolution,
        }
    }
}

pub fn points(
    ast: &IRSegment,
    range: f32,
    mid: Vector,
    resolution: u32,
) -> Result<Vec<Option<Vector>>> {
    let mut points = Vec::new();

    let min = mid.x - range / 2.0;
    let dx = range / resolution as f32;
    let mut args = Vec::new();
    args.push(IRValue::Number(0.0.into()));

    args.push(IRValue::Number(0.0.into()));
    for i in 0..resolution {
        let x = i as f64 * dx as f64 + min as f64;
        args[0] = IRValue::Number(x.into());
        let point = Some(Vector {
            x: x as f32,
            y: match eval(ast, args.clone())? {
                IRValue::Number(Number::Double(y)) => y,
                _ => return Err(anyhow!("unexpected number")),
            } as f32,
        });
        points.push(point);
    }

    Ok(points)
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

impl<'a> Program<Message> for GraphRenderer<'a> {
    type State = GraphState;
    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        let graphs = self.exprs.compiled_equations.iter().map(|(i, graph)| {
            self.graph_caches[i].draw(renderer, bounds.size(), |frame| {
                match points(
                    graph.0.as_ref().unwrap(),
                    bounds.width / self.scale,
                    self.mid,
                    self.resolution,
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
                    Err(e) => eprintln!(
                        "error in eval, {} code is: {:?}",
                        e,
                        graph.0.as_ref().unwrap()
                    ),
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
