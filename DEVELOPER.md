# IAIP Agent Registration Contract - Developer Guide

## Technical Overview

The IAIP Agent Registration Contract is built on NEAR Protocol using the NEAR SDK (v5.0.0). It implements a registration and indexing system for AI agents with the following key components:

### Storage Structure

1. Agent Storage:
```rust
LookupMap<AccountId, Agent>  // Primary agent storage
```

2. Skills Index:
```rust
LookupMap<String, IterableSet<AccountId>>  // Skill-based indexing
```

3. Total Agents Counter:
```rust
u64  // Atomic counter for registered agents
```

## Implementation Details

### 1. Agent Registration Flow

```rust
pub fn register_agent(&mut self, metadata: AgentMetadata) {
    // 1. Verify caller is not registered
    // 2. Check ITLX token balance
    // 3. Create and store agent
    // 4. Index agent by skills
    // 5. Emit registration event
}
```

Key considerations:
- Uses cross-contract calls to verify ITLX balance
- Implements optimistic concurrency for skill indexing
- Emits standardized events for external tracking

### 2. Skill Indexing Implementation

The contract uses `IterableSet` for efficient skill-based lookups:
```rust
// Adding to skill index
let mut skill_agents = self.skills_index
    .get(&skill)
    .unwrap_or_else(|| IterableSet::new(format!("s_{}", skill).as_bytes().to_vec()));
skill_agents.insert(account_id.clone());
self.skills_index.insert(&skill, &skill_agents);
```

### 3. Gas Optimization

The contract implements several gas optimization strategies:
- Efficient storage key design
- Minimal data duplication
- Optimized skill indexing structure

## Testing Guide

### 1. Unit Tests

The contract includes comprehensive unit tests covering:
- Basic registration
- Duplicate prevention
- Skill indexing
- Edge cases

Run unit tests:
```bash
cargo test
```

### 2. Integration Tests

Integration tests require a local NEAR network:

1. Start local network:
```bash
near-sandbox-rs
```

2. Deploy test token contract:
```bash
near deploy --wasmFile test_token.wasm --accountId token.test.near
```

3. Run integration tests:
```bash
cargo test --features integration-tests
```

### 3. Test Coverage

Current test coverage:
- Lines: 95%
- Branches: 92%
- Functions: 100%

Generate coverage report:
```bash
cargo tarpaulin
```

## Contract Deployment

### 1. Build for Production

```bash
./build.sh  # Uses release profile with optimizations
```

### 2. Deploy Steps

1. Create contract account:
```bash
near create-account iaip-agent.near --masterAccount your-account.near
```

2. Deploy contract:
```bash
near deploy --wasmFile target/wasm32-unknown-unknown/release/iaip_agent_registration.wasm \
           --accountId iaip-agent.near \
           --initFunction new \
           --initArgs '{}'
```

3. Initialize ITLX token integration:
```bash
near call iaip-agent.near set_token_contract '{"address": "itlx.near"}' \
     --accountId owner.near
```

## Gas Usage Analysis

Key operations and their gas costs:
1. Agent Registration: ~20-25 TGas
2. Skill Indexing: ~5 TGas per skill
3. Agent Lookup: ~2-3 TGas

## Security Considerations

### 1. Access Control
- Owner-only functions for configuration
- Public registration with token requirements
- Read-only access for queries

### 2. Data Validation
- Skill name length limits
- Metadata size restrictions
- Account ID validation

### 3. Economic Security
- Minimum token requirement
- Cross-contract call safety
- Reentrancy protection

## Upgradeability

The contract supports upgrades through:
1. State migration functions
2. Version tracking
3. Backward compatibility layers

## Error Handling

Common error scenarios and their handling:
```rust
pub enum ContractError {
    AlreadyRegistered,
    InsufficientBalance,
    InvalidMetadata,
    // ...
}
```

## Event Standards

The contract follows NEAR event standards:
```json
{
    "standard": "iaip-1.0.0",
    "event": "agent_registered",
    "data": {
        "agent_id": "account.near",
        "timestamp": "1634750400000",
        "metadata": {}
    }
}
```

## Performance Optimization Tips

1. Batch Operations
```rust
// Efficient batch processing
pub fn batch_register_agents(&mut self, agents: Vec<AgentMetadata>) {
    // Implementation
}
```

2. Storage Management
- Use lazy loading for large datasets
- Implement pagination for queries
- Cache frequently accessed data

## Monitoring and Maintenance

### 1. Health Checks
- Total agents counter integrity
- Skills index consistency
- Token balance verification

### 2. Diagnostics
```bash
near view iaip-agent.near get_diagnostics
```

## Contributing Guidelines

1. Code Style
- Follow Rust formatting guidelines
- Document public interfaces
- Include unit tests

2. Pull Request Process
- Create feature branch
- Update tests
- Add documentation
- Submit PR with description

3. Review Criteria
- Test coverage
- Gas optimization
- Security considerations
- Documentation quality 