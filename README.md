# Kahoot Clone Demo App - Subgraphs

Welcome to the project for GraphQL Summit 2023 "Federated Subscriptions in GraphOS" workshop!

## ⚠️ Before the workshop

Please complete the following pre-requisites and installations before the workshop.

You will need:

- [ ] An Apollo GraphOS account with either:
  - An Enterprise plan, with at least [Contributor access](https://www.apollographql.com/docs/graphos/org/members/#organization-wide-member-roles) in your organization.
  - An [Enterprise Trial](https://studio.apollographql.com/signup?type=enterprise-trial), for those without an Enterprise plan.
- [ ] [Download the Apollo Router binary](https://www.apollographql.com/docs/router/quickstart#download-options).
- [ ] [Install the Rover CLI](https://www.apollographql.com/docs/rover/getting-started#installation-methods).
- [ ] [Authenticate the Rover CLI](https://www.apollographql.com/docs/rover/configuring#authenticating-with-graphos).
  - You can use a [personal API key](https://www.apollographql.com/docs/graphos/api-keys/#personal-api-keys).
- [ ] Go to the `client` app repository and follow the README instructions there: https://github.com/apollographql/demo-kahoot-clone-client
- [ ] Clone this repository.
- [ ] Follow the installation instructions for your chosen language below.

#### TypeScript

1. Navigate to the `quiz_subgraphs_ts` folder.

1. Navigate to the `quiz` folder.

1. Run `npm i`, then `npm run watch`. This will run the quiz subgraph on port 4005.

1. Navigate to the `quiz_subgraphs_ts/player` folder.

1. Run `npm i` then `npm run watch`. This will run the player subgraph on port 4006.

#### Rust

1. Navigate to the `quiz_subgraphs/rs` folder.

1. Navigate to the `quiz` folder. This will run the quiz subgraph on port 4005.

1. Run `cargo run`.

1. Navigate to the `quiz_subgraphs_rs/player` folder.

1. Run `cargo run`. This will run the player subgraph on port 4006.

## Getting help

For any issues or problems, [join us on Discord](https://discord.gg/graphos) in the #summit-workshops channel.
