import { ApolloServer } from "@apollo/server";
import { expressMiddleware } from "@apollo/server/express4";
import { ApolloServerPluginDrainHttpServer } from "@apollo/server/plugin/drainHttpServer";
import { buildSubgraphSchema } from "@apollo/subgraph";
import bodyParser from "body-parser";
import cors from "cors";
import express from "express";
import { readFileSync } from "fs";
import { PubSub } from "graphql-subscriptions";
import gql from "graphql-tag";
import { useServer } from "graphql-ws/lib/use/ws";
import { createServer } from "http";
import { WebSocketServer } from "ws";
const { json } = bodyParser;

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
  return {
    quiz: {
      id: quizId,
      title: "test",
      currentQuestion: 0,
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
      ],
    },
    list: [
      {
        id: "0",
        points: 1,
        quizId,
      },
    ],
  };
}

const typeDefs = gql(readFileSync("./quiz.graphql", "utf-8"));

const resolvers = {
  Quiz: {
    __resolveReference(reference: Quiz) {
      return QUIZZES[reference.id];
    },
  },

  Player: {
    __resolveReference(reference: Player) {
      const leaderboard = getLeaderboard(reference.quizId);
      return leaderboard.list.find(
        (player) =>
          player.id === reference.id && player.quizId === reference.quizId
      );
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
      return {
        success: false,
        rightChoice: {
          id: "0",
          text: "test",
        },
      };
    },

    nextQuestion(_: undefined, { quizId }: { quizId: string }) {
      return {
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
      };
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
