use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

const SIGNUP_URL: &str = "http://localhost:8080/signup";
const LOGIN_URL: &str = "http://localhost:8080/login";
const GET_DECKS_URL: &str = "http://localhost:8080/decks";
const GET_DECK_URL: &str = "http://localhost:8080/decks/";
const CREATE_DECK_URL: &str = "http://localhost:8080/decks/create";
const DELETE_DECK_URL: &str = "http://localhost:8080/decks/delete/";

#[derive(Debug, Clone)]
pub enum Error {
    PayloadError,
    APIError,
    AuthError,
    NotFound,
    NetworkError,
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        dbg!(error);

        Error::NetworkError
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PayloadError => write!(f, "Malformed json"),
            Error::APIError => write!(f, "Server error"),
            Error::AuthError => write!(f, "Wrong login"),
            Error::NotFound => write!(f, "Not found"),
            Error::NetworkError => write!(f, "Network error"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Token {
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deck {
    #[serde(default = "empty_cards")]
    pub cards: Vec<Card>,
    pub description: String,
    pub title: String,
    pub id: String,
}

fn empty_cards() -> Vec<Card> {
    Vec::new()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Card {
    pub title: String,
    pub possible_answers: Vec<Answer>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Answer {
    pub text: String,
    #[serde(default = "false_answer")]
    pub is_correct: bool,
}

fn false_answer() -> bool {
    false
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signup {
    pub nick: String,
    pub bio: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

pub async fn sign_up(signup: Signup) -> Result<Token, Error> {
    let client = reqwest::Client::new();
    let response = client.post(SIGNUP_URL).json(&signup).send().await;
    println!("{:#?}", response);

    match response {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => {
                let token = res.json::<Token>().await;
                match token {
                    Ok(t) => Ok(t),
                    Err(_) => Err(Error::PayloadError),
                }
            }
            _ => Err(res.status().into()),
        },
        Err(err) => Err(Error::from(err)),
    }
}

pub async fn log_in(login: Login) -> Result<Token, Error> {
    let client = reqwest::Client::new();
    let response = client.post(LOGIN_URL).json(&login).send().await;
    println!("{:#?}", response);

    match response {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => {
                let token = res.json::<Token>().await;
                match token {
                    Ok(t) => Ok(t),
                    Err(_) => Err(Error::PayloadError),
                }
            }
            _ => Err(res.status().into()),
        },
        Err(err) => Err(Error::from(err)),
    }
}

impl From<reqwest::StatusCode> for Error {
    fn from(code: reqwest::StatusCode) -> Self {
        match code {
            reqwest::StatusCode::UNAUTHORIZED => Error::AuthError,
            reqwest::StatusCode::NOT_FOUND => Error::NotFound,
            _ => Error::APIError,
        }
    }
}

pub async fn get_decks(token: String) -> Result<Vec<Deck>, Error> {
    let deck_response = reqwest::Client::new()
        .get(GET_DECKS_URL)
        .bearer_auth(token)
        .send()
        .await;

    match deck_response {
        Ok(deck_res) => match deck_res.status() {
            reqwest::StatusCode::OK => {
                let deck_json = deck_res.json::<Vec<Deck>>().await;
                match deck_json {
                    Ok(decks) => {
                        println!("{:#?}", decks);
                        Ok(decks)
                    }
                    Err(err) => {
                        println!("{:#?}", err);
                        Err(Error::PayloadError)
                    }
                }
            }
            _ => Err(deck_res.status().into()),
        },
        Err(err) => {
            println!("{:#?}", err);
            Err(Error::from(err))
        }
    }
}

pub async fn get_deck(token: String, id: String) -> Result<Deck, Error> {
    let deck_response = reqwest::Client::new()
        .get(GET_DECK_URL.to_owned() + &id)
        .bearer_auth(token)
        .send()
        .await;

    match deck_response {
        Ok(deck_res) => match deck_res.status() {
            reqwest::StatusCode::OK => {
                let deck_json = deck_res.json::<Deck>().await;
                match deck_json {
                    Ok(deck) => {
                        println!("{:#?}", deck);
                        Ok(deck)
                    }
                    Err(err) => {
                        println!("{:#?}", err);
                        Err(Error::PayloadError)
                    }
                }
            }
            _ => Err(deck_res.status().into()),
        },
        Err(err) => {
            println!("{:#?}", err);
            Err(Error::from(err))
        }
    }
}

pub async fn delete_deck(token: String, deck_id: String) -> Result<reqwest::StatusCode, Error> {
    let deck_response = reqwest::Client::new()
        .post(DELETE_DECK_URL.to_owned() + &deck_id)
        .bearer_auth(token)
        .send()
        .await;

    match deck_response {
        Ok(deck_res) => match deck_res.status() {
            reqwest::StatusCode::OK => Ok(reqwest::StatusCode::OK),
            _ => Err(deck_res.status().into()),
        },
        Err(err) => {
            println!("{:#?}", err);
            Err(Error::from(err))
        }
    }
}

pub async fn create_deck(token: String, deck: Deck) -> Result<Deck, Error> {
    let deck_response = reqwest::Client::new()
        .post(CREATE_DECK_URL)
        .bearer_auth(token)
        .json(&deck)
        .send()
        .await;

    match deck_response {
        Ok(deck_res) => match deck_res.status() {
            reqwest::StatusCode::OK => {
                let deck_json = deck_res.json::<Deck>().await;
                match deck_json {
                    Ok(deck) => {
                        println!("{:#?}", deck);
                        Ok(deck)
                    }
                    Err(err) => {
                        println!("{:#?}", err);
                        Err(Error::PayloadError)
                    }
                }
            }
            _ => Err(deck_res.status().into()),
        },
        Err(err) => {
            println!("{:#?}", err);
            Err(Error::from(err))
        }
    }
}
