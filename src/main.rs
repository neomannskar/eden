use iced::widget::{button, column, container, row, text};
use iced::{Fill, Element};
use iced::Theme;
use iced::window;
use iced::{Size, Subscription};

#[derive(Debug, Clone)]
enum Message {
    Increment,
    WindowResized(Size),
}

#[derive(Default)]
pub struct State {
    value: u64,
}

fn update(counter: &mut State, message: Message) {
    match message {
        Message::Increment => counter.value += 1,
        Message::WindowResized(_) => {}
    }
}

fn view(counter: &State) -> Element<Message> {
    container(
        column![
            "Top",
            text(counter.value).size(20),
            button("Increment").on_press(Message::Increment),
            row!["Left", "Right"].spacing(10),
            "Bottom",
            container("I am a rounded box!").style(container::rounded_box),
            button("I am a styled button!").style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                match status {
                    button::Status::Active => {
                        button::Style::default()
                        .with_background(palette.success.strong.color)
                    }
                    _ => button::primary(theme, status),
                }
            })
        ]
        .spacing(10)
    )
    .padding(10)
    .center_x(Fill)
    .center_y(Fill)
    .into()
}

pub fn main() -> iced::Result {
    // iced::run("Eden", update, view)
    iced::application("Eden", update, view)
        .subscription(subscription)
        .theme(theme)
        .run()
}

fn subscription(_state: &State) -> Subscription<Message> {
    window::resize_events().map(|(_id, size)| Message::WindowResized(size))
}

fn theme(_state: &State) -> Theme {
    Theme::GruvboxDark
}
