use std::sync::Arc;

use iced::widget::{Column, column, text, text_input};
use iced::{Element, Error, Renderer, Task, Theme, color};
use tokio::sync::Mutex;

use crate::processor::Processor;

const CHAT_ITEM_SIZE: u16 = 18;

pub struct GUI;

impl GUI {
    pub fn run(processor: Arc<Mutex<Processor>>) -> Result<(), Error> {
        iced::application("Agente", Self::update, Self::view)
            .theme(|_| Theme::Dark)
            .centered()
            .run_with(|| {
                (
                    State {
                        input: String::default(),
                        chat: Vec::default(),
                        processor,
                    },
                    Task::perform((async || {})(), Event::Load),
                )
            })
    }

    fn view(state: &State) -> Column<'_, Event> {
        let messages: Vec<Element<Event>> = state
            .chat
            .iter()
            .map(|item| {
                let (from, color) = match item.from {
                    ChatItemOwner::User => ("Me", color!(0xCDE7F2)),
                    ChatItemOwner::System => ("Agente", color!(0xB7410E)),
                };

                let from = text::Span::new(format!("{from}: "))
                    .size(CHAT_ITEM_SIZE)
                    .color(color);
                let message = text::Span::new(item.message.clone())
                    .size(CHAT_ITEM_SIZE)
                    .color(color!(0xFFFFFF));

                text::Rich::with_spans([from, message]).into()
            })
            .collect();

        let mut chat_column = column![
            text_input::<Event, Theme, Renderer>("Prompt: ...", &state.input)
                .on_input(Event::Input)
                .on_submit(Event::Submit)
        ];

        for message_widget in messages {
            chat_column = chat_column.push(message_widget);
        }

        chat_column.padding(10)
    }

    fn update(state: &mut State, event: Event) -> Task<Event> {
        match event {
            Event::Load(_) => Task::none(),
            Event::Input(new_value) => {
                state.input = new_value;
                Task::none()
            }
            Event::Submit => {
                let prompt = state.input.clone();
                state.input = String::default();

                state.chat.push(ChatItem {
                    from: ChatItemOwner::User,
                    message: prompt.clone(),
                });

                let processor = state.processor.clone();
                Task::perform(
                    (async move || {
                        let mut processor = processor.lock().await;
                        let output = processor.handle(prompt).await;
                        output.unwrap_or_default()
                    })(),
                    Event::PushResponse,
                )
            }
            Event::PushResponse(response) => {
                state.chat.append(
                    &mut response
                        .into_iter()
                        .map(|message| ChatItem {
                            from: ChatItemOwner::System,
                            message,
                        })
                        .collect::<Vec<_>>(),
                );
                Task::none()
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Event {
    Load(()),
    Input(String),
    Submit,
    PushResponse(Vec<String>),
}

struct State {
    processor: Arc<Mutex<Processor>>,
    input: String,
    chat: Vec<ChatItem>,
}

struct ChatItem {
    from: ChatItemOwner,
    message: String,
}

enum ChatItemOwner {
    User,
    System,
}

impl ToString for ChatItemOwner {
    fn to_string(&self) -> String {
        match self {
            ChatItemOwner::User => String::from("user"),
            ChatItemOwner::System => String::from("system"),
        }
    }
}
