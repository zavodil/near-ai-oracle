use crate::completions::AgentCompletion;
use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use completions::parse_value_from_completion;
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::Deserialize;
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    bs58, env, log, near_bindgen,
    serde::Serialize,
    BorshStorageKey, PanicOnDefault, Timestamp,
};
use schemars::JsonSchema;
use serde_json::Value;
use sha2::{Digest, Sha256};

mod completions;
mod events;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct OracleItem {
    pub agent_name: String,
    pub agent_public_key: String,
    pub prompt: String,
    pub urls: Vec<String>,
    pub value: Option<String>,
    pub updated_at: Option<Timestamp>,
    // TODO: add provided_by: AccountId,
}

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    OracleItems,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    pub items: UnorderedMap<String, OracleItem>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            items: UnorderedMap::new(StorageKey::OracleItems),
        }
    }

    pub fn insert(&mut self, name: String, item: OracleItem) {
        assert!(self.items.get(&name).is_none(), "Already registered");

        self.items.insert(&name, &item);
    }

    pub fn get_value(&self, name: String) -> Option<String> {
        if let Some(item) = self.items.get(&name) {
            item.value
        } else {
            None
        }
    }

    pub fn get_agent_data(&self, name: String) -> Option<OracleItem> {
        self.items.get(&name)
    }

    pub fn add_value(&mut self, name: String, completion: AgentCompletion) {
        if let Some(mut item) = self.items.get(&name) {
            assert!(
                self.verify_oracle_item(item.clone(), completion.clone()),
                "Illegal completion"
            );

            // try to parse `value` from the completion
            let new_value = parse_value_from_completion(completion.completion);

            item.value = Some(new_value);
            item.updated_at = Some(env::block_timestamp());
            self.items.insert(&name, &item);
        }
    }

    // placeholder function, to be implemented
    pub fn run_agent(&mut self, agent: String, message: String) {
        events::emit::run_agent(&agent, &message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use completions::*;

    #[test]
    pub fn test_parse_value() {
        let completion = r#"{"result_found": true, "value": "Joe Biden"}"#;

        assert_eq!(
            parse_value_from_completion(completion.to_string()),
            "Joe Biden"
        );
    }

    #[test]
    pub fn test_verify_potus_1() {
        let items = UnorderedMap::new(StorageKey::OracleItems);
        let contract = Contract { items };

        let signature: &str = "uugEnaJEGeR4zUpKnbuOatBwcV+YFklvgGfpPTNyXt9lG5j1FWF6+jI6w21fH5fMRL2Wzv+Xlt3BxBVuZNIgBA==";

        let agent_name = "zavodil.near/potus/0.31";
        let completion = r#"{"result_found": true, "value": "Joe Biden"}"#;
        let model = "fireworks::accounts/fireworks/models/llama-v3p1-70b-instruct";
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "Given the following responses about the current President of the United States,  determine if they all refer to the same individual.\nResponse 1: Joe Biden\nResponse 2: Joe Biden\nResponse 3: Joseph Biden\n\nInstructions:\n- If names are similar, such as \"Joe Biden\" and \"Joseph Biden\", treat them as referring to the same individual.\n- If more than half agree, return the consistent response. {\"result_found\": true, \"value\": \"...\"}\n- If not, return the {\"result_found\": false}\n\nOnly respond with valid JSON. Do not include any other text.\n\nExample 1:\nResponse 1: Joseph R. Biden, Jr.\nResponse 2: Joe Biden\nResponse 3: Mr. Joe Biden\nResponse 4: Mickey Mouse\nOutput: {\"result_found\": true, \"value\": \"Joe Biden\"}\n\nExample 2:\nResponse 1: John Doe\nResponse 2: Joe Biden\nResponse 3: Donald Trump\nOutput: {\"result_found\": false}".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Always reply with valid JSON ONLY. Do not include anything else in your response".to_string(),
            },
        ];
        let temperature = Some(0.0);
        let max_tokens = Some(8192);

        contract.verify(
            "6TEw6D9qzZyftq8QNPH1LVHqZ5k66GU2BE4XJA4nkZmb".to_string(),
            signature.to_string(),
            agent_name.to_string(),
            model.to_string(),
            messages,
            temperature,
            max_tokens,
            completion.to_string(),
            Some(true),
        );
    }

    #[test]
    pub fn test_verify_potus_2() {
        let items = UnorderedMap::new(StorageKey::OracleItems);
        let contract = Contract { items };

        let signature: &str = "YYeiQqCa2K/HfGHhYscYIKQrzFuKRvIcSn64J1kfFlF4sU7UKgRuMxUYyZQDwEAITbtdTWr9igTZe9TcxvLABQ==";

        let agent_name = "zavodil.near/potus/0.34";
        let completion = "{\"result_found\": true, \"value\": \"Joe Biden\"}";
        let model = "fireworks::accounts/fireworks/models/llama-v3p1-70b-instruct";
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "Given the following responses about the current President of the United States,  determine if they all refer to the same individual.\nResponse 1: Joe Biden\nResponse 2: Joe Biden\nResponse 3: Joseph Biden\n\nInstructions:\n- If names are similar, such as \"Joe Biden\" and \"Joseph Biden\", treat them as referring to the same individual.\n- If more than half agree, return the consistent response. {\"result_found\": true, \"value\": \"...\"}\n- If not, return the {\"result_found\": false}\n\nOnly respond with valid JSON. Do not include any other text.\n\nExample 1:\nResponse 1: Joseph R. Biden, Jr.\nResponse 2: Joe Biden\nResponse 3: Mr. Joe Biden\nResponse 4: Mickey Mouse\nOutput: {\"result_found\": true, \"value\": \"Joe Biden\"}\n\nExample 2:\nResponse 1: John Doe\nResponse 2: Joe Biden\nResponse 3: Donald Trump\nOutput: {\"result_found\": false}".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Always reply with valid JSON ONLY. Do not include anything else in your response".to_string(),
            },
        ];
        let temperature = Some(0.0);
        let max_tokens = Some(8192);

        contract.verify(
            "39xDEwqX9WzyDzVGM4xaghPdLNVYHzPday7yWr96L8vW".to_string(),
            signature.to_string(),
            agent_name.to_string(),
            model.to_string(),
            messages,
            temperature,
            max_tokens,
            completion.to_string(),
            Some(true),
        );
    }
}
