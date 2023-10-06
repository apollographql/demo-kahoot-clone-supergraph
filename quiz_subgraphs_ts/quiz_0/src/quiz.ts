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
  // TODO
}

interface Question {
  // TODO
}

interface Quiz {
  // TODO
}

interface Player {
  // TODO
}

interface Leaderboard {
  // TODO
}

interface Response {
  // TODO
}

type ContextValue = {
  playerId: string | undefined;
};

const pubsub = new PubSub();

const QUIZZES: Record<string, Quiz> = {
  "0": {},
};

const LEADERBOARD: Record<string, Record<string, number>> = {};

function getLeaderboard(quizId: string): Leaderboard {
  return {};
}

const typeDefs = gql(readFileSync("./quiz.graphql", "utf-8"));

const resolvers = {
  Quiz: {
    __resolveReference(reference: Quiz) {
      // TODO
    },
  },

  Player: {
    __resolveReference(reference: Player) {
      // TODO
    },
  },

  Query: {},
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
