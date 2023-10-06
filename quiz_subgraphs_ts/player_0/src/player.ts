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

function uuid() {
  return Math.random().toString(36).substring(2, 9);
}

interface Player {
  // TODO
}

const pubsub = new PubSub();

const PLAYERS: Record<string, Player> = {};

const typeDefs = gql(readFileSync("./player.graphql", { encoding: "utf-8" }));

const resolvers = {
  Query: {
    test(_: any, { id }: { id: string }) {
      return "test";
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
