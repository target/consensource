<p align="center">
  <h1 align="center">ConsenSource</h3>
</p>

[![Build Status](https://travis-ci.org/target/consensource.svg?branch=master)](https://travis-ci.org/target/consensource)

ConsenSource is a blockchain platform for verifying sustainability-certified suppliers. This application also serves as a transparent platform for displaying industry certifications and audit data between parties (certificate accrediting groups, certificate issuers, suppliers, etc.).

ConsenSource runs on [Hyperledger Sawtooth](https://github.com/hyperledger/sawtooth-core), an enterprise blockchain solution.

1. [Overview](#overview)

2. [Docker Images](#docker-images)

3. [Running ConsenSource](#running-consensource)

4. [Documentation](#documentation)

5. [Contributing](#contributing)

6. [Maintainers](#maintainers)

## Overview

ConsenSource consists of several components

- [consensource-api](https://github.com/target/consensource-api) - A REST API that provides endpoints for accessing blockchain data, user information, and state data.

- [consensource-processor](https://github.com/target/consensource-processor) - A transaction processor for handling ConsenSource transaction logic.

- [consensource-sds](https://github.com/target/consensource-sds) - An event subscriber that listens to blockchain events in order to parse incoming data to be stored in an off-chain reporting database.

- [consensource-cli](https://github.com/target/consensource-cli) - A command line interface for creating ConsenSource blockchain transactions including agents, organizations, certificates, standards and accreditations.

- [consensource-database](https://github.com/target/consensource-cli) - A database library for handling off-chain storage.

- [consensource-common](https://github.com/target/consensource-common) - A common repo for housing custom Rust libraries, protobuf definitions, and Dockerfiles.

- [consensource-ui](https://github.com/target/consensource-ui) - Multiple user interfaces for each entity (standards bodies, certification bodies, suppliers and retailers) to interact with the ConsenSource blockchain.

## Docker Images

We provide Docker images for each of the components (described above) for running and deploying on your preferred container orchestration platform.

[target/consensource-api](https://hub.docker.com/repository/docker/target/consensource-api) - Rust v1.44 nightly image, consensource-api binary

[target/consensource-processor](https://hub.docker.com/repository/docker/target/consensource-processor) - Rust v1.44 stable image, consensource-processor binary

[target/consensource-sds](https://hub.docker.com/repository/docker/target/consensource-sds) - Rust v1.44 stable image, consensource-sds binary

[target/consensource-cli](https://hub.docker.com/repository/docker/target/consensource-cli) - Rust latest stable image, consensource-cli binary

_Note: we manage our own Rust base image in the [target/consensource-common repo](https://github.com/target/consensource-common/tree/master/docker)_

## Running ConsenSource

We provide two examples for running ConsenSource locally.

1. [docker-compose](docker-compose/README.md)

1. [kubernetes](kubernetes/README.md)

## Documentation

For more info on specific components, glossary and FAQs, please [visit the ConsenSource docs](https://target.github.io/consensource-docs/).

## Contributing

Please do! Check [CONTRIBUTING.md](./.github/CONTRIBUTING.md) for info.

## Maintainers

- [Adeeb Ahmed](https://github.com/adeebahmed)
- [Patrick Erichsen](https://github.com/Patrick-Erichsen)
- [Raphael Santo Domingo](https://github.com/pa3ng)
- [Trevor McDonald](https://github.com/trevormcdonald)
