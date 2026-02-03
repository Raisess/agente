use std::sync::Arc;

use iced::widget::{Column, column, row, scrollable, text, text_input};
use iced::{Element, Error, Length, Task, Theme, color};
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
                        __scroll_id: scrollable::Id::new("chat_scroll"),
                        input: String::default(),
                        thinking: false,
                        chat: Vec::default(),
                        processor,
                    },
                    Task::perform((async || {})(), Event::Load),
                )
            })
    }

    fn view(state: &State) -> Column<'_, Event> {
        let mut messages = state
            .chat
            .iter()
            .map(|item| {
                let (from, color) = match item.from {
                    ChatItemOwner::User => ("Me", color!(0x17d1b8)),
                    ChatItemOwner::System => ("Agente", color!(0xb7410e)),
                };

                let from = text::Span::new(format!("{from}: "))
                    .size(CHAT_ITEM_SIZE)
                    .color(color);

                let message = match item.r#type {
                    ChatItemType::Message => {
                        text::Span::new(item.message.clone())
                            .size(CHAT_ITEM_SIZE)
                            .color(color!(0xe3e4ed))
                    }
                    ChatItemType::Log => text::Span::new(format!(
                        "Error({})",
                        item.message.clone()
                    ))
                    .size(CHAT_ITEM_SIZE)
                    .color(color!(0xb51c1f)),
                };

                text::Rich::with_spans([from, message]).into()
            })
            .collect::<Vec<Element<Event>>>();

        if state.thinking {
            messages.push(
                text::Rich::with_spans([text::Span::new("Thinking...")
                    .size(CHAT_ITEM_SIZE)
                    .color(color!(0xb7410e))])
                .into(),
            );
        }

        let chat_column = column![
            scrollable(column(messages).spacing(1).padding(10))
                .id(state.__scroll_id.clone())
                .spacing(5)
                .width(Length::Fill)
                .height(Length::Fill),
            row([text_input("Prompt: ...", &state.input)
                .on_input(Event::Input)
                .on_submit(Event::Submit)
                .into()])
            .padding(10),
        ];

        chat_column
    }

    fn update(state: &mut State, event: Event) -> Task<Event> {
        match event {
            Event::Load(_) => Task::none(),
            Event::Input(new_value) => {
                state.input = new_value;
                Task::none()
            }
            Event::Submit => {
                state.thinking = true;

                let prompt = state.input.clone();
                state.input = String::default();

                state.chat.push(ChatItem {
                    r#type: ChatItemType::Message,
                    from: ChatItemOwner::User,
                    message: prompt.clone(),
                });

                let _ = scrollable::snap_to::<Event>(
                    state.__scroll_id.clone(),
                    scrollable::RelativeOffset::END,
                );

                let processor = state.processor.clone();
                Task::perform(
                    (async move || {
                        let mut processor = processor.lock().await;
                        match processor.handle(prompt).await {
                            Ok(output) => (
                                ChatItemType::Message,
                                output.unwrap_or_default(),
                            ),
                            Err(error) => (ChatItemType::Log, error.message()),
                        }
                    })(),
                    Event::PushResponse,
                )
            }
            Event::PushResponse(response) => {
                state.thinking = false;

                let item_type = response.0;
                state.chat.push(ChatItem {
                    r#type: item_type.clone(),
                    from: ChatItemOwner::System,
                    message: response.1,
                });

                let _ = scrollable::snap_to::<Event>(
                    state.__scroll_id.clone(),
                    scrollable::RelativeOffset::END,
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
    PushResponse((ChatItemType, String)),
}

struct State {
    __scroll_id: scrollable::Id,
    processor: Arc<Mutex<Processor>>,
    input: String,
    thinking: bool,
    chat: Vec<ChatItem>,
}

struct ChatItem {
    r#type: ChatItemType,
    from: ChatItemOwner,
    message: String,
}

enum ChatItemOwner {
    User,
    System,
}

#[derive(Clone, Debug)]
enum ChatItemType {
    Log,
    Message,
}

impl ToString for ChatItemOwner {
    fn to_string(&self) -> String {
        match self {
            ChatItemOwner::User => String::from("user"),
            ChatItemOwner::System => String::from("system"),
        }
    }
}
