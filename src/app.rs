use cosmic::app::{context_drawer, Core, Task};
use cosmic::dialog::ashpd::desktop::file_chooser::FileFilter;
use cosmic::dialog::file_chooser::{self};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::widget::{self, menu, nav_bar};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Element};
use reqwest::Url;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::api;
use crate::domain::DeviceName;
use crate::simulation::Simulation;

const REPOSITORY: &str = "https://github.com/pop-os/cosmic-app-template";
const APP_ICON: &[u8] = include_bytes!("../res/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    core: Core,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    nav_model: nav_bar::Model,
    client: Arc<api::Client>,
    simulations: Vec<api::SimulationOverview>,
    editor_content: widget::text_editor::Content,
    selected_simulation: Option<(Uuid, api::Simulation)>,
    text_display: Option<String>,
    has_error: Option<String>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    OpenRepositoryUrl,
    ToggleContextPage(ContextPage),
    FetchAllSimulations,
    SimulationsFetched(Vec<api::SimulationOverview>),
    OpenFile,
    Edit(widget::text_editor::Action),
    SelectSimulation(Uuid),
    FetchedSimulation(Uuid, api::Simulation),
    ShowSource(Uuid),
    ShowDeviceLog(Uuid, DeviceName),
    ShowAllDeviceLog(Uuid),
    FetchedText(Uuid, String),
    NewSimulation,
    Submit,
    CopyTextToClipboard,
    ReplaceEditorContent(String),
}

const DEFAULT_SIMULATION: &str = include_str!("../res/example_simulation.json");

/// Create a COSMIC application from the app model
impl Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "io.github.fhdo7100003.HaUi";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text("List simulations")
            .data::<Page>(Page::SimulationList);

        nav.insert()
            .text("Create simulation")
            .data::<Page>(Page::NewSimulation);

        let app = AppModel {
            core,
            context_page: ContextPage::default(),
            nav_model: nav,
            key_binds: HashMap::new(),
            client: Arc::new(api::Client {
                endpoint: Url::parse("http://localhost:8000").unwrap(),
                client: reqwest::Client::new(),
            }),
            simulations: Vec::new(),
            editor_content: widget::text_editor::Content::with_text(DEFAULT_SIMULATION),
            selected_simulation: None,
            text_display: None,
            has_error: None,
        };

        let client = app.client.clone();
        (
            app,
            Task::perform(
                async move { client.fetch_all_simulations().await.unwrap() },
                |res| Message::SimulationsFetched(res).into(),
            ),
        )
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![
            menu::Tree::with_children(
                menu::root("File"),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button("Open", None, MenuAction::OpenFile)],
                ),
            ),
            menu::Tree::with_children(
                menu::root("View"),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button("About", None, MenuAction::About)],
                ),
            ),
        ]);

        vec![menu_bar.into()]
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title("About"),
        })
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        // Activate the page in the model.
        self.nav_model.activate(id);

        self.update_title()
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<Self::Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_s, ..
        } = theme::active().cosmic().spacing;

        let page = self.nav_model.data::<Page>(self.nav_model.active());
        match page {
            Some(Page::SimulationList) => widget::row()
                .push(
                    widget::scrollable(widget::column::with_children(
                        self.simulations
                            .iter()
                            .map(|sim| {
                                widget::button::custom(
                                    widget::column()
                                        .push(widget::text(sim.id.to_string()))
                                        .push(widget::text(sim.timestamp.to_string())),
                                )
                                .on_press(Message::SelectSimulation(sim.id))
                                .width(Length::Fill)
                                .into()
                            })
                            .collect::<Vec<_>>(),
                    ))
                    .width(Length::FillPortion(1)),
                )
                .push_maybe(self.selected_simulation.as_ref().map(|(id, sim)| {
                    widget::column()
                        .push(widget::text(id.to_string()))
                        .push(
                            widget::row()
                                .push(widget::text("Result"))
                                .push(widget::text(format!("{} Wh", sim.res.result))),
                        )
                        .push(
                            widget::button::text("Show source simulation")
                                .on_press(Message::ShowSource(*id)),
                        )
                        .push(
                            widget::row()
                                .push(widget::text("Device Log"))
                                .push(
                                    widget::button::text("Show all")
                                        .on_press(Message::ShowAllDeviceLog(*id)),
                                )
                                .align_y(Alignment::Center),
                        )
                        .push(widget::scrollable(widget::column::with_children(
                            sim.devices
                                .iter()
                                .map(|dev| {
                                    widget::button::custom(widget::text(dev.as_str()))
                                        .on_press(Message::ShowDeviceLog(*id, dev.clone()))
                                        .width(Length::Fill)
                                        .into()
                                })
                                .collect::<Vec<_>>(),
                        )))
                        .spacing(space_xxs)
                        .width(Length::FillPortion(1))
                }))
                .push_maybe(self.text_display.as_ref().map(|text| {
                    widget::column()
                        .push(
                            widget::button::icon(widget::icon::from_name("edit-copy-symbolic"))
                                .on_press(Message::CopyTextToClipboard),
                        )
                        .push(widget::scrollable(widget::text(text)).width(Length::Fill))
                }))
                .spacing(space_s)
                .into(),
            Some(Page::NewSimulation) => widget::column()
                .push(
                    widget::row()
                        .push(widget::button::text("Open").on_press(Message::OpenFile))
                        .push(
                            widget::button::text("Reset to template")
                                .on_press(Message::NewSimulation),
                        )
                        .push(widget::button::text("Submit").on_press(Message::Submit)),
                )
                .push(widget::text_editor(&self.editor_content).on_action(Message::Edit))
                .push_maybe(self.has_error.as_ref().map(|err| widget::text::text(err)))
                .into(),
            None => widget::text("Select something you want to do on the left").into(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![])
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Submit => {
                let json = self.editor_content.text();
                // check before sending
                match serde_json::from_str::<Simulation>(&json) {
                    Ok(sim) => {
                        let client = self.client.clone();
                        return Task::perform(
                            async move { client.submit_simulation(&sim).await.unwrap() },
                            |_| Message::FetchAllSimulations.into(),
                        );
                    }
                    Err(e) => {
                        self.has_error = Some(format!("Failed parsing json: {}", e));
                    }
                }
            }
            Message::NewSimulation => {
                self.editor_content = widget::text_editor::Content::with_text(DEFAULT_SIMULATION);
            }
            Message::FetchAllSimulations => {
                let client = self.client.clone();
                return Task::perform(
                    async move { client.fetch_all_simulations().await.unwrap() },
                    |mut res| {
                        res.sort_unstable_by(|a, b| b.timestamp.cmp(&a.timestamp).reverse());
                        Message::SimulationsFetched(res).into()
                    },
                );
            }
            Message::ReplaceEditorContent(cont) => {
                self.editor_content = widget::text_editor::Content::with_text(&cont);
            }
            Message::CopyTextToClipboard => if let Some(content) = &self.text_display {},
            Message::ShowAllDeviceLog(id) => {
                let client = self.client.clone();
                return Task::perform(
                    async move { client.fetch_simulation_log(id).await.unwrap() },
                    move |text| Message::FetchedText(id, text).into(),
                );
            }
            Message::ShowSource(id) => {
                let client = self.client.clone();
                return Task::perform(
                    async move {
                        let ret = client.fetch_simulation_source(id).await.unwrap();
                        let json: serde_json::Value = serde_json::from_str(&ret).unwrap();
                        serde_json::to_string_pretty(&json).unwrap()
                    },
                    move |text| Message::FetchedText(id, text).into(),
                );
            }
            Message::ShowDeviceLog(id, ident) => {
                let client = self.client.clone();
                let ident = ident.clone();
                return Task::perform(
                    async move {
                        client
                            .fetch_simulation_log_by_device(id, &ident)
                            .await
                            .unwrap()
                    },
                    move |text| Message::FetchedText(id, text).into(),
                );
            }
            Message::FetchedText(source_id, text) => {
                if let Some((id, _)) = self.selected_simulation {
                    if source_id == id {
                        self.text_display = Some(text);
                    }
                }
            }
            Message::SelectSimulation(id) => {
                let client = self.client.clone();
                return Task::perform(
                    async move { client.fetch_simulation(id).await.unwrap() },
                    move |sim| Message::FetchedSimulation(id, sim).into(),
                );
            }
            Message::FetchedSimulation(id, sim) => {
                self.selected_simulation = Some((id, sim));
                self.text_display = None;
            }

            Message::OpenRepositoryUrl => {}

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::OpenFile => {
                return Task::perform(
                    async move {
                        let filter = FileFilter::new("Json").glob("*.json");
                        let file = file_chooser::open::Dialog::new()
                            .title("Choose simulation json")
                            .filter(filter)
                            .open_file()
                            .await
                            .unwrap();
                        std::fs::read_to_string(file.url().path()).unwrap()
                    },
                    |cont| Message::ReplaceEditorContent(cont).into(),
                )
            }
            Message::Edit(action) => self.editor_content.perform(action),
            Message::SimulationsFetched(sims) => {
                self.simulations = sims;
                self.nav_model.activate_position(0);
            }
        }
        Task::none()
    }
}

impl AppModel {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));

        let title = widget::text::title3("ha-ui");

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<Message> {
        let window_title = "ha-ui".to_string();

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

pub enum Page {
    SimulationList,
    NewSimulation,
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    OpenFile,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::OpenFile => Message::NewSimulation,
        }
    }
}
