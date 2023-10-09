use std::collections::HashMap;

use async_graphql::{Context, Object, SimpleObject, Subscription, ID};
use futures_util::{Stream, StreamExt};
use serde::Deserialize;
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

const DATA_FILE: &str = include_str!("../data.json");

impl InMemoryDb {
    pub(crate) async fn next_question(&self, quiz_id: &ID) -> Option<Question> {
        let mut quizzes = self.quizzes.write().await;
        let quiz = quizzes.get_mut(quiz_id)?;

        if quiz.current_question == -1 {
            // Clean the leaderboard
            let _ = self.leaderboard.write().await.remove(quiz_id);
        }
        quiz.current_question += 1;

        if quiz.current_question == quiz.questions.len() as i8 {
            quiz.current_question = -1;
            // If there is no more question
            None
        } else {
            let question = quiz.questions.get(quiz.current_question as usize).cloned();

            question
        }
    }

    pub(crate) async fn get_player_points(&self, player_id: &ID, quiz_id: &ID) -> Option<usize> {
        self.leaderboard
            .read()
            .await
            .get(quiz_id)?
            .get(player_id)
            .cloned()
    }

    pub(crate) async fn compute_leaderboard(&self, quiz_id: &ID) -> Option<Leaderboard> {
        let quiz = self.quizzes.read().await.get(quiz_id)?.clone();
        self.leaderboard.read().await.get(quiz_id).map(|players| {
            let mut list: Vec<Player> = players
                .iter()
                .map(|(player_id, points)| Player {
                    id: player_id.clone(),
                    points: *points,
                    quiz_id: quiz_id.clone(),
                })
                .collect();

            list.sort_by(|a, b| b.points.cmp(&a.points));

            Leaderboard { quiz, list }
        })
    }

    pub(crate) async fn answer(
        &self,
        player_id: &ID,
        quiz_id: &ID,
        _question_id: &ID,
        choice_id: Option<&ID>,
    ) -> Option<(Response, Leaderboard)> {
        let quiz = dbg!(self.get_quiz(quiz_id).await)?;
        if quiz.current_question < 0 {
            return None;
        }
        let current_question = dbg!(quiz.questions.get(quiz.current_question as usize))?;
        let right_choice = dbg!(current_question
            .choices
            .iter()
            .find(|c| c.id == current_question.good_answer))?;
        let incr = match choice_id {
            Some(choice_id) => {
                if &right_choice.id == choice_id {
                    1
                } else {
                    0
                }
            }
            None => 0,
        };
        self.leaderboard
            .write()
            .await
            .entry(quiz_id.clone())
            .and_modify(|quiz| {
                quiz.entry(player_id.clone())
                    .and_modify(|score| {
                        *score += incr;
                    })
                    .or_insert(incr);
            })
            .or_insert_with(|| [(player_id.clone(), incr)].into());

        let leaderboard = dbg!(self.compute_leaderboard(quiz_id).await);

        leaderboard.map(|l| {
            (
                Response {
                    success: incr == 1,
                    right_choice: right_choice.clone(),
                },
                l,
            )
        })
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
        let quiz_stream = self.quizzes.read().await.get(quiz_id).map(|s| {
            BroadcastStream::new(s.subscribe())
                .filter_map(|e| async move { e.ok() })
                .boxed()
        });

        match quiz_stream {
            Some(quiz_stream) => quiz_stream,
            None => {
                let (tx, rx) = broadcast::channel(2);
                self.quizzes.write().await.insert(quiz_id.clone(), tx);
                BroadcastStream::new(rx)
                    .filter_map(|e| async move { e.ok() })
                    .boxed()
            }
        }
    }

    pub(crate) async fn next_question(&self, quiz_id: &ID, question: Question) {
        if let Some(quiz_broker) = self.quizzes.read().await.get(quiz_id) {
            let _err = quiz_broker.send(question);
        }
    }

    pub(crate) async fn unsubscribe_quiz(&self, quiz_id: &ID) {
        self.quizzes.write().await.remove(quiz_id);
    }

    pub(crate) async fn unsubscribe_leaderboard(&self, quiz_id: &ID) {
        self.leaderboard.write().await.remove(quiz_id);
    }

    pub(crate) async fn subscribe_leaderboard(
        &self,
        quiz_id: &ID,
    ) -> impl Stream<Item = Leaderboard> {
        let leaderboard_stream = self.leaderboard.read().await.get(quiz_id).map(|s| {
            BroadcastStream::new(s.subscribe())
                .filter_map(|e| async move { e.ok() })
                .boxed()
        });

        match leaderboard_stream {
            Some(leaderboard_stream) => leaderboard_stream,
            None => {
                let (tx, rx) = broadcast::channel(2);
                self.leaderboard.write().await.insert(quiz_id.clone(), tx);
                BroadcastStream::new(rx)
                    .filter_map(|e| async move { e.ok() })
                    .boxed()
            }
        }
    }

    pub(crate) async fn broadcast_leaderboard(&self, quiz_id: &ID, leaderboard: Leaderboard) {
        if let Some(leaderboard_broker) = self.leaderboard.read().await.get(quiz_id) {
            let err = leaderboard_broker.send(leaderboard);
            if let Err(err) = err {
                eprintln!("error when broadcasting leaderboard: {err}");
            }
        }
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

#[derive(Clone, SimpleObject, Debug)]
pub(crate) struct Player {
    pub(crate) id: ID,
    #[graphql(external)]
    pub(crate) quiz_id: ID,
    pub(crate) points: usize,
}

#[derive(Clone, SimpleObject, Debug, Deserialize)]
pub(crate) struct Quiz {
    pub(crate) id: ID,
    pub(crate) title: String,
    pub(crate) questions: Vec<Question>,
    #[graphql(skip, default)]
    pub(crate) current_question: i8,
}

impl Default for Quiz {
    fn default() -> Self {
        Self {
            id: Default::default(),
            title: Default::default(),
            questions: Default::default(),
            current_question: -1,
        }
    }
}

#[derive(SimpleObject, Clone, Debug, Deserialize)]
pub(crate) struct Question {
    pub(crate) id: ID,
    pub(crate) title: String,
    pub(crate) choices: Vec<Choice>,
    #[graphql(skip)]
    pub(crate) good_answer: ID,
}

#[derive(Clone, Default, SimpleObject, Debug, Deserialize)]
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
        let quizzes: Vec<Quiz> = serde_json::from_str(DATA_FILE)
            .expect("cannot deserialize the json file containing quizzes");
        Self {
            quizzes: RwLock::new(quizzes.into_iter().fold(HashMap::new(), |mut acc, quiz| {
                acc.insert(quiz.id.clone(), quiz);
                acc
            })),
            leaderboard: Default::default(),
        }
    }
}
