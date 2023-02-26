# Arpa Node

This crate provides a node side on-chain implementation as well as an off-chain demo to the provided DKG and Threshold-BLS based randomness service(Randcast).

The Arpa Node consists of an event queue, two types of task schedulers and a set of listeners and subscribers. Events are passed within components and drive them to work. All the components and data access layer(with sqlite) are wrapped in a context, which holds and shares all the information needed for the client bin and grpc servers to expose services.

Note that task schedulers manage components and sub-handlers from listeners, subscribers and grpc servers as different task types, instead of DKG or BLS tasks the network publishes.

# Node-client bin

Node-client is a long-running client to run the ARPA node.

With structopt, now it is more explicit and self-explanatory:

```bash
cargo run --bin node-client -- -h
```

# Management grpc server

Management grpc server supports getting states and interacting with a running node. It can be used for scenario tests or DevOps.

Please see `management.proto` for detailed apis.

# Node-account-client bin(WIP)

Node-account-client is a practical tool to generate keystore corresponding to ARPA node format.

# Node-cmd-client bin(WIP)

Node-cmd-client is a practical tool to interact with on-chain contracts for ARPA node owner or administrator.

# User-client bin(WIP)

User-client is a practical tool to interact with on-chain contracts for Randcast users.

Note: Basically for demo use, in real environment a Randcast user should request and receive randomness by extending consumer contract instead of calling controller contract through an EOA directly.

# Dependencies

Install [protoc](https://github.com/hyperium/tonic#dependencies) and [foundry](https://github.com/foundry-rs/foundry#installation), then run

```bash
cargo build
```

# Node Config

Configuration items in `config.yml` are listed here:

- node_committer_rpc_endpoint: Config endpoint to expose committer grpc services. (example: "[::1]:50060")

- node_management_rpc_endpoint: Config endpoint to expose management grpc services. (example: "[::1]:50099")

- node_management_rpc_token: Config token phrase for authenticaing management grpc requests by `authorization` header. (example: "arpa_network")

- provider_endpoint: Config endpoint to interact with chain provider. (example: "[::1]:50052")

- controller_address: Config on-chain arpa network controller contract address. (example: "0x0000000000000000000000000000000000000001")

- data_path(Optional): Config DB file for persistence. (example: "data.sqlite")

- account: Config node identity in the network. There are three available account types.
  - example(not recommended): private_key: "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
  - example:
    ```
    keystore:
        password: env
        path: test.keystore
    ```
  - example:
    ```
    hdwallet:
        mnemonic: env
        path(Optional): "m/44'/60'/0'/0"
        index: 1
        passphrase(Optional): "custom_password"
    ```
- listeners(Optional): Config listeners to run with node client to customize services. By default all the listeners will be enabled. All of them can be disabled by setting a empty value explicitly.
  - example:
  ```
  - Block
  - PreGrouping
  - PostCommitGrouping
  - PostGrouping
  - NewRandomnessTask
  - ReadyToHandleRandomnessTask
  - RandomnessSignatureAggregation
  ```

To protect secrets, several items can be set with literal `env` as placeholder. Their env keys are:

- ARPA_NODE_MANAGEMENT_SERVER_TOKEN (node_management_rpc_token)
- ARPA_NODE_ACCOUNT_PRIVATE_KEY (account, private_key)
- ARPA_NODE_ACCOUNT_KEYSTORE_PASSWORD (account, keystore, password)
- ARPA_NODE_HD_ACCOUNT_MNEMONIC (account, hdwallet, mnemonic)

# Demo Steps

## deploy contract server(different ip endpoints on different chains):

```bash
cargo run --bin controller-server "[::1]:50052"
cargo run --bin adapter-server "[::1]:50053"
```

## run nodes:

```bash
cd crates/arpa-node
cargo run --bin node-client -- -m demo -i 1
cargo run --bin node-client -- -m demo -i 2
cargo run --bin node-client -- -m demo -i 3
```

```bash
cargo run --bin node-client -- -m demo -i 4
cargo run --bin node-client -- -m demo -i 5
cargo run --bin node-client -- -m demo -i 6
```

## use user-client to request randomness:

```bash
cargo run --bin user-client 0x9000000000000000000000000000000000000001 "[::1]:50052" request foo
cargo run --bin user-client 0x9000000000000000000000000000000000000001 "[::1]:50053" request bar
cargo run --bin user-client 0x9000000000000000000000000000000000000001 "[::1]:50052" last_output
```

## use node-cmd-client to get views or call some helper methods(1 - controller 2 - adapter):

```bash
cargo run --bin node-cmd-client 0x90000000000000000000000000000000000000ad "[::1]:50052" "1" set_initial_group "[::1]:50053"
cargo run --bin node-cmd-client 0x9000000000000000000000000000000000000001 "[::1]:50052" "1" get_group "1"
cargo run --bin node-cmd-client 0x9000000000000000000000000000000000000001 "[::1]:50053" "2" get_group "1"
```

## 1 MainChain Demo(Happy Path) Example:

```bash
# deploy contract
cargo run --bin controller-server "[::1]:50052"
```

```bash
# run 3 nodes to prepare a BLS-ready group
cd crates/arpa-node
cargo run --bin node-client -- -m demo -i 1
cargo run --bin node-client -- -m demo -i 2
cargo run --bin node-client -- -m demo -i 3
```

```bash
# check result by view get_group
cargo run --bin node-cmd-client 0x9000000000000000000000000000000000000001 "[::1]:50052" "2" get_group "1"
```

```bash
# now we can request randomness task as a user on main chain
cargo run --bin user-client 0x9000000000000000000000000000000000000001 "[::1]:50052" request foo
cargo run --bin user-client 0x9000000000000000000000000000000000000001 "[::1]:50052" last_output
cargo run --bin user-client 0x9000000000000000000000000000000000000001 "[::1]:50052" request bar
cargo run --bin user-client 0x9000000000000000000000000000000000000001 "[::1]:50052" last_output
# verify result by view last_output and node logs
```