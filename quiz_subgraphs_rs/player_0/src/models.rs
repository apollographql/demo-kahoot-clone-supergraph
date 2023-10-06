use std::collections::HashMap;

use async_graphql::{Context, Object, SimpleObject, Subscription, ID};
use futures_util::{stream, Stream, StreamExt};
use tokio::sync::{
    broadcast::{self},
    RwLock,
};
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

#[derive(Default)]
pub(crate) struct InMemoryDb {
    players: RwLock<HashMap<ID, Player>>,
}

impl InMemoryDb {
    pub(crate) async fn get_player(&self, player_id: &ID) -> Option<Player> {
        todo!()
    }

    pub(crate) async fn create_player(&self, username: String, quiz_id: &ID) -> Option<Player> {
        todo!()
    }

    pub(crate) async fn players_for_quiz(&self, quiz_id: &ID) -> Vec<Player> {
        todo!()
    }
}

#[derive(Default)]
pub(crate) struct InMemoryBroker {
    players: RwLock<HashMap<ID, broadcast::Sender<Vec<Player>>>>,
}

impl InMemoryBroker {
    pub(crate) async fn subscribe_new_players(
        &self,
        quiz_id: &ID,
    ) -> impl Stream<Item = Vec<Player>> {
        stream::empty()
    }

    pub(crate) async fn new_players(&self, quiz_id: &ID, players: Vec<Player>) {
        todo!()
    }
}

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn test<'ctx>(&self, ctx: &Context<'ctx>, id: ID) -> String {
        String::from("test")
    }
}

pub(crate) struct SubscriptionRoot;

// #[Subscription]
// impl SubscriptionRoot {
//     async fn example<'ctx>(
//         &self,
//         ctx: &Context<'ctx>,
//         quiz_id: ID,
//     ) -> async_graphql::Result<impl Stream<Item = String>> {
//         stream::empty()
//     }
// }

pub(crate) struct MutationRoot;

// #[Object]
// impl MutationRoot {
// TODO
// }

#[derive(Clone, SimpleObject, Debug)]
pub(crate) struct Player {
    pub(crate) test: usize,
}
