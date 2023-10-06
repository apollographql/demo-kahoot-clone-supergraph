use std::collections::HashMap;

use async_graphql::{Context, Object, SimpleObject, Subscription, ID};
use futures_util::{Stream, StreamExt, stream};
use tokio::sync::{
    broadcast::{self},
    RwLock,
};
use tokio_stream::wrappers::BroadcastStream;

pub(crate) struct InMemoryDb {
    quizzes: RwLock<HashMap<ID, Quiz>>,
    // leaderboard by quiz id -> points by player_id
    leaderboard: RwLock<HashMap<ID, HashMap<ID, usize>>>,
}

impl InMemoryDb {
    pub(crate) async fn next_question(&self, quiz_id: &ID) -> Option<Question> {
        todo!()
    }

    pub(crate) async fn get_player_points(&self, player_id: &ID, quiz_id: &ID) -> Option<usize> {
        todo!()
    }

    pub(crate) async fn compute_leaderboard(&self, quiz_id: &ID) -> Option<Leaderboard> {
        todo!()
    }

    pub(crate) async fn answer(
        &self,
        player_id: &ID,
        quiz_id: &ID,
        _question_id: &ID,
        choice_id: Option<&ID>,
    ) -> Option<(Response, Leaderboard)> {
        todo!()
    }

    pub(crate) async fn get_quiz(&self, quiz_id: &ID) -> Option<Quiz> {
        todo!()
    }

    pub(crate) async fn get_quizzes(&self) -> Vec<Quiz> {
        todo!()
    }
}

#[derive(Default)]
pub(crate) struct InMemoryBroker {
    quizzes: RwLock<HashMap<ID, broadcast::Sender<Question>>>,
    leaderboard: RwLock<HashMap<ID, broadcast::Sender<Leaderboard>>>,
}

impl InMemoryBroker {
    pub(crate) async fn subscribe_quiz(&self, quiz_id: &ID) -> impl Stream<Item = Question> {
        stream::empty()
    }

    pub(crate) async fn next_question(&self, quiz_id: &ID, question: Question) {
        todo!()
    }

    pub(crate) async fn unsubscribe_quiz(&self, quiz_id: &ID) {
        todo!()
    }

    pub(crate) async fn unsubscribe_leaderboard(&self, quiz_id: &ID) {
        todo!()
    }

    pub(crate) async fn subscribe_leaderboard(
        &self,
        quiz_id: &ID,
    ) -> impl Stream<Item = Leaderboard> {
        stream::empty()
    }

    pub(crate) async fn broadcast_leaderboard(&self, quiz_id: &ID, leaderboard: Leaderboard) {
        todo!()
    }
}

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    // TODO
    async fn test<'ctx>(&self, ctx: &Context<'ctx>, id: ID) -> String {
        String::from("test")
    }
}

pub(crate) struct SubscriptionRoot;

// #[Subscription]
// impl SubscriptionRoot {
//     // TODO
// async fn example<'ctx>(
//     &self,
//     ctx: &Context<'ctx>,
//     quiz_id: ID,
// ) -> async_graphql::Result<impl Stream<Item = String>> {
//     stream::empty()
// }
// }

pub(crate) struct MutationRoot;

// #[Object]
// impl MutationRoot {
//     // TODO
// }

#[derive(Clone, SimpleObject, Debug)]
pub(crate) struct Player {
    test: usize,
}

#[derive(Clone, Default, SimpleObject, Debug)]
pub(crate) struct Quiz {
    test: usize,
}

#[derive(SimpleObject, Clone, Debug)]
pub(crate) struct Question {
    test: usize,
}

#[derive(Clone, Default, SimpleObject, Debug)]
pub(crate) struct Choice {
    test: usize,
}

#[derive(Clone, Default, SimpleObject, Debug)]
pub(crate) struct Leaderboard {
    test: usize,
}

#[derive(Clone, Default, SimpleObject, Debug)]
pub(crate) struct Response {
    test: usize,
}

impl Default for InMemoryDb {
    fn default() -> Self {
        Self {
            quizzes: RwLock::default(),
            leaderboard: Default::default(),
        }
    }
}
