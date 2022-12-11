use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

use self::{
    get_popular_decks::GetPopularDecksPopularDecks,
    new_deck::{
        CreateAnswerInput, CreateCardInput, CreateDeckInput, NewDeckCreateDeck,
        NewDeckCreateDeckDeck,
    },
    obtain_deck::ObtainDeckDeck,
};

const GRAPHQL_URL: &str = "http://localhost:8080/query";
const SIGNUP_URL: &str = "http://localhost:8080/signup";
const LOGIN_URL: &str = "http://localhost:8080/login";

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
    let client: reqwest::Client;
    if let Ok(c) = reqwest::Client::builder()
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            ))
            .collect(),
        )
        .build()
    {
        client = c;
    } else {
        return Err(Error::NetworkError);
    }

    let response = post_graphql::<GetPopularDecks, _>(
        &client,
        GRAPHQL_URL,
        get_popular_decks::Variables { first: Some(50) },
    )
    .await;

    match response {
        Ok(response) => {
            if let Some(response_body) = response.data {
                if let Some(decks_ql) = response_body.popular_decks {
                    return Ok(decks_ql.into());
                }

                return Err(Error::PayloadError);
            }

            return Err(Error::PayloadError);
        }

        Err(err) => Err(Error::from(err)),
    }
}

pub async fn get_deck(token: String, id: String) -> Result<Deck, Error> {
    let client: reqwest::Client;
    if let Ok(c) = reqwest::Client::builder()
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            ))
            .collect(),
        )
        .build()
    {
        client = c;
    } else {
        return Err(Error::NetworkError);
    }

    let response = post_graphql::<ObtainDeck, _>(
        &client,
        GRAPHQL_URL,
        obtain_deck::Variables { id: id.clone() },
    )
    .await;

    match response {
        Ok(response) => {
            if let Some(response_body) = response.data {
                if let Some(d_ql) = response_body.deck {
                    match d_ql.try_into() {
                        Ok(deck) => return Ok(deck),
                        Err(err) => return Err(err),
                    }
                }
                return Err(Error::PayloadError);
            }
            return Err(Error::PayloadError);
        }
        Err(err) => return Err(Error::from(err)),
    }
}

pub async fn delete_deck(token: String, deck_id: String) -> Result<reqwest::StatusCode, Error> {
    let client: reqwest::Client;
    if let Ok(c) = reqwest::Client::builder()
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            ))
            .collect(),
        )
        .build()
    {
        client = c;
    } else {
        return Err(Error::NetworkError);
    }

    let response = post_graphql::<RemoveDeck, _>(
        &client,
        GRAPHQL_URL,
        remove_deck::Variables {
            id: deck_id.clone(),
        },
    )
    .await;

    match response {
        Ok(response) => {
            if let Some(response_body) = response.data {
                if let Some(del_deck) = response_body.delete_deck {
                    if let Some(_success) = del_deck.success {
                        return Ok(reqwest::StatusCode::OK);
                    }
                    return Err(Error::PayloadError);
                }
                return Err(Error::PayloadError);
            }
            return Err(Error::PayloadError);
        }
        Err(err) => {
            println!("{:#?}", err);
            return Err(Error::from(err));
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "new_deck.graphql",
    response_derives = "Debug"
)]
struct NewDeck;

impl From<Deck> for CreateDeckInput {
    fn from(deck: Deck) -> Self {
        let mut deck_ql = CreateDeckInput {
            title: deck.title.clone(),
            description: deck.description.clone(),
            is_public: false,
            cards: vec![],
        };

        deck_ql.cards = deck
            .cards
            .iter()
            .map(|card| {
                let mut card_ql = CreateCardInput {
                    title: card.title.clone(),
                    answers: vec![],
                };

                card_ql.answers = card
                    .possible_answers
                    .iter()
                    .map(|answer| CreateAnswerInput {
                        text: answer.text.clone(),
                        is_correct: answer.is_correct,
                    })
                    .collect();

                card_ql
            })
            .collect();
        deck_ql
    }
}

pub async fn create_deck(token: String, deck: Deck) -> Result<Deck, Error> {
    let client: reqwest::Client;
    if let Ok(c) = reqwest::Client::builder()
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            ))
            .collect(),
        )
        .build()
    {
        client = c;
    } else {
        return Err(Error::NetworkError);
    }

    let response = post_graphql::<NewDeck, _>(
        &client,
        GRAPHQL_URL,
        new_deck::Variables { input: deck.into() },
    )
    .await;

    match response {
        Ok(response) => {
            if let Some(response_body) = response.data {
                if let Some(d_ql) = response_body.create_deck {
                    match d_ql.try_into() {
                        Ok(deck) => return Ok(deck),
                        Err(err) => return Err(err),
                    }
                }
                return Err(Error::PayloadError);
            }
            return Err(Error::PayloadError);
        }
        Err(err) => return Err(Error::from(err)),
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "obtain_deck.graphql",
    response_derives = "Debug"
)]
struct ObtainDeck;

impl TryFrom<ObtainDeckDeck> for Deck {
    type Error = crate::Error;

    fn try_from(d_ql: ObtainDeckDeck) -> Result<Self, Self::Error> {
        let mut deck = Deck {
            cards: vec![],
            description: d_ql.description.clone(),
            title: d_ql.title.clone(),
            id: d_ql.id.clone(),
        };
        let c_ql = d_ql.cards.unwrap_or(vec![]);
        for c in c_ql.iter() {
            let mut c_answers: Vec<Answer> = vec![];
            let c_card = c.as_ref().unwrap();

            let ans_placeholder = vec![];
            let a_ql = c_card.answers.as_ref().unwrap_or(&ans_placeholder);

            if a_ql.len() == 0 {
                return Err(Error::PayloadError);
            }

            for a in a_ql.iter() {
                let a_answer = a.as_ref().unwrap();
                c_answers.push(Answer {
                    text: a_answer.text.clone(),
                    is_correct: a_answer.is_correct,
                });
            }
            deck.cards.push(Card {
                title: c_card.title.clone(),
                possible_answers: c_answers,
            });
        }
        Ok(deck)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "popular_decks.graphql",
    response_derives = "Debug"
)]
struct GetPopularDecks;

impl From<GetPopularDecksPopularDecks> for Vec<Deck> {
    fn from(decks_ql: GetPopularDecksPopularDecks) -> Self {
        let mut decks: Vec<Deck> = vec![];
        let edges = decks_ql.edges.unwrap_or(vec![]);
        for edge in edges.iter() {
            let d_ql = edge.node.as_ref().unwrap();
            decks.push(Deck {
                cards: vec![],
                description: d_ql.description.clone(),
                title: d_ql.title.clone(),
                id: d_ql.id.clone(),
            })
        }
        decks
    }
}

impl TryFrom<NewDeckCreateDeck> for Deck {
    type Error = crate::Error;

    fn try_from(d: NewDeckCreateDeck) -> Result<Self, Self::Error> {
        let d_ql: NewDeckCreateDeckDeck;
        if let Some(deck) = d.deck {
            d_ql = deck;
        } else {
            return Err(Error::PayloadError);
        }
        let mut deck = Deck {
            cards: vec![],
            description: d_ql.description.clone(),
            title: d_ql.title.clone(),
            id: d_ql.id.clone(),
        };
        let c_ql = d_ql.cards.unwrap_or(vec![]);
        for c in c_ql.iter() {
            let mut c_answers: Vec<Answer> = vec![];
            let c_card = c.as_ref().unwrap();

            let ans_placeholder = vec![];
            let a_ql = c_card.answers.as_ref().unwrap_or(&ans_placeholder);

            if a_ql.len() == 0 {
                return Err(Error::PayloadError);
            }

            for a in a_ql.iter() {
                let a_answer = a.as_ref().unwrap();
                c_answers.push(Answer {
                    text: a_answer.text.clone(),
                    is_correct: a_answer.is_correct,
                });
            }
            deck.cards.push(Card {
                title: c_card.title.clone(),
                possible_answers: c_answers,
            });
        }
        Ok(deck)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "remove_deck.graphql",
    response_derives = "Debug"
)]
struct RemoveDeck;
