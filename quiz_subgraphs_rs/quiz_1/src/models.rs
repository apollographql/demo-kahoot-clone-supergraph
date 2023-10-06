use std::collections::HashMap;

use async_graphql::{Context, Object, SimpleObject, Subscription, ID};
use futures_util::{stream, Stream, StreamExt};
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
        Some(Question::default())
    }

    pub(crate) async fn get_player_points(&self, player_id: &ID, quiz_id: &ID) -> Option<usize> {
        Some(2)
    }

    pub(crate) async fn compute_leaderboard(&self, quiz_id: &ID) -> Option<Leaderboard> {
        Some(Leaderboard::default())
    }

    pub(crate) async fn answer(
        &self,
        player_id: &ID,
        quiz_id: &ID,
        _question_id: &ID,
        choice_id: Option<&ID>,
    ) -> Option<(Response, Leaderboard)> {
        Some((Response::default(), Leaderboard::default()))
    }

    pub(crate) async fn get_quiz(&self, quiz_id: &ID) -> Option<Quiz> {
        self.quizzes.read().await.get(quiz_id).cloned()
    }

    pub(crate) async fn get_quizzes(&self) -> Vec<Quiz> {
        self.quizzes.read().await.values().cloned().collect()
    }
}

#[derive(Default)]
pub(crate) struct InMemoryBroker {
    quizzes: RwLock<HashMap<ID, broadcast::Sender<Question>>>,
    leaderboard: RwLock<HashMap<ID, broadcast::Sender<Leaderboard>>>,
}

impl InMemoryBroker {
    pub(crate) async fn subscribe_quiz(&self, quiz_id: &ID) -> impl Stream<Item = Question> {
        stream::iter(vec![Question {
            id: "0".into(),
            title: "Test".to_string(),
            good_answer: "0".into(),
            choices: vec![
                Choice {
                    id: "0".into(),
                    text: String::from("test 1"),
                },
                Choice {
                    id: "1".into(),
                    text: String::from("test 2"),
                },
                Choice {
                    id: "2".into(),
                    text: String::from("test 3"),
                },
                Choice {
                    id: "3".into(),
                    text: String::from("test 4"),
                },
            ],
        }])
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
        stream::iter(vec![Leaderboard::default()])
    }

    pub(crate) async fn broadcast_leaderboard(&self, quiz_id: &ID, leaderboard: Leaderboard) {
        todo!()
    }
}

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn all_quizzes<'ctx>(&self, ctx: &Context<'ctx>) -> Vec<Quiz> {
        let in_memory_db: &InMemoryDb = ctx.data_unchecked();

        in_memory_db.get_quizzes().await
    }

    async fn leaderboard_for_quiz<'ctx>(&self, ctx: &Context<'ctx>, id: ID) -> Option<Leaderboard> {
        let in_memory_db: &InMemoryDb = ctx.data_unchecked();

        in_memory_db.compute_leaderboard(&id).await
    }

    #[graphql(entity)]
    async fn find_player_by_id_and_quiz_id<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        id: ID,
        quiz_id: ID,
    ) -> Player {
        let in_memory_db: &InMemoryDb = ctx.data_unchecked();
        let points = in_memory_db
            .get_player_points(&id, &quiz_id)
            .await
            .unwrap_or_default();
        Player {
            id,
            quiz_id,
            points,
        }
    }

    #[graphql(entity)]
    async fn find_quiz_by_id<'ctx>(&self, ctx: &Context<'ctx>, id: ID) -> Option<Quiz> {
        let in_memory_db: &InMemoryDb = ctx.data_unchecked();
        in_memory_db.get_quiz(&id).await
    }
}

pub(crate) struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn new_question<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        quiz_id: ID,
    ) -> impl Stream<Item = Question> {
        let in_memory_broker: &InMemoryBroker = ctx.data_unchecked();

        in_memory_broker.subscribe_quiz(&quiz_id).await
    }

    async fn leaderboard_for_quiz<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        id: ID,
    ) -> impl Stream<Item = Leaderboard> {
        let in_memory_broker: &InMemoryBroker = ctx.data_unchecked();

        in_memory_broker.subscribe_leaderboard(&id).await
    }
}

pub(crate) struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn answer<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        quiz_id: ID,
        question_id: ID,
        choice_id: Option<ID>,
    ) -> async_graphql::Result<Response> {
        let player_id: &ID = match ctx.data_opt() {
            Some(id) => id,
            None => {
                return Err("cannot find the player header".into());
            }
        };

        let in_memory_db: &InMemoryDb = ctx.data_unchecked();

        let (response, leaderboard) = in_memory_db
            .answer(player_id, &quiz_id, &question_id, choice_id.as_ref())
            .await
            .ok_or_else(|| async_graphql::Error::new("cannot answer"))?;

        let in_memory_broker: &InMemoryBroker = ctx.data_unchecked();
        in_memory_broker
            .broadcast_leaderboard(&quiz_id, leaderboard)
            .await;

        Ok(response)
    }

    async fn next_question<'ctx>(&self, ctx: &Context<'ctx>, quiz_id: ID) -> Option<Question> {
        let in_memory_db: &InMemoryDb = ctx.data_unchecked();
        let in_memory_broker: &InMemoryBroker = ctx.data_unchecked();

        let _ = in_memory_broker.subscribe_quiz(&quiz_id).await;
        let question = in_memory_db.next_question(&quiz_id).await;
        match question {
            Some(question) => {
                in_memory_broker
                    .next_question(&quiz_id, question.clone())
                    .await;
                Some(question)
            }
            None => {
                in_memory_broker.unsubscribe_quiz(&quiz_id).await;
                in_memory_broker.unsubscribe_leaderboard(&quiz_id).await;
                None
            }
        }
    }
}

#[derive(Clone, SimpleObject, Debug, Default)]
pub(crate) struct Player {
    pub(crate) id: ID,
    #[graphql(external)]
    pub(crate) quiz_id: ID,
    pub(crate) points: usize,
}

#[derive(Clone, Default, SimpleObject, Debug)]
pub(crate) struct Quiz {
    pub(crate) id: ID,
    pub(crate) title: String,
    pub(crate) questions: Vec<Question>,
    #[graphql(skip)]
    pub(crate) current_question: i8,
}

#[derive(SimpleObject, Clone, Debug, Default)]
pub(crate) struct Question {
    pub(crate) id: ID,
    pub(crate) title: String,
    pub(crate) choices: Vec<Choice>,
    #[graphql(skip)]
    pub(crate) good_answer: ID,
}

#[derive(Clone, Default, SimpleObject, Debug)]
pub(crate) struct Choice {
    pub(crate) id: ID,
    pub(crate) text: String,
}

#[derive(Clone, Default, SimpleObject, Debug)]
pub(crate) struct Leaderboard {
    pub(crate) quiz: Quiz,
    pub(crate) list: Vec<Player>,
}

#[derive(Clone, Default, SimpleObject, Debug)]
pub(crate) struct Response {
    pub(crate) success: bool,
    pub(crate) right_choice: Choice,
}

impl Default for InMemoryDb {
    fn default() -> Self {
        Self {
            quizzes: RwLock::new(
                [(
                    ID::from("0"),
                    Quiz {
                        id: ID::from("0"),
                        title: String::from("Subscription quiz"),
                        questions: vec![Question {
                            id: ID::from("0"),
                            title: String::from(
                                "How many protocols are currently supported by the Apollo Router for subscriptions?",
                            ),
                            choices: vec![
                                Choice {
                                    id: ID::from("0"),
                                    text: String::from("1"),
                                },
                                Choice {
                                    id: ID::from("1"),
                                    text: String::from("2"),
                                },
                                Choice {
                                    id: ID::from("2"),
                                    text: String::from("3"),
                                },
                                Choice {
                                    id: ID::from("3"),
                                    text: String::from("4"),
                                },
                            ],
                            good_answer: ID::from("1"),
                        }, Question {
                            id: ID::from("1"),
                            title: String::from(
                                "In this Kahoot clone app, which protocol is used for the connection between the client and the router?",
                            ),
                            choices: vec![
                                Choice {
                                    id: ID::from("0"),
                                    text: String::from("HTTP multipart connection"),
                                },
                                Choice {
                                    id: ID::from("1"),
                                    text: String::from("Server-side events (SSE)"),
                                },
                                Choice {
                                    id: ID::from("2"),
                                    text: String::from("WebSockets"),
                                },
                                Choice {
                                    id: ID::from("3"),
                                    text: String::from("Google Remote Procedure Call (gRPC)"),
                                },
                            ],
                            good_answer: ID::from("0"),
                        }],
                        current_question: -1,
                    },
                )]
                .into(),
            ),
            leaderboard: Default::default(),
        }
    }
}
