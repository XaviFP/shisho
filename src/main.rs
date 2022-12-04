mod client;
mod styling;
use crate::client::*;
use iced::widget::{
    button, column, container, horizontal_space, progress_bar, radio, row, scrollable, text,
    text_input,
};
use iced::{
    alignment::Horizontal,
    alignment::Vertical,
    keyboard::KeyCode,
    time::{Duration, Instant},
    Alignment, Application, Command, Element, Length, Padding, Settings, Subscription, Theme,
};

const ANSWER_DELAY: Duration = Duration::new(1, 0);
const RESULTS_DELAY: Duration = Duration::new(2, 0);

pub fn main() -> iced::Result {
    Shisho::run(Settings::default())
}

struct Shisho {
    signup: Signup,
    login: Login,
    first_login: bool,
    remote_error: Option<client::Error>,
    token: String,
    decks: Vec<Deck>,
    selected_deck: usize,
    selected_card: usize,
    selected_answers: Vec<Vec<bool>>,
    check: bool,
    answered: bool,
    last_tick: Instant,
    duration: Duration,
    already_selected: bool,
    fully_fetched: Vec<bool>,
    state: States,
    score: f32,
    edit_deck: EditDeck,
    pending_operation: PendingOperation,
}

#[derive(Debug)]
enum States {
    Welcome,
    Signup,
    Loaded,
    Details,
    Round,
    Result,
    Create,
    Edit,
}

#[derive(Debug)]
enum PendingOperation {
    GetDecks,
    GetDeck(String),
    DeleteDeck(String),
    CreateDeck(Deck),
    None,
}

#[derive(Debug, Clone)]
enum TargetView {
    Details,
    Welcome,
}

#[derive(Debug, Clone)]
enum Message {
    SignUp,
    SendSignUp,
    ToLoginFromSignUp,
    SendLogIn,
    HandleAuthResponse(Result<Token, Error>),
    GetDecks,
    HandleDecksResponse(Result<Vec<Deck>, Error>),
    HandleDeckResponse(Result<Deck, Error>),
    SendCreateDeckRequest,
    HandleCreateDeckResponse(Result<Deck, Error>),
    HandleDeleteDeckResponse(Result<reqwest::StatusCode, Error>),
    UsernameChanged(String),
    PasswordChanged(String),
    SignupUsernameChanged(String),
    SignupPasswordChanged(String),
    SignupBioChanged(String),
    SignupNickChanged(String),
    SelectDeck(usize),
    StartRound,
    CancelRound(TargetView),
    Answer(usize),
    Tick(Instant),
    EditDeck,
    NewDeck,
    AddCard,
    DeleteDeck,
    DeleteCard(usize),
    DeleteAnswer(usize, usize),
    AddAnswer(usize),
    EditDeckTitleChanged(String),
    EditDeckDescriptionChanged(String),
    CardTitleChanged(usize, String),
    AnswerTextChanged((usize, usize), String),
    AnswerIsCorrectChanged((usize, usize), bool),
    FocusNext,
    FocusPrevious,
    KeyboardAnswer(usize),
    None(usize),
}

impl Application for Shisho {
    type Theme = Theme;
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Shisho, Command<Message>) {
        (
            Shisho {
                state: States::Welcome,
                signup: Signup {
                    nick: "".to_owned(),
                    bio: "".to_owned(),
                    username: "".to_owned(),
                    password: "".to_owned(),
                },
                login: Login {
                    username: "".to_owned(),
                    password: "".to_owned(),
                },
                remote_error: None,
                decks: Vec::new(),
                selected_deck: 0,
                selected_card: 0,
                selected_answers: Vec::new(),
                check: false,
                answered: false,
                last_tick: Instant::now(),
                duration: Duration::default(),
                already_selected: false,
                fully_fetched: Vec::new(),
                token: "".to_owned(),
                score: 0.0,
                edit_deck: EditDeck::new(),
                first_login: true,
                pending_operation: PendingOperation::None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Shisho".to_owned()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();
        let time_subscription = match self.answered {
            false => Subscription::none(),
            true => iced::time::every(Duration::from_millis(10)).map(Message::Tick),
        };
        subscriptions.push(time_subscription);
        let event_subscription = iced::subscription::events().map(filter_event);
        subscriptions.push(event_subscription);

        iced::subscription::Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::GetDecks => {
                Command::perform(get_decks(self.token.clone()), Message::HandleDecksResponse)
            }
            Message::HandleDecksResponse(result) => {
                match result {
                    Ok(decks) => {
                        self.fully_fetched = vec![false; decks.len()];
                        self.decks = decks;
                        self.pending_operation = PendingOperation::None;
                    }
                    Err(err) => {
                        print!("{:#?}", err);
                        self.remote_error = Some(err);
                    }
                }
                self.state = States::Loaded;

                Command::none()
            }
            Message::HandleDeckResponse(result) => match result {
                Ok(deck) => {
                    self.fully_fetched[self.selected_deck] = true;
                    self.decks[self.selected_deck].cards = deck.cards;
                    self.pending_operation = PendingOperation::None;
                    self.state = States::Details;

                    Command::none()
                }
                Err(err) => {
                    print!("{:#?}", err);
                    match err {
                        Error::AuthError => {
                            let login = self.login.clone();

                            Command::perform(log_in(login), Message::HandleAuthResponse)
                        }
                        _ => {
                            self.remote_error = Some(err);

                            Command::none()
                        }
                    }
                }
            },
            Message::SendCreateDeckRequest => {
                let deck = deck_from_edit_deck(&self.edit_deck);

                Command::perform(
                    create_deck(self.token.clone(), deck),
                    Message::HandleCreateDeckResponse,
                )
            }
            Message::HandleCreateDeckResponse(result) => match result {
                Ok(deck) => match self.state {
                    States::Create => {
                        self.decks.push(deck);
                        self.fully_fetched.push(true);
                        self.state = States::Loaded;

                        let index = self.decks.len() - 1;
                        self.select_deck(index)
                    }
                    States::Edit => {
                        let id = self.decks[self.selected_deck].id.clone();
                        self.decks[self.selected_deck] = deck;
                        self.state = States::Details;

                        Command::perform(
                            delete_deck(self.token.clone(), id),
                            Message::HandleDeleteDeckResponse,
                        )
                    }
                    _ => Command::none(),
                },
                Err(err) => {
                    print!("{:#?}", err);
                    match err {
                        Error::AuthError => {
                            let deck = deck_from_edit_deck(&self.edit_deck);
                            self.pending_operation = PendingOperation::CreateDeck(deck);
                            let login = self.login.clone();

                            Command::perform(log_in(login), Message::HandleAuthResponse)
                        }
                        _ => {
                            self.remote_error = Some(err);

                            Command::none()
                        }
                    }
                }
            },
            Message::HandleDeleteDeckResponse(result) => match result {
                Ok(_) => {
                    self.pending_operation = PendingOperation::None;
                    Command::none()
                }
                Err(err) => {
                    print!("{:#?}", err);
                    match err {
                        Error::AuthError => {
                            let login = self.login.clone();

                            Command::perform(log_in(login), Message::HandleAuthResponse)
                        }
                        _ => {
                            self.remote_error = Some(err);

                            Command::none()
                        }
                    }
                }
            },
            Message::DeleteDeck => {
                let id = self.decks[self.selected_deck].id.clone();
                self.decks.remove(self.selected_deck);
                self.already_selected = false;
                self.pending_operation = PendingOperation::DeleteDeck(id.clone());
                self.state = States::Loaded;

                Command::perform(
                    delete_deck(self.token.clone(), id),
                    Message::HandleDeleteDeckResponse,
                )
            }
            Message::SendLogIn => {
                let login = self.login.clone();

                Command::perform(log_in(login), Message::HandleAuthResponse)
            }
            Message::SignUp => {
                self.signup.username = self.login.username.clone();
                self.signup.password = self.login.password.clone();
                self.state = States::Signup;

                Command::none()
            }
            Message::HandleAuthResponse(result) => match result {
                Ok(t) => {
                    self.token = t.token.clone();
                    if self.first_login == true {
                        self.first_login = false;
                        self.pending_operation = PendingOperation::GetDecks;
                        Command::perform(get_decks(t.token), Message::HandleDecksResponse)
                    } else {
                        match &self.pending_operation {
                            PendingOperation::GetDecks => {
                                Command::perform(get_decks(t.token), Message::HandleDecksResponse)
                            }
                            PendingOperation::GetDeck(id) => Command::perform(
                                get_deck(self.token.clone(), id.to_owned()),
                                Message::HandleDeckResponse,
                            ),
                            PendingOperation::DeleteDeck(id) => Command::perform(
                                delete_deck(self.token.clone(), id.to_owned()),
                                Message::HandleDeleteDeckResponse,
                            ),
                            PendingOperation::CreateDeck(deck) => Command::perform(
                                create_deck(self.token.clone(), deck.clone()),
                                Message::HandleCreateDeckResponse,
                            ),
                            PendingOperation::None => Command::none(),
                        }
                    }
                }
                Err(err) => {
                    print!("{:#?}", err);
                    self.pending_operation = PendingOperation::None;
                    self.remote_error = Some(err);

                    Command::none()
                }
            },
            Message::UsernameChanged(new_username) => {
                self.login.username = new_username;

                Command::none()
            }
            Message::PasswordChanged(new_password) => {
                self.login.password = new_password;

                Command::none()
            }
            Message::SignupUsernameChanged(new_username) => {
                self.signup.username = new_username;

                Command::none()
            }
            Message::SignupPasswordChanged(new_password) => {
                self.signup.password = new_password;

                Command::none()
            }
            Message::SignupBioChanged(new_bio) => {
                self.signup.bio = new_bio;

                Command::none()
            }
            Message::SignupNickChanged(new_nick) => {
                self.signup.nick = new_nick;

                Command::none()
            }
            Message::EditDeckTitleChanged(new_title) => {
                self.edit_deck.title = new_title;

                Command::none()
            }
            Message::EditDeckDescriptionChanged(new_description) => {
                self.edit_deck.description = new_description;

                Command::none()
            }
            Message::CardTitleChanged(index, new_title) => {
                self.edit_deck.cards[index].title = new_title;

                Command::none()
            }
            Message::AnswerTextChanged((card_index, answer_index), new_text) => {
                self.edit_deck.cards[card_index].possible_answers[answer_index].text = new_text;

                Command::none()
            }
            Message::AnswerIsCorrectChanged((card_index, answer_index), is_correct) => {
                self.edit_deck.cards[card_index].possible_answers[answer_index].is_correct =
                    is_correct;

                Command::none()
            }
            Message::SendSignUp => {
                let signup = self.signup.clone();

                Command::perform(sign_up(signup), Message::HandleAuthResponse)
            }
            Message::SelectDeck(index) => self.select_deck(index),
            Message::ToLoginFromSignUp => {
                if self.login.username == "" {
                    self.login.username = self.signup.username.clone();
                }
                if self.login.password == "" {
                    self.login.password = self.signup.password.clone();
                }

                self.state = States::Welcome;

                Command::none()
            }
            Message::StartRound => {
                if self.decks[self.selected_deck].cards.len() > 0 {
                    self.state = States::Round;
                    self.selected_answers = Vec::new();
                    for card in self.decks[self.selected_deck].cards.iter() {
                        self.selected_answers
                            .push(vec![false; card.possible_answers.len()])
                    }
                }

                Command::none()
            }
            Message::CancelRound(target) => {
                self.selected_card = 0;
                self.check = false;
                self.duration = Duration::ZERO;
                self.last_tick = Instant::now();
                self.answered = false;
                let state = match target {
                    TargetView::Details => States::Details,
                    TargetView::Welcome => States::Loaded,
                };
                self.state = state;

                Command::none()
            }
            Message::Answer(index) => {
                self.answer(index);

                Command::none()
            }
            Message::Tick(now) => {
                match self.state {
                    States::Round => match self.answered {
                        false => {}
                        true => {
                            self.duration += now - self.last_tick;
                            self.last_tick = now;

                            if self.duration.gt(&ANSWER_DELAY) {
                                if self.selected_card
                                    == self.decks[self.selected_deck].cards.len() - 1
                                {
                                    self.state = States::Result;
                                    self.duration = Duration::ZERO;
                                    self.last_tick = Instant::now();
                                    self.selected_card = 0;
                                    self.score = calculate_score(
                                        &self.decks[self.selected_deck].cards,
                                        &self.selected_answers,
                                    )
                                } else {
                                    self.selected_card += 1;
                                    self.check = false;
                                    self.answered = false;
                                }
                            }
                        }
                    },
                    States::Result => {
                        if self.duration.lt(&RESULTS_DELAY) {
                            self.duration += now - self.last_tick;
                            self.last_tick = now;
                            if self.duration.gt(&RESULTS_DELAY) {
                                self.duration = RESULTS_DELAY;
                                self.answered = false;
                            }
                        } else {
                            self.answered = false;
                        }
                    }
                    _ => {}
                }

                Command::none()
            }
            Message::EditDeck => {
                let selected_deck = &self.decks[self.selected_deck];
                self.edit_deck = selected_deck.into();
                self.state = States::Edit;

                Command::none()
            }
            Message::NewDeck => {
                self.edit_deck = EditDeck::new();
                self.state = States::Create;

                Command::none()
            }
            Message::AddCard => {
                self.edit_deck.cards.push(EditCard::new());

                iced::widget::scrollable::snap_to(
                    iced::widget::scrollable::Id::new("edit_view_scroller"),
                    1.0,
                )
            }
            Message::AddAnswer(card_index) => {
                self.edit_deck.cards[card_index]
                    .possible_answers
                    .push(EditAnswer::new());

                Command::none()
            }
            Message::DeleteCard(card_index) => {
                self.edit_deck.cards.remove(card_index);

                Command::none()
            }
            Message::DeleteAnswer(card_index, answer_index) => {
                self.edit_deck.cards[card_index]
                    .possible_answers
                    .remove(answer_index);

                Command::none()
            }
            Message::FocusNext => iced::widget::focus_next(),
            Message::FocusPrevious => iced::widget::focus_previous(),
            Message::KeyboardAnswer(answer) => {
                match self.state {
                    States::Round => self.answer(answer),
                    _ => {}
                }

                Command::none()
            }
            Message::None(_) => Command::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match self.state {
            States::Welcome => self.welcome_view(),
            States::Loaded => self.decks_view().into(),
            States::Details => self.deck_details_view(),
            States::Signup => self.signup_view(),
            States::Round => self.round_view(),
            States::Result => self.results_view(),
            States::Edit => self.edit_deck_view(),
            States::Create => self.edit_deck_view(),
        };
        content
    }
}

fn filter_event(e: iced::Event) -> Message {
    let mut key: Option<iced::keyboard::KeyCode> = None;
    let mut modif: Option<iced::keyboard::Modifiers> = None;

    match e {
        iced::Event::Keyboard(e) => {
            match e {
                iced::keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                } => {
                    key = Some(key_code);
                    modif = Some(modifiers);
                }
                _ => {}
            };
        }
        _ => {}
    };

    match key {
        Some(key_code) => {
            if key_code == KeyCode::Tab {
                match modif {
                    Some(modifiers) => {
                        if modifiers.shift() {
                            return Message::FocusPrevious;
                        }
                        return Message::FocusNext;
                    }
                    None => return Message::None(0),
                }
            }
            return to_answer(key_code);
        }
        None => return Message::None(0),
    }
}

fn to_answer(key: KeyCode) -> Message {
    match key {
        KeyCode::Key1 => Message::KeyboardAnswer(0),
        KeyCode::Key2 => Message::KeyboardAnswer(1),
        KeyCode::Key3 => Message::KeyboardAnswer(2),
        KeyCode::Key4 => Message::KeyboardAnswer(3),
        KeyCode::Key5 => Message::KeyboardAnswer(4),
        KeyCode::Key6 => Message::KeyboardAnswer(5),
        KeyCode::Key7 => Message::KeyboardAnswer(6),
        KeyCode::Key8 => Message::KeyboardAnswer(7),
        KeyCode::Key9 => Message::KeyboardAnswer(8),
        _ => Message::None(0),
    }
}

impl Shisho {
    fn answer(&mut self, answer: usize) {
        if !self.answered && answer < self.selected_answers[self.selected_card].len() {
            self.selected_answers[self.selected_card][answer] = true;
            self.check = true;
            self.duration = Duration::ZERO;
            self.last_tick = Instant::now();
            self.answered = true;
        }
    }

    fn select_deck(&mut self, index: usize) -> iced::Command<Message> {
        self.already_selected = true;
        self.selected_deck = index;
        match self.fully_fetched[index] {
            true => {
                self.state = States::Details;
                Command::none()
            }
            false => {
                self.pending_operation = PendingOperation::GetDeck(self.decks[index].id.clone());
                Command::perform(
                    get_deck(self.token.clone(), self.decks[index].id.clone()),
                    Message::HandleDeckResponse,
                )
            }
        }
    }

    fn welcome_view(&self) -> Element<Message> {
        let shisho_text = row![shisho_text()];

        let mut login_fields = column![]
            .max_width(200)
            .spacing(20)
            .align_items(Alignment::End);

        let username_input = text_input("Username", &self.login.username, Message::UsernameChanged)
            .on_submit(Message::SendLogIn);

        let password_input = iced::widget::TextInput::password(
            text_input("Password", &self.login.password, Message::PasswordChanged)
                .on_submit(Message::SendLogIn),
        );

        let (err, message) = match self.remote_error.clone() {
            Some(err) => (true, err.to_string()),
            None => (false, "".to_owned()),
        };

        if err {
            login_fields =
                login_fields.push(username_input.style(styling::wrong_tex_input_style()));
            login_fields =
                login_fields.push(password_input.style(styling::wrong_tex_input_style()));
            login_fields = login_fields.push(
                text(message).style(iced::theme::Text::Color(iced::Color::from_rgb8(255, 0, 0))),
            );
        } else {
            login_fields = login_fields.push(username_input);
            login_fields = login_fields.push(password_input);
        }

        login_fields = login_fields.push(
            row![
                button(text("Signup")).on_press(Message::SignUp),
                button(text("Login")).on_press(Message::SendLogIn)
            ]
            .spacing(90),
        );

        let content = column![shisho_text, login_fields]
            .spacing(200)
            .align_items(Alignment::Center);
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn signup_view(&self) -> Element<Message> {
        let shisho_text = row![shisho_text()];

        let mut signup_fields = column![]
            .max_width(200)
            .spacing(20)
            .align_items(Alignment::End);

        let username_input = text_input(
            "Username",
            &self.signup.username,
            Message::SignupUsernameChanged,
        )
        .on_submit(Message::SendSignUp);

        let password_input = iced::widget::TextInput::password(
            text_input(
                "Password",
                &self.signup.password,
                Message::SignupPasswordChanged,
            )
            .on_submit(Message::SendSignUp),
        );

        let (err, message) = match self.remote_error.clone() {
            Some(err) => (true, err.to_string()),
            None => (false, "".to_owned()),
        };

        if err {
            signup_fields =
                signup_fields.push(username_input.style(styling::wrong_tex_input_style()));
            signup_fields =
                signup_fields.push(password_input.style(styling::wrong_tex_input_style()));
        } else {
            signup_fields = signup_fields.push(username_input);
            signup_fields = signup_fields.push(password_input);
        }
        signup_fields = signup_fields.push(
            text_input("Nickname", &self.signup.nick, Message::SignupNickChanged)
                .on_submit(Message::SendSignUp),
        );
        signup_fields = signup_fields.push(
            text_input("Bio", &self.signup.bio, Message::SignupBioChanged)
                .on_submit(Message::SendSignUp),
        );
        if err {
            signup_fields = signup_fields.push(
                text(message).style(iced::theme::Text::Color(iced::Color::from_rgb8(255, 0, 0))),
            );
        }
        signup_fields = signup_fields.push(
            row![
                button(text("Back")).on_press(Message::ToLoginFromSignUp),
                button(text("Signup")).on_press(Message::SendSignUp)
            ]
            .spacing(5),
        );
        let content = column![shisho_text, signup_fields].spacing(200);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn decks_view(&self) -> Element<Message> {
        let mut left_column = column![].spacing(15).max_width(200);
        let mut right_column = column![].spacing(15).max_width(200);

        for (index, deck) in self.decks.iter().enumerate() {
            if index % 2 == 0 {
                left_column = left_column.push(
                    button(
                        container(
                            column![text(deck.title.clone()), text(deck.description.clone())]
                                .padding(10)
                                .spacing(15),
                        )
                        .style(styling::card_style())
                        .height(iced::Length::Units(115))
                        .width(iced::Length::Units(200)),
                    )
                    .on_press(Message::SelectDeck(index))
                    .style(styling::invisible_button()),
                );
            } else {
                right_column = right_column.push(
                    button(
                        container(
                            column![text(deck.title.clone()), text(deck.description.clone())]
                                .padding(10)
                                .spacing(15),
                        )
                        .style(styling::card_style())
                        .height(iced::Length::Units(115))
                        .width(iced::Length::Units(200)),
                    )
                    .on_press(Message::SelectDeck(index))
                    .style(styling::invisible_button()),
                );
            }
        }

        let columns_row = row![left_column, right_column].spacing(15);
        let decks_scroll = column![scrollable(column![columns_row])
            .scrollbar_width(5)
            .scroller_width(5),];
        let shisho_text = row![shisho_text()].padding(Padding::from([0, 0, 15, 0]));
        let decks_title = row![
            column![button(refresh_icon()).on_press(Message::GetDecks)]
                .width(iced::Length::Fill)
                .align_items(Alignment::Start),
            column![text("Decks").size(28)]
                .width(iced::Length::Fill)
                .align_items(Alignment::Center),
            column![button("New Deck").on_press(Message::NewDeck)]
                .width(iced::Length::Fill)
                .align_items(Alignment::End),
        ]
        .width(iced::Length::Units(400));

        let content = column![shisho_text, decks_title, decks_scroll]
            .align_items(Alignment::Center)
            .spacing(30);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }

    fn deck_details_view(&self) -> Element<Message> {
        let mut details = (String::from(""), String::from(""));
        if self.already_selected {
            details = get_selected_deck_info(&self.decks[self.selected_deck]);
        }

        let mut deck_details_column = column![].max_width(500);
        let mut deck_details_title_row = row![
            column![
                button(text("Back").size(15)).on_press(Message::CancelRound(TargetView::Welcome))
            ]
            .align_items(Alignment::Start)
            .width(iced::Length::Fill),
            column![text("Deck Details").size(30)]
                .align_items(Alignment::Start)
                .width(iced::Length::Fill),
        ];

        if self.already_selected {
            deck_details_title_row = deck_details_title_row.push(
                column![row![
                    button(text("Start").size(15)).on_press(Message::StartRound),
                    button(text("Edit").size(15)).on_press(Message::EditDeck),
                    button(text("Delete").size(15))
                        .on_press(Message::DeleteDeck)
                        .style(iced::theme::Button::Destructive)
                ]
                .spacing(5),]
                .align_items(Alignment::End)
                .width(iced::Length::Fill),
            );
        }

        deck_details_column = deck_details_column
            .push(deck_details_title_row)
            .push(row![text("Title: ").size(22), text(details.0).size(22)])
            .push(row![
                text("Description: ")
                    .size(22)
                    .horizontal_alignment(Horizontal::Center),
                text(details.1).size(22)
            ])
            .push(row![text("Cards: ")
                .size(22)
                .horizontal_alignment(Horizontal::Center),]);

        if self.already_selected {
            let cards_column = self.decks[self.selected_deck]
                .cards
                .iter()
                .enumerate()
                .fold(
                    column![horizontal_space(Length::Fill)].spacing(10),
                    |cards_column, (index, card)| {
                        let card_column = column![
                            horizontal_space(Length::Fill),
                            text(format!(
                                "{}/{}",
                                index + 1,
                                self.decks[self.selected_deck].cards.len()
                            ))
                            .size(25),
                            row![text(format!("Question: {}", card.title))],
                        ]
                        .padding(Padding::new(5))
                        .spacing(10)
                        .width(iced::Length::Fill);

                        cards_column.push(container(card_column).style(styling::card_style()))
                    },
                );

            deck_details_column = deck_details_column.push(cards_column);
        }

        let body_row = row![deck_details_column]
            .align_items(Alignment::Start)
            .padding(Padding::from([0, 20]))
            .spacing(100);

        let body = container(scrollable(body_row).scrollbar_width(5).scroller_width(5));
        let header = shisho_text();
        let content = column![header, body]
            .spacing(25)
            .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .into()
    }

    fn round_view(&self) -> Element<Message> {
        let title_row = row![
            text("Deck:").size(22),
            text(self.decks[self.selected_deck].title.clone()).size(22)
        ];

        let round_info_row = row![text(format!(
            "{}/{} Cards",
            self.selected_card + 1,
            self.decks[self.selected_deck].cards.len()
        ))
        .size(35)];

        let answers_column = self.decks[self.selected_deck].cards[self.selected_card]
            .possible_answers
            .iter()
            .enumerate()
            .fold(
                column![].width(iced::Length::Units(400)),
                |answers_column, (index, answer)| {
                    let mut is_selected = None;
                    if self.selected_answers[self.selected_card][index] {
                        is_selected = Some(index);
                    }

                    let mut row = row![radio(
                        format!("{}) {}", index + 1, answer.text),
                        index,
                        is_selected,
                        Message::Answer,
                    )]
                    .spacing(10);

                    if self.check {
                        if answer.is_correct && is_selected != None {
                            row = row.push(text("Correct!").size(28));
                        } else if !answer.is_correct && is_selected != None {
                            row = row.push(text("Wrong").size(28));
                        }
                    }
                    answers_column.push(row)
                },
            );

        let card = column![
            row![
                text(format!(
                    "Question: {}",
                    self.decks[self.selected_deck].cards[self.selected_card].title
                )),
                horizontal_space(iced::Length::Units(100))
            ],
            row![text("Possible answers:")],
            answers_column
        ]
        .padding(Padding::new(5))
        .spacing(10)
        .max_width(400);

        let card_container = container(card).style(styling::card_style());

        let back_to_decks_button =
            button(text("Back to deck")).on_press(Message::CancelRound(TargetView::Details));

        let mut content = column![
            title_row,
            back_to_decks_button,
            round_info_row,
            card_container
        ]
        .align_items(Alignment::Center)
        .spacing(15);

        if self.check {
            let progress = self.duration.as_secs_f32() / ANSWER_DELAY.as_secs_f32();
            let progress_bar = progress_bar(0.0..=1.0, progress)
                .height(Length::Units(5))
                .width(Length::Units(600));
            content = content.push(progress_bar);
        }

        container(content)
            .width(Length::Fill)
            .center_x()
            .align_x(iced::alignment::Horizontal::Center)
            .into()
    }

    fn results_view(&self) -> Element<Message> {
        let title_row = row![
            text("Deck:").size(22),
            text(self.decks[self.selected_deck].title.clone()).size(22)
        ];

        let score = self.score * self.duration.as_secs_f32() / RESULTS_DELAY.as_secs_f32();
        let round_info_row = row![
            text("Total Score: ").size(35),
            text(format!("{:.2}%", score))
                .size(35)
                .style(iced::theme::Text::Color(styling::score_text_color(score)))
        ];

        let cards_column = self.decks[self.selected_deck]
            .cards
            .iter()
            .enumerate()
            .fold(
                column![]
                    .padding(Padding::from([0, 12, 0, 0]))
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .max_width(400),
                |cards_column, (card_index, card)| {
                    let correct_card =
                        card_is_correct(&card.possible_answers, &self.selected_answers[card_index]);
                    let answers_column = card.possible_answers.iter().enumerate().fold(
                        column![].width(iced::Length::Units(400)),
                        |answers_column, (answer_index, answer)| {
                            let mut is_selected = None;
                            if self.selected_answers[card_index][answer_index] {
                                is_selected = Some(answer_index);
                            }

                            let mut row = row![radio(
                                format!("{}) {}", answer_index + 1, answer.text),
                                answer_index,
                                is_selected,
                                Message::None,
                            )]
                            .spacing(10);

                            if self.check {
                                if answer.is_correct {
                                    row = row.push(right_icon().vertical_alignment(Vertical::Top));
                                } else if !answer.is_correct
                                    && !correct_card
                                    && self.selected_answers[card_index][answer_index]
                                {
                                    row = row.push(wrong_icon().vertical_alignment(Vertical::Top));
                                }
                            }
                            answers_column.push(row)
                        },
                    );

                    let card_widget = column![
                        row![text(format!(
                            "Question: {}",
                            self.decks[self.selected_deck].cards[card_index].title
                        ))],
                        row![text("Possible answers:")],
                        answers_column
                    ]
                    .padding(Padding::new(5))
                    .spacing(10);

                    let mut card_container = container(card_widget);

                    if correct_card {
                        card_container = card_container.style(styling::correct_card_style());
                    } else {
                        card_container = card_container.style(styling::wrong_card_style());
                    }
                    cards_column.push(card_container)
                },
            );

        let cards_scroll = scrollable(cards_column)
            .scrollbar_width(5)
            .scroller_width(5);

        let back_to_decks_button =
            button("Back to deck").on_press(Message::CancelRound(TargetView::Details));

        let content = column![
            title_row,
            back_to_decks_button,
            round_info_row,
            cards_scroll
        ]
        .align_items(Alignment::Center)
        .spacing(15);

        container(content)
            .width(Length::Fill)
            .center_x()
            .align_x(iced::alignment::Horizontal::Center)
            .into()
    }

    fn edit_deck_view(&self) -> Element<Message> {
        let title_row = row![
            text("Title: ").size(22),
            text_input(
                "Deck Title",
                &self.edit_deck.title,
                Message::EditDeckTitleChanged,
            )
        ];

        let description_row = row![
            text("Description: ").size(22),
            text_input(
                "Deck description",
                &self.edit_deck.description,
                Message::EditDeckDescriptionChanged,
            )
        ];

        let cards = self.edit_deck.cards.iter().enumerate().fold(
            column![].padding(Padding::from([0, 12, 0, 12])).spacing(10),
            |cards, (card_index, card)| {
                let answers = card.possible_answers.iter().enumerate().fold(
                    column![],
                    |answers, (answer_index, answer)| {
                        let text_row = row![
                            text("Text: "),
                            text_input("Answer's text", &answer.text, move |text| {
                                Message::AnswerTextChanged((card_index, answer_index), text)
                            }),
                            button(text("Remove"))
                                .on_press(Message::DeleteAnswer(card_index, answer_index))
                                .style(iced::theme::Button::Destructive)
                        ]
                        .spacing(10);

                        let is_correct_row = row![
                            text("Is correct?:"),
                            radio("Yes", true, Some(answer.is_correct), |is_correct| {
                                Message::AnswerIsCorrectChanged(
                                    (card_index, answer_index),
                                    is_correct,
                                )
                            }),
                            radio("No", false, Some(answer.is_correct), |is_correct| {
                                Message::AnswerIsCorrectChanged(
                                    (card_index, answer_index),
                                    is_correct,
                                )
                            })
                        ]
                        .spacing(10)
                        .padding(Padding::from([0, 0, 10, 0]));

                        answers.push(text_row).push(is_correct_row).into()
                    },
                );
                let card_widget = column![
                    row![
                        text("Question: "),
                        text_input("Card's question", &card.title, move |title| {
                            Message::CardTitleChanged(card_index, title)
                        },)
                    ]
                    .spacing(10),
                    row![text("Answers:")],
                    answers,
                    row![
                            column![].width(iced::Length::Fill),
                            column![
                                button(text("Add answer")).on_press(Message::AddAnswer(card_index))
                            ]
                            .width(iced::Length::Fill)
                            .align_items(Alignment::Center),
                            column![button(text("Remove Card"))
                                .on_press(Message::DeleteCard(card_index))
                                .style(iced::theme::Button::Destructive)]
                            .width(iced::Length::Fill)
                            .align_items(Alignment::End)
                        ]
                ]
                .padding(Padding::from([7, 7]))
                .spacing(10);

                cards.push(container(card_widget).style(styling::card_style()))
            },
        );

        let add_card_button = column![button(text("Add new card")).on_press(Message::AddCard)]
            .padding(Padding::from([0, 0, 20, 0]));

        let cards_scroll = scrollable(
            column![cards, add_card_button]
                .spacing(10)
                .align_items(Alignment::Center),
        )
        .scrollbar_width(5)
        .scroller_width(5)
        .id(iced::widget::scrollable::Id::new("edit_view_scroller"));

        let mut button_row = row![].spacing(5);
        match self.state {
            States::Create => {
                button_row = button_row
                    .push(
                        button(text("Back to deck"))
                            .on_press(Message::CancelRound(TargetView::Welcome)),
                    )
                    .push(button(text("Create deck")).on_press(Message::SendCreateDeckRequest))
            }
            States::Edit => {
                button_row = button_row
                    .push(
                        button(text("Back to deck"))
                            .on_press(Message::CancelRound(TargetView::Details)),
                    )
                    .push(button(text("Save changes")).on_press(Message::SendCreateDeckRequest))
            }
            _ => {}
        }

        let content = column![title_row, description_row, button_row, cards_scroll]
            .align_items(Alignment::Center)
            .spacing(15);

        container(content)
            .width(Length::Fill)
            .center_x()
            .align_x(iced::alignment::Horizontal::Center)
            .into()
    }
}

fn shisho_text() -> iced::widget::Text<'static> {
    text("Shisho").size(86)
}

const ICONS: iced::Font = iced::Font::External {
    name: "Icons",
    bytes: include_bytes!("../../shisho/fonts/icons.ttf"),
};

fn icon(unicode: char) -> iced::widget::Text<'static> {
    text(unicode.to_string()).font(ICONS).size(35)
}

fn right_icon() -> iced::widget::Text<'static> {
    icon('\u{E876}').style(iced::theme::Text::Color(iced::Color::from_rgb8(0, 255, 0)))
}

fn wrong_icon() -> iced::widget::Text<'static> {
    icon('\u{E5CD}').style(iced::theme::Text::Color(iced::Color::from_rgb8(255, 0, 0)))
}

fn refresh_icon() -> iced::widget::Text<'static> {
    icon('\u{E5D5}').size(20)
}

fn calculate_score(cards: &Vec<Card>, chosen: &Vec<Vec<bool>>) -> f32 {
    let mut correct_cards = 0;
    for (index, card) in cards.iter().enumerate() {
        if card_is_correct(&card.possible_answers, &chosen[index]) {
            correct_cards += 1;
        }
    }
    ((correct_cards) as f32 / (cards.len()) as f32) * (100) as f32
}

fn card_is_correct(answers: &Vec<Answer>, chosen: &Vec<bool>) -> bool {
    for (index, answer) in answers.iter().enumerate() {
        if answer.is_correct != chosen[index] {
            return false;
        }
    }
    true
}

fn get_selected_deck_info(deck: &Deck) -> (String, String) {
    (deck.title.clone(), deck.description.clone())
}

struct EditDeck {
    id: String,
    title: String,
    description: String,
    cards: Vec<EditCard>,
}

impl EditDeck {
    fn new() -> Self {
        EditDeck {
            id: "".to_owned(),
            title: "".to_owned(),
            description: "".to_owned(),
            cards: Vec::new(),
        }
    }
}

impl From<&client::Deck> for EditDeck {
    fn from(deck: &client::Deck) -> Self {
        EditDeck {
            id: deck.id.clone(),
            title: deck.title.clone(),
            description: deck.description.clone(),
            cards: deck.cards.iter().map(|card| card.into()).collect(),
        }
    }
}

fn deck_from_edit_deck(edit_deck: &EditDeck) -> client::Deck {
    client::Deck {
        id: if edit_deck.id == "" {
            "".to_owned()
        } else {
            edit_deck.id.clone()
        },
        title: edit_deck.title.clone(),
        description: edit_deck.description.clone(),
        cards: edit_deck
            .cards
            .iter()
            .map(|edit_card| card_from_edit_card(edit_card))
            .collect(),
    }
}

struct EditCard {
    title: String,
    possible_answers: Vec<EditAnswer>,
}

impl EditCard {
    fn new() -> Self {
        EditCard {
            title: "".to_owned(),
            possible_answers: Vec::new(),
        }
    }
}

impl From<&client::Card> for EditCard {
    fn from(card: &client::Card) -> Self {
        EditCard {
            title: card.title.clone(),
            possible_answers: card
                .possible_answers
                .iter()
                .map(|answer| answer.into())
                .collect(),
        }
    }
}

fn card_from_edit_card(edit_card: &EditCard) -> client::Card {
    client::Card {
        title: edit_card.title.clone(),
        possible_answers: edit_card
            .possible_answers
            .iter()
            .map(|edit_answer| answer_from_edit_answer(edit_answer))
            .collect(),
    }
}

struct EditAnswer {
    text: String,
    is_correct: bool,
}

impl EditAnswer {
    fn new() -> Self {
        EditAnswer {
            text: "".to_owned(),
            is_correct: false,
        }
    }
}

impl From<&client::Answer> for EditAnswer {
    fn from(answer: &client::Answer) -> Self {
        EditAnswer {
            text: answer.text.clone(),
            is_correct: answer.is_correct,
        }
    }
}

fn answer_from_edit_answer(edit_answer: &EditAnswer) -> client::Answer {
    client::Answer {
        text: edit_answer.text.clone(),
        is_correct: edit_answer.is_correct,
    }
}
