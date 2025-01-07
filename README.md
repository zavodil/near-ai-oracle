# NEAR Permissionless AI General Purpose Oracle Contract

## Overview

This contract provides a permissionless, decentralized AI-powered oracle on the NEAR blockchain. It allows users to create and query oracle items, where data is fetched from external URLs and processed by AI agents to provide a consensus-driven response. The oracle is fully autonomous, free to use, and allows anyone to contribute data and run agents for query processing.

## How It Works

### 1. Creating an Oracle Item
To create an oracle item, a user submits the following:
- A **query prompt** that defines the question to be answered.
- A list of **URLs** where relevant information for the query can be found.
- The **public key** of an AI agent on NEAR that will process the data.

### 2. Submitting Data to the Oracle
Anyone can prepare the data for the oracle by running an AI agent to process it and then submit the data to the oracle. The agent performs the following steps:

- **Reads** the prompt and list of URLs from the smart contract.
- **Fetches content** from the provided URLs.
- **Summarizes** the responses and attempts to determine a consensus answer.
- **Creates** a transaction payload with the agent's signature. Any user can then send the transaction to the hub, paying for the gas.

### Benefits
- **Permissionless**: Any user can interact with the oracle by submitting URLs or running agents to provide answers.
- **Autonomous Agents**: Agents are fully autonomous and execute immutable code. They read the prompt and other data from the contract.
- **No Admin or Validators**: There are no central administrators or validators in the oracle system. Anyone can add new oracle items and submit data.
- **Free Usage**: The oracle is designed to be free, and users are only responsible for paying for gas when submitting data to the NEAR hub.
- **Unstoppable**: No one can shut down the oracle. It is fully decentralized and autonomous, meaning it cannot be stopped or censored.


### Example Oracle Item
**Prompt**: Current president of the USA  
**URLs**:
- ["https://www.whitehouse.gov/", "https://en.wikipedia.org/wiki/President_of_the_United_States", "https://www.usa.gov/presidents"]

If at least two URLs return consistent answers, they are summarized and sent to the oracle. 

Example result:
```
Current president of the USA according to https://www.whitehouse.gov/ is Joe Biden
Current president of the USA according to https://en.wikipedia.org/wiki/President_of_the_United_States is Joe Biden
Current president of the USA according to https://www.usa.gov/presidents is Joseph Biden

Current president of the USA is Joe Biden
```

**Transaction**: https://nearblocks.io/txns/CGKx2QwC9aLYM3EdwUNd9xbFbJkc1HeNkztUMCrMRNNM


Contract Methods
====

`new() -> Contract`

Initializes the contract and prepares it to store oracle items.

---

`insert(name: String, item: OracleItem)`

Registers a new oracle item with the given name and associated data (OracleItem).

---

`get_value(name: String) -> Option<String>`

Returns the current value of an oracle item by its name.

---

`get_agent_data(name: String) -> Option<OracleItem>`

Returns the details of the oracle item (including the agent's public key, URLs, and prompt) by its name.

---

`add_value(name: String, completion: AgentCompletion)`

Adds a value to an oracle item after verifying the data submitted by an AI agent. The data is checked for consistency and legitimacy before being added to the oracle.

---

`verify_oracle_item(item: OracleItem, completion: AgentCompletion) -> bool`

Verifies the legitimacy of the AI agent's submission to ensure that it is consistent with the oracle item’s parameters.

---

`verify(public_key: String, signature: String, agent_name: String, model: String, messages: Vec<Message>, temperature: Option<f64>, max_tokens: Option<u32>, completion: String, verbose: Option<bool>) -> bool`

Verifies the signature of the AI agent’s response to ensure the data is authentic and comes from the correct agent.

Example of Usage
====

1. Registering an Oracle Item

```
export CONTRACT_ID=oracle.ai-is-near.near
near call $CONTRACT_ID insert '{"name": "potus/0.4", "item": {"agent_name": "zavodil.near/potus/0.4", "agent_public_key": "3WkfBwtJxQNYVuGmF4XytuPpnEwjbkpp6uGsXVLoJ5aZ", "prompt": "Current president of the USA", "urls": ["https://www.whitehouse.gov/", "https://en.wikipedia.org/wiki/President_of_the_United_States", "https://www.usa.gov/presidents"]}}' --accountId ai-is-near.near
```

2. Submitting a Response with AI Agent

https://app.near.ai/agents/zavodil.near/potus/0.4/run

3. Querying the Oracle

```
near view $CONTRACT_ID get_agent_data '{"name":"potus/0.4"}'
```

Contract Code
===
The full contract code is written in Rust and utilizes the NEAR SDK to interact with the blockchain. It includes the following components:

- OracleItem: Represents the data structure for each oracle item.
- AgentCompletion: Contains the response data from the AI agent.
- Methods for registering items, submitting responses, and querying the oracle.
- Verification functions to ensure that agent responses are authentic and match the provided public key and signature.