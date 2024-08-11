use graph::Graph;
use iced::{
    subscription,
    widget::{column, container, text, Canvas},
    Application, Command, Length, Point, Settings, Vector,
};

mod graph;

fn main() {
    Somsed::run(Settings {
        antialiasing: true,
        ..Default::default()
    })
    .unwrap();
}

#[derive(Debug)]
pub enum Message {
    Moved(Vector),
}

fn function(x: f32) -> f32 {
    return x * x;
}

struct Somsed {
    graph: Graph<fn(f32) -> f32>,
}

impl Application for Somsed {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_: ()) -> (Self, Command<Message>) {
        (
            Self {
                graph: Graph {
                    f: |x| x.sin(),
                    scale: 100.0,
                    mid: Vector { x: 0.0, y: 0.0 },
                    resolution: 1000,
                },
            },
            Command::none(),
        )
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        column![Canvas::new(&self.graph)
            .width(Length::Fill)
            .height(Length::Fill)]
        .into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::Moved(p) => {
                self.graph.mid = self.graph.mid + p;
                iced::Command::none()
            }
        }
    }

    fn title(&self) -> String {
        "Somsed".to_string()
    }
}
