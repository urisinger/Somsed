use std::collections::HashMap;

use desmoxide::lang::{ast::AST, expression_provider::ExpressionId};
use iced::{
    font,
    widget::{
        button, column, container, mouse_area, row, scrollable, text,
        text::LineHeight,
        text_input::{focus, Id},
        tooltip, TextInput,
    },
    Color, Command, Element, Font, Length, Padding,
};

use super::icons;
use crate::Message;

pub fn view<'element>(
    equations: &'element HashMap<ExpressionId, String>,
    errors: &HashMap<ExpressionId, String>,
    shown_error: &Option<ExpressionId>,
    width: f32,
) -> Element<'element, crate::Message> {
    let mut elements = equations
        .iter()
        .map(|(i, equation)| {
            let input = TextInput::new("", equation)
                .on_input(move |s| Message::EquationChanged(*i, s))
                .size(20)
                .padding(Padding {
                    top: 10.0,
                    bottom: 10.0,
                    right: 0.0,
                    left: 0.0,
                })
                .line_height(LineHeight::Absolute(30.0.into()))
                .id(Id::new(format!("equation_{}", i.0)))
                .width(Length::Fill);

            let show_err = mouse_area(
                container(icons::error())
                    .width(Length::Fixed(35.0))
                    .height(Length::Fixed(50.0)),
            )
            .on_enter(Message::ShowError(Some(*i)))
            .on_exit(Message::ShowError(None));

            let left: Element<crate::Message> = if let Some(i) = *shown_error {
                if let Some(err) = &errors.get(&i) {
                    tooltip(
                        show_err,
                        container(text(err).style(iced::theme::Text::Color(Color::WHITE)))
                            .padding(5)
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .style(styles::floating_box()),
                        tooltip::Position::Bottom,
                    )
                    .into()
                } else {
                    show_err.into()
                }
            } else {
                show_err.into()
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
    use iced::{widget::container, Border, Color, Shadow, Vector};

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
