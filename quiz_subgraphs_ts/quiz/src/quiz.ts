import { ApolloServer } from "@apollo/server";
import { expressMiddleware } from "@apollo/server/express4";
import cors from "cors";
import bodyParser from "body-parser";
const { json } = bodyParser;
import express from "express";
import { createServer } from "http";
import { ApolloServerPluginDrainHttpServer } from "@apollo/server/plugin/drainHttpServer";
import { buildSubgraphSchema } from "@apollo/subgraph";
import { WebSocketServer } from "ws";
import { useServer } from "graphql-ws/lib/use/ws";
import gql from "graphql-tag";
import { readFileSync } from "fs";
import { PubSub } from "graphql-subscriptions";

interface Choice {
  id: string;
  text: string;
}

interface Question {
  id: string;
  title: string;
  choices: Choice[];
  goodAnswer: string;
}

interface Quiz {
  id: string;
  title: string;
  questions: Question[];
  currentQuestion: number;
}

interface Player {
  id: string;
  quizId: string;
  points: number;
}

interface Leaderboard {
  quiz: Quiz;
  list: Player[];
}

interface Response {
  success: boolean;
  rightChoice: Choice;
}

type ContextValue = {
  playerId: string | undefined;
};

const pubsub = new PubSub();

const QUIZZES: Record<string, Quiz> = {
  "0": {
    id: "0",
    title: "Subscription quiz",
    questions: [
      {
        id: "0",
        title:
          "How many protocols are currently supported by the Apollo Router for subscriptions?",
        choices: [
          {
            id: "0",
            text: "1",
          },
          {
            id: "1",
            text: "2",
          },
          {
            id: "2",
            text: "3",
          },
          {
            id: "3",
            text: "4",
          },
        ],
        goodAnswer: "1",
      },
      {
        id: "1",
        title:
          "In this Kahoot clone app, which protocol is used for the connection between the client and the router?",
        choices: [
          {
            id: "0",
            text: "HTTP multipart connection",
          },
          {
            id: "1",
            text: "Sever-sent events (SSE)",
          },
          {
            id: "2",
            text: "WebSocket protocol",
          },
          {
            id: "3",
            text: "Google Remote Procedure Call (gRPC)",
          },
        ],
        goodAnswer: "0",
      },
    ],
    currentQuestion: 0,
  },
};

const LEADERBOARD: Record<string, Record<string, number>> = {};

function getLeaderboard(quizId: string): Leaderboard {
  const leaderboardData = LEADERBOARD[quizId];
  const quiz = QUIZZES[quizId];

  const leaderboard: Leaderboard = {
    quiz,
    list: [],
  };

  if (leaderboardData) {
    leaderboard.list = Object.keys(leaderboardData).map((playerId) => ({
      id: playerId,
      quizId,
      points: leaderboardData[playerId],
    }));
  }

  return leaderboard;
}

const typeDefs = gql(readFileSync("./quiz.graphql", "utf-8"));

const resolvers = {
  Quiz: {
    __resolveReference(reference: Quiz) {
      return QUIZZES[reference.id];
    },
  },

  Query: {
    allQuizzes() {
      return Object.values(QUIZZES);
    },

    leaderboardForQuiz(_: undefined, { id }: { id: string }) {
      return getLeaderboard(id);
    },
  },

  Mutation: {
    answer(
      _: undefined,
      {
        quizId,
        questionId,
        choiceId,
      }: { quizId: string; questionId: string; choiceId: string },
      { playerId }: ContextValue
    ): Response {
      if (!playerId) {
        throw new Error("cannot find the player header");
      }

      const quiz = QUIZZES[quizId];
      if (quiz.currentQuestion < 0) {
        throw new Error("no current question");
      }

      const currentQuestion =
        quiz.questions.find(
          (question) => +question.id === quiz.currentQuestion
        ) || quiz.questions[0];
      const rightChoice =
        currentQuestion.choices.find(
          (choice) => choice.id === currentQuestion.goodAnswer
        ) || currentQuestion.choices[0];
      const increment = choiceId === rightChoice?.id ? 1 : 0;

      if (LEADERBOARD[quizId]) {
        const playerScore = LEADERBOARD[quiz.id][playerId];
        if (playerScore) {
          LEADERBOARD[quizId][playerId] += increment;
        } else {
          LEADERBOARD[quizId][playerId] = increment;
        }
      } else {
        LEADERBOARD[quizId] = { [playerId]: increment };
      }

      pubsub.publish("ANSWER", { leaderboardForQuiz: getLeaderboard(quizId) });

      return {
        success: !!increment,
        rightChoice,
      };
    },

    nextQuestion(_: undefined, { quizId }: { quizId: string }) {
      let question: Question | undefined;
      const quiz = QUIZZES[quizId];

      if (!quiz) {
        return question;
      }

      if (quiz.currentQuestion === -1) {
        delete LEADERBOARD[quizId];
      }

      quiz.currentQuestion += 1;

      if (quiz.currentQuestion === Object.keys(quiz.questions).length) {
        quiz.currentQuestion = 0;
      }

      question = quiz.questions.find(
        (question) => +question.id === quiz.currentQuestion
      );

      pubsub.publish("NEXT_QUESTION", { newQuestion: question });

      return question;
    },
  },

  Subscription: {
    newQuestion: {
      subscribe() {
        return pubsub.asyncIterator(["NEXT_QUESTION"]);
      },
    },

    leaderboardForQuiz: {
      subscribe() {
        return pubsub.asyncIterator(["ANSWER"]);
      },
    },
  },
};

const app = express();
const httpServer = createServer(app);

const schema = buildSubgraphSchema({ typeDefs, resolvers } as any);
const wsServer = new WebSocketServer({
  server: httpServer,
  path: "/ws",
});
const serverCleanup = useServer({ schema }, wsServer);

const server = new ApolloServer<ContextValue>({
  schema,
  plugins: [
    ApolloServerPluginDrainHttpServer({ httpServer }),
    {
      async serverWillStart() {
        return {
          async drainServer() {
            await serverCleanup.dispose();
          },
        };
      },
    },
  ],
});

await server.start();
app.use(
  "/",
  cors(),
  json(),
  expressMiddleware(server, {
    async context({ req }) {
      return {
        playerId: req.headers.player,
      } as ContextValue;
    },
  })
);

const PORT = process.env.PORT || 4005;
httpServer.listen(PORT, () => {
  console.log(`🚀 Quiz subgraph ready at http://localhost:${PORT}/`);
});
