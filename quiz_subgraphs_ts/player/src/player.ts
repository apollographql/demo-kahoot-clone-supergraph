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

function uuid() {
  return Math.random().toString(36).substring(2, 9);
}

interface Player {
  id: string;
  name: string;
  quizId: string;
}

const pubsub = new PubSub();

const PLAYERS: Record<string, Player> = {};

const typeDefs = gql(readFileSync("./player.graphql", { encoding: "utf-8" }));

function playersForAQuiz(quizId: string) {
  return Object.values(PLAYERS).filter((player) => player.quizId === quizId);
}

const resolvers = {
  Player: {
    __resolveReference(reference: Player) {
      return Object.values(PLAYERS).find(
        (player) =>
          player.id === reference.id && player.quizId === reference.quizId
      );
    },
  },

  Query: {
    player(_: any, { playerId }: { playerId: string }) {
      return PLAYERS[playerId];
    },

    playersForAQuiz(_: any, { quizId }: { quizId: string }) {
      return playersForAQuiz(quizId);
    },
  },

  Mutation: {
    createPlayer(
      _: any,
      { userName, quizId }: { userName: string; quizId: string }
    ): Player {
      const player: Player = {
        id: uuid(),
        name: userName,
        quizId,
      };
      PLAYERS[player.id] = player;
      pubsub.publish("CREATE_PLAYER", {
        playersForAQuiz: playersForAQuiz(quizId),
      });
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
