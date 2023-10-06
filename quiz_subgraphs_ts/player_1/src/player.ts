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
import crypto from "crypto";

function uuid() {
  return crypto.randomBytes(6).toString("hex");
}

interface Player {
  id: string;
  name: string;
  quizId: string;
}

const pubsub = new PubSub();

const typeDefs = gql(readFileSync("./player.graphql", "utf-8"));

function playersForAQuiz(quizId: string) {
  return [];
}

const resolvers = {
  Player: {
    __resolveReference(reference: Player) {
      return {
        id: reference.id,
        name: "test",
        quizId: reference.quizId,
      };
    },
  },

  Query: {
    player(_: undefined, { playerId }: { playerId: string }) {
      return {
        id: playerId,
        name: "test",
        quizId: "0",
      };
    },

    playersForAQuiz(_: undefined, { quizId }: { quizId: string }) {
      return playersForAQuiz(quizId);
    },
  },

  Mutation: {
    createPlayer(
      _: undefined,
      { userName, quizId }: { userName: string; quizId: string }
    ): Player {
      const player: Player = {
        id: uuid(),
        name: userName,
        quizId,
      };
      return player;
    },
  },

  Subscription: {
    playersForAQuiz: {
      subscribe() {
        return pubsub.asyncIterator(["CREATE_PLAYER"]);
      },
    },
  },
};

const app = express();
const httpServer = createServer(app);

const schema = buildSubgraphSchema({ typeDefs, resolvers });
const wsServer = new WebSocketServer({
  server: httpServer,
  path: "/ws",
});
const serverCleanup = useServer({ schema }, wsServer);

const server = new ApolloServer({
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
app.use("/", cors(), json(), expressMiddleware(server));

const PORT = process.env.PORT || 4006;
httpServer.listen(PORT, () => {
  console.log(`🚀 Player subgraph ready at http://localhost:${PORT}/`);
});
