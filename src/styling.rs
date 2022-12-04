pub struct CardStyle {}

impl iced::widget::container::StyleSheet for CardStyle {
    type Style = iced::theme::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let mut style = iced::widget::container::Appearance::default();
        style.border_color = iced::Color::BLACK;
        style.border_width = 2.0;
        style.border_radius = 15.0;
        style.background = Some(iced::Background::Color(iced::Color::from_rgba8(
            77, 166, 255, 0.2,
        )));
        style
    }
}

pub fn card_style() -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(CardStyle {}))
}

pub struct InvisibleCardButton {}

impl iced::widget::button::StyleSheet for InvisibleCardButton {
    type Style = iced::theme::Theme;
    fn active(&self, _theme: &iced::theme::Theme) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            shadow_offset: iced::Vector::default(),
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: iced::Color::TRANSPARENT,
            text_color: iced::Color::BLACK,
        }
    }
}

pub fn invisible_button() -> iced::theme::Button {
    iced::theme::Button::Custom(Box::new(InvisibleCardButton {}))
}

pub struct CorrectCardStyle {}

impl iced::widget::container::StyleSheet for CorrectCardStyle {
    type Style = iced::theme::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let mut appearance = iced::widget::container::Appearance::default();
        appearance.border_color = iced::Color::BLACK;
        appearance.border_width = 2.0;
        appearance.border_radius = 15.0;
        appearance.background = Some(iced::Background::Color(iced::Color::from_rgba8(
            77, 255, 166, 0.2,
        )));
        appearance
    }
}

pub fn correct_card_style() -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(CorrectCardStyle {}))
}

pub struct WrongCardStyle {}

impl iced::widget::container::StyleSheet for WrongCardStyle {
    type Style = iced::theme::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let mut appearance = iced::widget::container::Appearance::default();
        appearance.border_color = iced::Color::BLACK;
        appearance.border_width = 2.0;
        appearance.border_radius = 15.0;
        appearance.background = Some(iced::Background::Color(iced::Color::from_rgba8(
            255, 166, 77, 0.2,
        )));
        appearance
    }
}

pub fn wrong_card_style() -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(WrongCardStyle {}))
}

pub struct WrongTextInput {}

impl iced::widget::text_input::StyleSheet for WrongTextInput {
    type Style = iced::theme::Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: iced::Background::Color(iced::Color::from_rgba(255., 230., 234., 1.)),
            border_radius: 1.,
            border_width: 1.,
            border_color: iced::Color::from_rgb(255., 0., 0.),
        }
    }

    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: iced::Background::Color(iced::Color::from_rgba(255., 220., 224., 1.)),
            border_radius: 1.,
            border_width: 1.,
            border_color: iced::Color::from_rgb(255., 0., 0.),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> iced::Color {
        iced::Color::BLACK
    }

    fn value_color(&self, _style: &Self::Style) -> iced::Color {
        iced::Color::BLACK
    }

    fn selection_color(&self, _style: &Self::Style) -> iced::Color {
        iced::Color::from_rgba8(77, 166, 255, 0.2)
    }
}

pub fn wrong_tex_input_style() -> iced::theme::TextInput {
    iced::theme::TextInput::Custom(Box::new(WrongTextInput {}))
}

pub fn score_text_color(score: f32) -> iced::Color {
    if score < 10.0 {
        iced::Color::from_rgba8(255, 0, 0, 1.0)
    } else if 10.0 <= score && score < 20.0 {
        iced::Color::from_rgba8(255, 42, 0, 1.0)
    } else if 20.0 <= score && score < 30.0 {
        iced::Color::from_rgba8(255, 85, 0, 1.0)
    } else if 30.0 <= score && score < 40.0 {
        iced::Color::from_rgba8(255, 128, 0, 1.0)
    } else if 40.0 <= score && score < 50.0 {
        iced::Color::from_rgba8(255, 170, 0, 1.0)
    } else if 50.0 <= score && score < 60.0 {
        iced::Color::from_rgba8(255, 212, 0, 1.0)
    } else if 60.0 <= score && score < 70.0 {
        iced::Color::from_rgba8(212, 255, 0, 1.0)
    } else if 70.0 <= score && score < 80.0 {
        iced::Color::from_rgba8(170, 255, 0, 1.0)
    } else if 80.0 <= score && score < 90.0 {
        iced::Color::from_rgba8(128, 255, 0, 1.0)
    } else if 90.0 <= score && score < 100.0 {
        iced::Color::from_rgba8(85, 255, 0, 1.0)
    } else if 100.0 == score {
        iced::Color::from_rgba8(0, 255, 0, 1.0)
    } else {
        iced::Color::from_rgba8(255, 255, 255, 1.0)
    }
}
