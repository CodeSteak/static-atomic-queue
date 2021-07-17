use derive_more::From;
use std::sync::{Arc, Mutex};

use iced::{Application, Clipboard, Command, Container, Element, Length, Subscription};

use audio_processor_iced_design_system as design_system;
use std::time::Duration;
use ui::main_content_view;

pub mod services;
pub mod ui;

pub struct App {
    #[allow(dead_code)]
    plugin_host: Arc<Mutex<plugin_host_lib::TestPluginHost>>,
    main_content_view: main_content_view::MainContentView,
    start_result: Result<(), plugin_host_lib::audio_io::StartError>,
}

#[derive(Debug, Clone, From)]
pub enum AppMessage {
    Content(main_content_view::Message),
    None,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = AppMessage;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut plugin_host = plugin_host_lib::TestPluginHost::default();
        let start_result = plugin_host.start().map_err(|err| {
            log::error!("Failed to start host: {:?}", err);
            err
        });
        let plugin_host = Arc::new(Mutex::new(plugin_host));
        let (main_content_view, command) =
            main_content_view::MainContentView::new(plugin_host.clone());

        (
            App {
                plugin_host,
                main_content_view,
                start_result,
            },
            command.map(|msg| msg.into()),
        )
    }

    fn title(&self) -> String {
        String::from("plugin-host")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            AppMessage::Content(message) => self
                .main_content_view
                .update(message)
                .map(AppMessage::Content),
            _ => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::time::every(Duration::from_millis(16)).map(|_| AppMessage::None)
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let content = match &self.start_result {
            Ok(_) => self.main_content_view.view().map(AppMessage::Content),
            Err(err) => ui::start_error_view::StartErrorView::view(err).map(|_| AppMessage::None),
        };
        Container::new(content)
            .style(design_system::style::container::Container0)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}