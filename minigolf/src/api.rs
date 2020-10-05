//
use serde::{Deserialize, Serialize};

use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Player {
    pub uuid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinGame {
    pub id: Option<usize>,
    pub player: Player,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Putt {
    pub id: usize,
    pub player: Player,
    pub time: Duration,
    pub impulse: [f64; 2],
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetUpdate {
    pub id: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JoinResponse {
    pub id: usize,
    pub uid: usize,
    pub game: PublicStates,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PublicPut {
    pub id: usize,
    pub player: PublicPlayer,
    pub time: Duration,
    pub impulse: [f64; 2],
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PublicStates {
    WaitingForPlayers(PublicWaitingForPlayers),
    Warmup(PublicWarmup),
    Play(PublicPlay),
    GameOver(PublicGameOver),
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PublicPlayer {
    pub id: usize,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PublicWaitingForPlayers {
    players: Vec<PublicPlayer>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PublicWarmup {
    time: Duration,
    players: Vec<PublicPlayer>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PublicPlay {
    pub time: Duration,
    pub players: Vec<PublicPlayer>,
    pub puts: Vec<PublicPut>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PublicGameOver {
    pub time: Duration,
    pub reserved: Vec<PublicPlayer>,
    pub discard: Vec<PublicPlayer>,
}
