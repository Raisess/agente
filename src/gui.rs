use std::sync::Arc;

use iced::widget::{Column, column, text, text_input};
use iced::{Element, Error, Renderer, Task, Theme, color};
use tokio::sync::Mutex;

use agente_domain::ports::agent::{MessageRequest, MessageRole};

use crate::processor::Processor;

#[derive(Clone, Debug)]
enum Event {
    Load(()),
    Input(String),
    Submit,
    PushResponse(Vec<String>),
}

struct State {
    input: String,
    chat: Vec<MessageRequest>,
    processor: Arc<Mutex<Processor>>,
}

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
            .map(|message| {
                text(&message.content)
                    .size(10)
                    .color(color!(0x0000ff))
                    .into()
            })
            .collect();

        let mut chat_column = column![
            text_input::<Event, Theme, Renderer>("Prompt: ...", &state.input)
                .on_input(Event::Input)
                .on_submit(Event::Submit)
                .into()
        ];

        for message_widget in messages {
            chat_column = chat_column.push(message_widget);
        }

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
                let prompt = state.input.clone();
                state.input = String::default();

                state.chat.push(MessageRequest {
                    role: MessageRole::User,
                    content: prompt.clone(),
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
                        .map(|content| MessageRequest {
                            role: MessageRole::Assistant,
                            content,
                        })
                        .collect::<Vec<_>>(),
                );
                Task::none()
            }
        }
    }
}
