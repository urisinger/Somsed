pub mod equations {
    use std::collections::HashMap;

    use desmoxide::lang::ast::AST;
    use iced::{
        widget::{
            button, column, container, mouse_area, row, scrollable, text,
            text::LineHeight,
            text_input::{focus, Id},
            tooltip, TextInput,
        },
        Color, Command, Element, Length, Padding,
    };

    use crate::Message;

    pub struct Equations {
        equations: Vec<String>,
        errors: Vec<Option<String>>,
        width: f32,
        shown_error: Option<usize>,
    }

    pub fn view<'a, 'element>(
        equations: &HashMap<String>,
        parsed_equations: &HashMap<Result<AST, String>>,
        shown_error: &mut Option<usize>,
        width: f32,
    ) -> Element<'element, crate::Message> {
        let mut elements = equations
            .iter()
            .enumerate()
            .map(|(i, equation)| {
                let input = TextInput::new("", equation)
                    .on_input(move |s| crate::Message::EquationChanged(i, s))
                    .size(20)
                    .padding(Padding {
                        top: 10.0,
                        bottom: 10.0,
                        right: 0.0,
                        left: 0.0,
                    })
                    .line_height(LineHeight::Absolute(30.0.into()))
                    .id(Id::new(format!("equation_{i}")))
                    .width(Length::Fill);

                let show_eq = mouse_area(
                    container("")
                        .width(Length::Fixed(35.0))
                        .height(Length::Fixed(50.0)),
                )
                .on_enter(Message::ShowError(Some(i)))
                .on_exit(Message::ShowError(None));

                let left: Element<crate::Message> = if let Some(i) = *shown_error {
                    if let Err(err) = &parsed_equations[i] {
                        tooltip(
                            show_eq,
                            container(text(err).style(iced::theme::Text::Color(Color::WHITE)))
                                .padding(5)
                                .width(Length::Shrink)
                                .height(Length::Shrink)
                                .style(styles::floating_box()),
                            tooltip::Position::Bottom,
                        )
                        .into()
                    } else {
                        show_eq.into()
                    }
                } else {
                    show_eq.into()
                };
                row![left, input].into()
            })
            .collect::<Vec<Element<crate::Message>>>();

        elements.push(
            button(container("").style(styles::add_eq()).width(Length::Fill))
                .style(iced::theme::Button::Text)
                .on_press(crate::Message::EquationAdded("".to_string()))
                .width(Length::Fill)
                .padding(0)
                .height(Length::Fixed(50.0))
                .into(),
        );

        let sidebar = scrollable(column(elements))
            .width(Length::Fill)
            .height(Length::Fill);

        let view = container(sidebar)
            .width(Length::Fixed(width))
            .height(Length::Fill)
            .style(styles::sidebar());

        view.into()
    }

    mod styles {
        use iced::{widget::container, Background, Border, Color, Shadow, Vector};

        pub fn add_eq() -> container::Appearance {
            container::Appearance {
                border: Border {
                    width: 1.0,
                    color: Color::from_rgb8(240, 240, 240),
                    ..Default::default()
                },
                ..Default::default()
            }
        }

        pub fn graph_disable(highlited: bool) -> container::Appearance {
            if highlited {
                container::Appearance {
                    background: Some(Background::Color(Color::from_rgb8(106, 147, 210))),
                    ..Default::default()
                }
            } else {
                container::Appearance {
                    ..Default::default()
                }
            }
        }

        pub fn floating_box() -> container::Appearance {
            container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb8(102, 102, 102))),
                border: Border {
                    radius: 8.0.into(),
                    width: 0.3,
                    color: Color::from_rgb8(102, 102, 102),
                },
                ..Default::default()
            }
        }

        pub fn sidebar() -> container::Appearance {
            container::Appearance {
                border: Border {
                    width: 1.0,
                    radius: 0.5.into(),

                    color: Color::from_rgb8(204, 204, 204),
                },
                shadow: Shadow {
                    blur_radius: 5.0,
                    color: Color::from_rgb8(204, 204, 204),
                    offset: Vector::new(2.0, 0.0),
                },
                ..Default::default()
            }
        }
    }
}
