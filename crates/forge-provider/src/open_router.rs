use super::error::Result;
use super::open_ai::Role; // Importing Role
use super::provider::{InnerProvider, Provider};
use futures::stream::Stream as FuturesStream;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize}; // Importing Stream trait

#[derive(Debug, Clone)]
struct OpenRouterConfig {
    api_key: String,
    base_url: Option<String>,
}

impl OpenRouterConfig {
    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn api_base(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://openrouter.ai/api/v1")
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );
        headers.insert("X-Title", HeaderValue::from_static("Tailcall"));
        headers
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_base(), path)
    }

    fn query(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }
}

#[derive(Clone)]
pub struct OpenRouter {
    http_client: reqwest::Client,
    config: OpenRouterConfig,
    model: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub created: u64,
    pub description: String,
    pub context_length: u64,
    pub architecture: Architecture,
    pub pricing: Pricing,
    pub top_provider: TopProvider,
    pub per_request_limits: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Architecture {
    pub modality: String,
    pub tokenizer: String,
    pub instruct_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Pricing {
    pub prompt: String,
    pub completion: String,
    pub image: String,
    pub request: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TopProvider {
    pub context_length: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub is_moderated: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct ListModelResponse {
    pub object: Option<String>,
    pub data: Vec<Model>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Request {
    pub messages: Option<Vec<Message>>,
    pub prompt: Option<String>,
    pub model: Option<String>,
    pub response_format: Option<ResponseFormat>,
    pub stop: Option<Vec<String>>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub seed: Option<u32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub repetition_penalty: Option<f32>,
    pub logit_bias: Option<std::collections::HashMap<u32, f32>>,
    pub top_logprobs: Option<u32>,
    pub min_p: Option<f32>,
    pub top_a: Option<f32>,
    pub prediction: Option<Prediction>,
    pub transforms: Option<Vec<String>>,
    pub models: Option<Vec<String>>,
    pub route: Option<String>,
    pub provider: Option<ProviderPreferences>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TextContent {
    pub r#type: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageContentPart {
    pub r#type: String,
    pub image_url: ImageUrl,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageUrl {
    pub url: String,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ContentPart {
    Text(TextContent),
    Image(ImageContentPart),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: ContentPart,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionDescription {
    pub description: Option<String>,
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tool {
    pub r#type: String,
    pub function: FunctionDescription,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ToolChoice {
    None,
    Auto,
    Function { name: String },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseFormat {
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Prediction {
    pub r#type: String,
    pub content: String,
}

// New ResponseType struct based on expected API response
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseType {
    pub status: String,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

// Defining ProviderPreferences struct
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProviderPreferences {
    // Define fields as necessary
}

impl OpenRouter {
    fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        let config = OpenRouterConfig { api_key, base_url };
        let http_client = reqwest::Client::new();

        Self {
            http_client,
            config,
            model: model.unwrap_or("openai/gpt-4o-mini".to_string()),
        }
    }

    fn new_message(&self, role: Role, input: &str) -> Message {
        Message {
            role: role.to_string(),
            content: ContentPart::Text(TextContent {
                r#type: "text".to_string(),
                text: input.to_string(),
            }),
            name: None,
        }
    }

    fn prompt_request(&self, input: String) -> Result<Request> {
        Ok(Request {
            model: Some(self.model.clone()),
            messages: Some(vec![self.new_message(Role::User, &input)]),
            temperature: Some(0.7),
            stream: Some(true),
            ..Default::default()
        })
    }
}

#[async_trait::async_trait]
impl InnerProvider for OpenRouter {
    fn name(&self) -> &'static str {
        "Open Router"
    }

    async fn prompt(
        &self,
        input: String,
    ) -> Result<Box<dyn FuturesStream<Item = Result<String>> + Unpin>> {
        let request = self.prompt_request(input)?;
        let response = self
            .http_client
            .post(self.config.url("/chat/completions"))
            .headers(self.config.headers())
            .json(&request)
            .send()
            .await?
            .json::<ResponseType>() // Adjusted to use ResponseType
            .await?;

        // Handle the response and return a stream
        let stream = futures::stream::iter(
            response.data.into_iter().map(|data| Ok(data.to_string())), // Adjusted to match expected output
        );

        Ok(Box::new(stream))
    }

    async fn models(&self) -> Result<Vec<String>> {
        Ok(self
            .http_client
            .get(self.config.url("/models"))
            .headers(self.config.headers())
            .send()
            .await?
            .json::<ListModelResponse>()
            .await?
            .data
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>())
    }
}

impl Provider {
    pub fn open_router(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        Provider::new(OpenRouter::new(api_key, model, base_url))
    }
}
