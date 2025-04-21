use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::{testing_env, AccountId};
use near_sdk::env;
use near_sdk::test_utils::test_env::{alice, bob};
use near_sdk::store::IterableSet;
use near_sdk::collections::LookupMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, PanicOnDefault, require};

mod agent_registration {
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
        pub reputation_history: Vec<(u64, u64)>,
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
        pub reputation_info: AgentInfo,
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
    }
}

fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(accounts(0))
        .signer_account_id(predecessor_account_id.clone())
        .predecessor_account_id(predecessor_account_id)
        .block_height(1)
        .block_timestamp(1_600_000_000_000_000);
    builder
}

#[test]
fn test_register_agent_with_reputation() {
    let reputation_contract = accounts(0);
    let agent_account = accounts(1);
    
    let context = get_context(agent_account.clone());
    testing_env!(context.build());
    
    let mut contract = agent_registration::AgentRegistration::new(reputation_contract.clone());
    
    // Register an agent
    contract.register_agent(agent_registration::AgentMetadata {
        name: "Test Agent".to_string(),
        description: "Test description".to_string(),
        skills: vec!["Rust".to_string(), "Smart Contracts".to_string()],
        purpose: "Test purpose".to_string(),
    });
    
    // Verify agent was registered with default reputation
    let agent = contract.get_agent(&agent_account).unwrap();
    assert_eq!(agent.metadata.name, "Test Agent");
    assert_eq!(agent.reputation_info.reputation, 0);
    assert_eq!(agent.reputation_info.task_history.len(), 0);
    assert_eq!(agent.reputation_info.reputation_history.len(), 1);
}

#[test]
fn test_update_agent_reputation() {
    let reputation_contract = accounts(0);
    let agent_account = accounts(1);
    
    // Register agent
    let context = get_context(agent_account.clone());
    testing_env!(context.build());
    
    let mut contract = agent_registration::AgentRegistration::new(reputation_contract.clone());
    
    contract.register_agent(agent_registration::AgentMetadata {
        name: "Test Agent".to_string(),
        description: "Test description".to_string(),
        skills: vec!["Rust".to_string()],
        purpose: "Test purpose".to_string(),
    });
    
    // Update reputation as reputation contract
    let context = get_context(reputation_contract.clone());
    testing_env!(context.build());
    
    let new_reputation_info = agent_registration::AgentInfo {
        reputation: 50,
        task_history: vec![agent_registration::TaskResult {
            task_id: "test_task".to_string(),
            success: true,
            timestamp: env::block_timestamp(),
            details: "Test task completed".to_string(),
        }],
        reputation_history: vec![(env::block_timestamp(), 50)],
    };
    
    contract.update_agent_reputation(agent_account.clone(), new_reputation_info);
    
    // Verify reputation update
    let agent = contract.get_agent(&agent_account).unwrap();
    assert_eq!(agent.reputation_info.reputation, 50);
    assert_eq!(agent.reputation_info.task_history.len(), 1);
    assert_eq!(agent.reputation_info.reputation_history.len(), 1);
}

#[test]
#[should_panic(expected = "Only reputation contract can update reputation")]
fn test_unauthorized_reputation_update() {
    let reputation_contract = accounts(0);
    let agent_account = accounts(1);
    let unauthorized = accounts(2);
    
    // Register agent
    let context = get_context(agent_account.clone());
    testing_env!(context.build());
    
    let mut contract = agent_registration::AgentRegistration::new(reputation_contract.clone());
    
    contract.register_agent(agent_registration::AgentMetadata {
        name: "Test Agent".to_string(),
        description: "Test description".to_string(),
        skills: vec!["Rust".to_string()],
        purpose: "Test purpose".to_string(),
    });
    
    // Try to update reputation as unauthorized account
    let context = get_context(unauthorized);
    testing_env!(context.build());
    
    let new_reputation_info = agent_registration::AgentInfo {
        reputation: 50,
        task_history: vec![],
        reputation_history: vec![],
    };
    
    contract.update_agent_reputation(agent_account, new_reputation_info);
}

#[test]
fn test_get_agent_reputation_history() {
    let reputation_contract = accounts(0);
    let agent_account = accounts(1);
    
    // Register agent
    let context = get_context(agent_account.clone());
    testing_env!(context.build());
    
    let mut contract = agent_registration::AgentRegistration::new(reputation_contract.clone());
    
    contract.register_agent(agent_registration::AgentMetadata {
        name: "Test Agent".to_string(),
        description: "Test description".to_string(),
        skills: vec!["Rust".to_string()],
        purpose: "Test purpose".to_string(),
    });
    
    // Update reputation multiple times
    let context = get_context(reputation_contract.clone());
    testing_env!(context.build());
    
    let timestamps = vec![
        env::block_timestamp(),
        env::block_timestamp() + 1000,
        env::block_timestamp() + 2000,
    ];
    
    let mut accumulated_history = vec![(env::block_timestamp(), 0)]; // Initial reputation
    
    for (i, timestamp) in timestamps.iter().enumerate() {
        let reputation = (i as u64 + 1) * 10;
        accumulated_history.push((*timestamp, reputation));
        
        let new_reputation_info = agent_registration::AgentInfo {
            reputation,
            task_history: vec![],
            reputation_history: accumulated_history.clone(),
        };
        
        contract.update_agent_reputation(agent_account.clone(), new_reputation_info);
    }
    
    // Verify reputation history
    let history = contract.get_agent_reputation_history(&agent_account);
    assert_eq!(history.len(), 4); // Initial registration + 3 updates
    assert_eq!(history[0].1, 0); // Initial reputation
    assert_eq!(history[1].1, 10);
    assert_eq!(history[2].1, 20);
    assert_eq!(history[3].1, 30);
}

#[test]
fn test_get_agent_task_history() {
    let reputation_contract = accounts(0);
    let agent_account = accounts(1);
    
    // Register agent
    let context = get_context(agent_account.clone());
    testing_env!(context.build());
    
    let mut contract = agent_registration::AgentRegistration::new(reputation_contract.clone());
    
    contract.register_agent(agent_registration::AgentMetadata {
        name: "Test Agent".to_string(),
        description: "Test description".to_string(),
        skills: vec!["Rust".to_string()],
        purpose: "Test purpose".to_string(),
    });
    
    // Add multiple tasks
    let context = get_context(reputation_contract.clone());
    testing_env!(context.build());
    
    let tasks = vec![
        agent_registration::TaskResult {
            task_id: "task1".to_string(),
            success: true,
            timestamp: env::block_timestamp(),
            details: "First task".to_string(),
        },
        agent_registration::TaskResult {
            task_id: "task2".to_string(),
            success: false,
            timestamp: env::block_timestamp() + 1000,
            details: "Second task".to_string(),
        },
    ];
    
    let new_reputation_info = agent_registration::AgentInfo {
        reputation: 50,
        task_history: tasks.clone(),
        reputation_history: vec![],
    };
    
    contract.update_agent_reputation(agent_account.clone(), new_reputation_info);
    
    // Verify task history
    let history = contract.get_agent_task_history(&agent_account, None, None);
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].task_id, "task1");
    assert_eq!(history[1].task_id, "task2");
    assert!(history[0].success);
    assert!(!history[1].success);
}

#[test]
fn test_get_agent_task_history_pagination() {
    let reputation_contract = accounts(0);
    let agent_account = accounts(1);
    
    // Register agent
    let context = get_context(agent_account.clone());
    testing_env!(context.build());
    
    let mut contract = agent_registration::AgentRegistration::new(reputation_contract.clone());
    
    contract.register_agent(agent_registration::AgentMetadata {
        name: "Test Agent".to_string(),
        description: "Test description".to_string(),
        skills: vec!["Rust".to_string()],
        purpose: "Test purpose".to_string(),
    });
    
    // Add multiple tasks
    let context = get_context(reputation_contract.clone());
    testing_env!(context.build());
    
    let mut tasks = Vec::new();
    for i in 0..10 {
        tasks.push(agent_registration::TaskResult {
            task_id: format!("task{}", i),
            success: true,
            timestamp: env::block_timestamp() + (i as u64 * 1000),
            details: format!("Task {}", i),
        });
    }
    
    let new_reputation_info = agent_registration::AgentInfo {
        reputation: 50,
        task_history: tasks,
        reputation_history: vec![],
    };
    
    contract.update_agent_reputation(agent_account.clone(), new_reputation_info);
    
    // Test pagination
    let history = contract.get_agent_task_history(&agent_account, Some(5), Some(3));
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].task_id, "task5");
    assert_eq!(history[1].task_id, "task6");
    assert_eq!(history[2].task_id, "task7");
} 