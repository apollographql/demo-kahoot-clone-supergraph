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
        Some(Player {
            id: player_id.clone(),
            ..Default::default()
        })
    }

    pub(crate) async fn create_player(&self, username: String, quiz_id: &ID) -> Option<Player> {
        let player_id = Uuid::new_v4();
        Some(Player {
            id: ID::from(player_id.to_string()),
            name: username,
            quiz_id: quiz_id.clone(),
        })
    }

    pub(crate) async fn players_for_quiz(&self, quiz_id: &ID) -> Vec<Player> {
        vec![Player::default()]
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
        stream::iter(vec![vec![Player::default()], vec![Player::default()]])
    }

    pub(crate) async fn new_players(&self, quiz_id: &ID, players: Vec<Player>) {
        todo!()
    }
}

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn player<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        player_id: ID,
    ) -> async_graphql::Result<Player> {
        let in_memory_db: &InMemoryDb = ctx.data_unchecked();
        Ok(in_memory_db.get_player(&player_id).await.unwrap())
    }

    async fn players_for_a_quiz<'ctx>(&self, ctx: &Context<'ctx>, quiz_id: ID) -> Vec<Player> {
        let in_memory_db: &InMemoryDb = ctx.data_unchecked();

        in_memory_db.players_for_quiz(&quiz_id).await
    }

    #[graphql(entity)]
    async fn find_player_by_id_and_quiz_id<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        id: ID,
        quiz_id: ID,
    ) -> Option<Player> {
        let in_memory_db: &InMemoryDb = ctx.data_unchecked();
        let player = in_memory_db.get_player(&id).await;
        player
    }
}

pub(crate) struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn players_for_a_quiz<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        quiz_id: ID,
    ) -> async_graphql::Result<impl Stream<Item = Vec<Player>>> {
        let in_memory_broker: &InMemoryBroker = ctx.data_unchecked();
        let player_stream = in_memory_broker.subscribe_new_players(&quiz_id).await;
        Ok(player_stream)
    }
}

pub(crate) struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_player<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        user_name: String,
        quiz_id: ID,
    ) -> async_graphql::Result<Player> {
        let in_memory_db: &InMemoryDb = ctx.data_unchecked();

        let new_player = in_memory_db
            .create_player(user_name, &quiz_id)
            .await
            .ok_or_else(|| async_graphql::Error::new("cannot create a player"))?;

        Ok(new_player)
    }
}

#[derive(Clone, SimpleObject, Debug, Default)]
pub(crate) struct Player {
    pub(crate) id: ID,
    pub(crate) name: String,
    pub(crate) quiz_id: ID,
}
