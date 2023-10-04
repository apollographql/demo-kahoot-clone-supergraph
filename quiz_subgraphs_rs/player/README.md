# Getting started

+ Install Rust via rustup, go on the [rustup website](https://rustup.rs/) and follow the instructions.
+ Launch the subgraph using `cargo run`
+ It will expose GraphQL server on http://localhost:4005 by default (without any path in the URL).
+ If you want to change the port, simply expose another `PORT` env variable.
+ If you use the explorer, be careful the websocket url is exposed on `ws://localhost:4005/ws`

# Test a scenario

+ Subscribe on new questions:

```graphql
subscription {
  newQuestion(quizId: 0) {
    id
    title
    choices {
      id
      text
    }
  }
}
```

+ Subscribe on new leaderboard:

```graphql
subscription SubscriptionRoot {
  leaderboardForQuiz(id: 0) {
    list {
      id
      points
    }
  }
}
```



+ Trigger a new question:

```graphql
mutation {
  nextQuestion(quizId: 0) {
    title
  }
}
```


+ Answer to the current question (it should include a `player` request header containing the player ID, you can use "1" for example):

```graphql
mutation {
  answer(quizId: 0, questionId: 0, choiceId: 1) {
    success
    rightChoice {
      id
      text
    }
  }
}
```

+ You'll be able to see in both subscriptions the right data, when you answered to all questions both subscriptions will be closed by the server