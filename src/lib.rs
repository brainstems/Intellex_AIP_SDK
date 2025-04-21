use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::store::IterableSet;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Gas, Promise, PanicOnDefault, NearToken, require};

const ITLX_TOKEN_CONTRACT: &str = "itlx.token.near"; // Replace with actual ITLX token contract
const MIN_ITLX_BALANCE: u128 = 100_000_000_000_000_000_000_000; // 100 ITLX (assuming 24 decimals)
const GAS_FOR_FT_BALANCE: Gas = Gas::from_gas(5_000_000_000_000);
const GAS_FOR_REPUTATION_CALL: Gas = Gas::from_gas(5_000_000_000_000);

// Import structs from reputation contract
use crate::reputation::{TaskResult, AgentInfo};

// Module to include reputation contract interface
mod reputation {
    use super::*;
    
    #[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct TaskResult {
        pub task_id: String,
        pub success: bool,
        pub timestamp: u64,
        pub details: String,
    }

    #[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
    #[serde(crate = "near_sdk::serde")]
    pub struct AgentInfo {
        pub reputation: u64,
        pub task_history: Vec<TaskResult>,
        pub reputation_history: Vec<(u64, u64)>, // (timestamp, reputation)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AgentMetadata {
    pub name: String,
    pub description: String,
    pub skills: Vec<String>,
    pub purpose: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Agent {
    pub owner_id: AccountId,
    pub metadata: AgentMetadata,
    pub registered_at: u64,
    pub reputation_info: AgentInfo,  // Using AgentInfo from reputation contract
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AgentRegistration {
    agents: LookupMap<AccountId, Agent>,
    skills_index: LookupMap<String, IterableSet<AccountId>>,
    total_agents: u64,
    reputation_contract_id: AccountId,
}

#[near_bindgen]
impl AgentRegistration {
    #[init]
    pub fn new(reputation_contract_id: AccountId) -> Self {
        Self {
            agents: LookupMap::new(b"a"),
            skills_index: LookupMap::new(b"s"),
            total_agents: 0,
            reputation_contract_id,
        }
    }

    pub fn register_agent(&mut self, metadata: AgentMetadata) {
        let account_id = env::predecessor_account_id();
        
        // Check if agent is already registered
        require!(
            !self.agents.contains_key(&account_id),
            "Agent already registered"
        );

        // Check ITLX token balance
        let promise = Promise::new(ITLX_TOKEN_CONTRACT.parse().unwrap())
            .function_call(
                "ft_balance_of".to_string(),
                serde_json::to_vec(&account_id).unwrap(),
                NearToken::from_yoctonear(0),
                GAS_FOR_FT_BALANCE,
            );

        // Initialize agent with default reputation info
        let agent = Agent {
            owner_id: account_id.clone(),
            metadata: metadata.clone(),
            registered_at: env::block_timestamp(),
            reputation_info: AgentInfo {
                reputation: 0,
                task_history: Vec::new(),
                reputation_history: vec![(env::block_timestamp(), 0)],
            },
        };

        self.agents.insert(&account_id, &agent);
        self.total_agents += 1;

        // Index by skills
        for skill in &metadata.skills {
            let skill_key = format!("s_{}", skill);
            let mut skill_agents = match self.skills_index.get(skill) {
                Some(existing_set) => existing_set,
                None => IterableSet::<AccountId>::new(skill_key.as_bytes().to_vec())
            };
            
            skill_agents.insert(account_id.clone());
            self.skills_index.insert(skill, &skill_agents);
        }

        // Call reputation contract to initialize agent's reputation
        Promise::new(self.reputation_contract_id.clone())
            .function_call(
                "initialize_agent".to_string(),
                serde_json::to_vec(&account_id).unwrap(),
                NearToken::from_yoctonear(0),
                GAS_FOR_REPUTATION_CALL,
            );
    }

    #[private]
    pub fn update_agent_reputation(&mut self, agent_id: AccountId, reputation_info: AgentInfo) {
        require!(
            env::predecessor_account_id() == self.reputation_contract_id,
            "Only reputation contract can update reputation"
        );

        if let Some(mut agent) = self.agents.get(&agent_id) {
            agent.reputation_info = reputation_info;
            self.agents.insert(&agent_id, &agent);
        }
    }

    pub fn get_agent(&self, agent_id: &AccountId) -> Option<Agent> {
        self.agents.get(agent_id).map(|agent| agent.clone())
    }

    pub fn get_agents_by_skill(&self, skill: &String) -> Vec<AccountId> {
        match self.skills_index.get(skill) {
            Some(skill_agents) => skill_agents.iter().cloned().collect(),
            None => Vec::new()
        }
    }

    pub fn get_total_agents(&self) -> u64 {
        self.total_agents
    }

    pub fn get_agent_skills(&self, agent_id: &AccountId) -> Option<Vec<String>> {
        self.agents
            .get(agent_id)
            .map(|agent| agent.metadata.skills.clone())
    }

    pub fn get_agent_reputation(&self, agent_id: &AccountId) -> Option<u64> {
        self.agents
            .get(agent_id)
            .map(|agent| agent.reputation_info.reputation)
    }

    pub fn get_agent_task_history(&self, agent_id: &AccountId, from_index: Option<u64>, limit: Option<u64>) -> Vec<TaskResult> {
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(50).min(100);

        self.agents
            .get(agent_id)
            .map(|agent| {
                agent.reputation_info.task_history
                    .iter()
                    .skip(from_index as usize)
                    .take(limit as usize)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_agent_reputation_history(&self, agent_id: &AccountId) -> Vec<(u64, u64)> {
        self.agents
            .get(agent_id)
            .map(|agent| agent.reputation_info.reputation_history.clone())
            .unwrap_or_default()
    }

    pub fn sync_reputation(&mut self, agent_id: AccountId) -> Promise {
        Promise::new(self.reputation_contract_id.clone())
            .function_call(
                "get_agent_info".to_string(),
                serde_json::to_vec(&agent_id).unwrap(),
                NearToken::from_yoctonear(0),
                GAS_FOR_REPUTATION_CALL,
            )
            .then(
                Promise::new(env::current_account_id())
                    .function_call(
                        "update_agent_reputation".to_string(),
                        serde_json::to_vec(&(agent_id, "")).unwrap(),
                        NearToken::from_yoctonear(0),
                        GAS_FOR_REPUTATION_CALL,
                    )
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_registration_with_reputation() {
        let reputation_contract = accounts(0);
        let agent_account = accounts(1);
        
        let context = get_context(agent_account.clone());
        testing_env!(context.build());
        
        let mut contract = AgentRegistration::new(reputation_contract.clone());
        
        let metadata = AgentMetadata {
            name: "Test Agent".to_string(),
            description: "Test Description".to_string(),
            skills: vec!["Rust".to_string()],
            purpose: "Testing".to_string(),
        };
        
        contract.register_agent(metadata);
        
        let agent = contract.get_agent(&agent_account).unwrap();
        assert_eq!(agent.reputation_info.reputation, 0);
        assert_eq!(agent.reputation_info.task_history.len(), 0);
        assert_eq!(agent.reputation_info.reputation_history.len(), 1);
    }

    #[test]
    fn test_reputation_sync() {
        let reputation_contract = accounts(0);
        let agent_account = accounts(1);
        
        let context = get_context(agent_account.clone());
        testing_env!(context.build());
        
        let mut contract = AgentRegistration::new(reputation_contract.clone());
        
        // Register agent
        contract.register_agent(AgentMetadata {
            name: "Test Agent".to_string(),
            description: "Test Description".to_string(),
            skills: vec!["Rust".to_string()],
            purpose: "Testing".to_string(),
        });
        
        // Update reputation as reputation contract
        let new_reputation_info = AgentInfo {
            reputation: 50,
            task_history: vec![TaskResult {
                task_id: "test_task".to_string(),
                success: true,
                timestamp: env::block_timestamp(),
                details: "Test task completed".to_string(),
            }],
            reputation_history: vec![(env::block_timestamp(), 50)],
        };
        
        let context = get_context(reputation_contract.clone());
        testing_env!(context.build());
        
        contract.update_agent_reputation(agent_account.clone(), new_reputation_info);
        
        let agent = contract.get_agent(&agent_account).unwrap();
        assert_eq!(agent.reputation_info.reputation, 50);
        assert_eq!(agent.reputation_info.task_history.len(), 1);
        assert_eq!(agent.reputation_info.reputation_history.len(), 1);
    }
} 