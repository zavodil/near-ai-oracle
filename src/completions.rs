use crate::*;
use std::fmt::Write;
#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AgentCompletion {
    signature: String,
    agent_name: String,
    model: String,
    messages: Vec<Message>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    pub(crate) completion: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[borsh(crate = "near_sdk::borsh")]
pub struct CompletionSignaturePayload {
    pub agent_name: String,
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<u32>,
    pub max_tokens: Option<u32>,
    pub completion: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[near_bindgen]
impl Contract {
    pub fn verify_oracle_item(&self, item: OracleItem, completion: AgentCompletion) -> bool {
        assert_eq!(item.agent_name, completion.agent_name, "Illegal Agent");

        self.verify(
            item.agent_public_key,
            completion.signature,
            completion.agent_name,
            completion.model,
            completion.messages,
            completion.temperature,
            completion.max_tokens,
            completion.completion,
            Some(true),
        )
    }

    pub fn verify(
        &self,
        public_key: String,
        signature: String,
        agent_name: String,
        model: String,
        messages: Vec<Message>,
        temperature: Option<f64>,
        max_tokens: Option<u32>,
        completion: String,
        verbose: Option<bool>,
    ) -> bool {
        let verbose = verbose.unwrap_or(false);

        // Create the payload structure using the provided parameters such as agent name,
        // completion text, model, messages, temperature, and max tokens.
        // This payload will later be serialized and used for signature verification.
        let payload: &CompletionSignaturePayload = &create_payload(
            agent_name.as_str(),
            completion.as_str(),
            model.as_str(),
            messages,
            temperature,
            max_tokens,
        );

        // 1. Serialize the payload using Borsh
        let mut borsh_payload = Vec::new();
        payload.serialize(&mut borsh_payload).unwrap();

        log!("payload {:?}", payload);

        if verbose {
            let message = base64.encode(borsh_payload.clone());
            log!("Base64 payload: {}", message);
        }

        // 2. Compute the SHA-256 hash of the Borsh-serialized data
        let mut hasher = Sha256::new();
        hasher.update(&borsh_payload);
        let to_sign = hasher.finalize();
        if verbose {
            log!("Message to sign: {:?}", to_sign);
        }

        // 3. Decode the public key from Base58
        let pk_bytes = bs58::decode(&public_key).into_vec().unwrap();
        if pk_bytes.len() != 32 {
            panic!("Invalid public key length");
        }
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&pk_bytes);

        // 4. Decode the signature from Base64
        let sig_bytes = base64.decode(signature).unwrap();
        if sig_bytes.len() != 64 {
            panic!("Signature check failed");
        }
        let mut sig = [0u8; 64];
        sig.copy_from_slice(&sig_bytes);

        if verbose {
            log!("Signature: {:?}", sig);
            log!("Public Key: {:?}", pk);
            let sig_bytes = sig.iter().fold(String::new(), |mut acc, &byte| {
                write!(&mut acc, "\\x{:02x}", byte).unwrap();
                acc
            });
            log!("Signature bytes: b\"{}\"", sig_bytes);
        }

        // 5. Validate the signature using `near_sdk::env::ed25519_verify`
        let verification = env::ed25519_verify(&sig, &to_sign, &pk);

        if verbose {
            log!("Verification: {:?}", verification);
        }

        assert!(verification, "Signature check failed");

        verification
    }
}

fn create_payload(
    agent_name: &str,
    completion: &str,
    model: &str,
    messages: Vec<Message>,
    temperature: Option<f64>,
    max_tokens: Option<u32>,
) -> CompletionSignaturePayload {
    // Convert temperature to u32
    let temperature = temperature.map(|temp| (temp * 1000.0).round() as u32);

    CompletionSignaturePayload {
        agent_name: agent_name.to_string(),
        model: model.to_string(),
        messages,
        temperature,
        max_tokens,
        completion: completion.to_string(),
    }
}

pub fn parse_value_from_completion(completion: String) -> String {
    serde_json::from_str::<Value>(completion.as_str())
        .ok() // Converts the Result to Option. If parsing is successful, it returns Some(json), otherwise None.
        .and_then(|json| {
            json.get("value")
                .map(|v| v.as_str().unwrap_or("").to_string())
        }) // Get the "value" field as a string
        .unwrap_or_else(|| completion) // If there's an error or "value" key is missing, return the original string.
}
