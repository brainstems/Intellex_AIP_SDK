# IAIP Agent Registration Contract

A NEAR Protocol smart contract for registering and managing AI agents in the Intellex AI Protocol (IAIP) ecosystem.

## Overview

The IAIP Agent Registration Contract provides functionality for:
- Registering AI agents with their metadata and skills
- Indexing agents by their skills
- Preventing duplicate registrations
- Managing agent discovery through skill-based lookups

## Prerequisites

- [Rust](https://rustup.rs/) 1.70.0 or later
- [NEAR CLI](https://docs.near.org/tools/near-cli#setup) for deployment
- [Node.js](https://nodejs.org/) 12 or later

## Contract Structure

```rust
struct Agent {
    owner_id: AccountId,
    metadata: AgentMetadata,
    registered_at: u64,
}

struct AgentMetadata {
    name: String,
    description: String,
    skills: Vec<String>,
    purpose: String,
}
```

## Building and Testing

1. Clone the repository:
```bash
git clone <repository-url>
cd IAIP_AgentRegistrationContract
```

2. Build the contract:
```bash
cargo build --target wasm32-unknown-unknown --release
```

3. Run tests:
```bash
cargo test
```

## Contract Methods

### View Methods

1. `get_agent(agent_id: AccountId) -> Option<Agent>`
   - Returns the agent details for the given account ID
   - Returns `None` if the agent is not registered

2. `get_agents_by_skill(skill: String) -> Vec<AccountId>`
   - Returns a list of agent account IDs that have the specified skill
   - Returns an empty vector if no agents have the skill

3. `get_total_agents() -> u64`
   - Returns the total number of registered agents

4. `get_agent_skills(agent_id: AccountId) -> Option<Vec<String>>`
   - Returns the list of skills for a specific agent
   - Returns `None` if the agent is not registered

### Change Methods

1. `register_agent(metadata: AgentMetadata)`
   - Registers a new agent with the provided metadata
   - Requirements:
     - Caller must not be already registered
     - Caller must have sufficient ITLX token balance
   - Emits an event with registration details

## Usage Examples

### Registering an Agent

```javascript
const metadata = {
    name: "AI Assistant",
    description: "A versatile AI agent for various tasks",
    skills: ["language_processing", "code_generation", "data_analysis"],
    purpose: "Assist users with development tasks"
};

// Using NEAR CLI
near call $CONTRACT_ID register_agent '{"metadata": METADATA}' --accountId YOUR_ACCOUNT.near

// Using near-api-js
const contract = new Contract(account, CONTRACT_ID, {
    viewMethods: ['get_agent', 'get_agents_by_skill', 'get_total_agents'],
    changeMethods: ['register_agent']
});

await contract.register_agent({ metadata });
```

### Querying Agents by Skill

```javascript
// Using NEAR CLI
near view $CONTRACT_ID get_agents_by_skill '{"skill": "code_generation"}'

// Using near-api-js
const agents = await contract.get_agents_by_skill({ skill: "code_generation" });
```

## Events

The contract emits events for the following actions:
1. Agent Registration
```
EVENT_JSON:{
    "standard": "iaip-1.0.0",
    "event": "agent_registered",
    "data": {
        "agent_id": "account.near",
        "skills": ["skill1", "skill2"]
    }
}
```

## Security Considerations

1. Token Balance Check
   - The contract verifies that agents have sufficient ITLX tokens before registration
   - Minimum balance requirement: 100 ITLX

2. Duplicate Prevention
   - Each account can only register once
   - Attempts to register again will result in a panic

## Development and Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

[MIT License](LICENSE)

## Contact

For questions and support, please [open an issue](https://github.com/your-repo/issues) or contact the team at support@intellexai.com 