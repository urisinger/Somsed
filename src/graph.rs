use iced::{
    event::Status,
    mouse::{self, Cursor},
    widget::canvas::{event, Event, Frame, Geometry, Path, Program, Stroke},
    Color, Point, Theme, Vector,
};

use crate::Message;

pub struct Graph<F>
where
    F: Fn(f32) -> f32,
{
    pub f: F,

    pub scale: f32,
    pub mid: Vector,
    pub resolution: u32,
}

impl<F> Graph<F>
where
    F: Fn(f32) -> f32,
{
    pub fn get_points(&self, range: f32) -> Vec<Vector> {
        let mut points = Vec::new();

        let min = self.mid.x - range / 2.0;
        let dx = range / self.resolution as f32;

        for i in 0..self.resolution {
            let x = i as f32 * dx + min;
            let point = Vector { x, y: (self.f)(x) };
            points.push(point);
        }

        points
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

impl<F> Program<Message> for Graph<F>
where
    F: Fn(f32) -> f32,
{
    type State = GraphState;
    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let points = self.get_points(bounds.width / self.scale);
        for i in 0..points.len() - 1 {
            let (point, next_point) = (points[i], points[i + 1]);

            frame.stroke(
                &Path::line(
                    Point {
                        x: (point.x - self.mid.x) * self.scale as f32 + bounds.width / 2.0,
                        y: (point.y - self.mid.y) * self.scale as f32 + bounds.height / 2.0,
                    },
                    Point {
                        x: (next_point.x - self.mid.x) * self.scale as f32 + bounds.width / 2.0,
                        y: (next_point.y - self.mid.y) * self.scale as f32 + bounds.height / 2.0,
                    },
                ),
                Stroke::default().with_width(4.0),
            );
        }

        vec![frame.into_geometry()]
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
                _ => (event::Status::Ignored, None),
            },
            _ => (event::Status::Ignored, None),
        }
    }
}
