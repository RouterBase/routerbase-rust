use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{error::Error, fmt};

pub const DEFAULT_BASE_URL: &str = "https://routerbase.com/v1";
pub const DEFAULT_MODEL: &str = "google/gemini-2.5-flash";

#[derive(Debug, Clone)]
pub struct Client {
    api_key: String,
    base_url: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            agent: ureq::Agent::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, RouterBaseError> {
        self.post_json("/chat/completions", &request)
    }

    pub fn models(&self) -> Result<ModelsResponse, RouterBaseError> {
        self.get_json("/models")
    }

    fn get_json<T>(&self, path: &str) -> Result<T, RouterBaseError>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .agent
            .get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .call()
            .map_err(RouterBaseError::from_ureq)?;

        decode_response(response)
    }

    fn post_json<T, B>(&self, path: &str, body: &B) -> Result<T, RouterBaseError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = format!("{}{}", self.base_url, path);
        let body = serde_json::to_string(body)?;
        let response = self
            .agent
            .post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(RouterBaseError::from_ureq)?;

        decode_response(response)
    }
}

fn decode_response<T>(response: ureq::Response) -> Result<T, RouterBaseError>
where
    T: DeserializeOwned,
{
    let body = response
        .into_string()
        .map_err(|error| RouterBaseError::Transport {
            message: error.to_string(),
        })?;

    Ok(serde_json::from_str(&body)?)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
}

impl ChatCompletionRequest {
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            model: DEFAULT_MODEL.to_string(),
            messages,
            temperature: None,
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ChatCompletionResponse {
    pub id: Option<String>,
    pub choices: Vec<Choice>,
}

impl ChatCompletionResponse {
    pub fn first_text(&self) -> Option<&str> {
        self.choices
            .first()
            .map(|choice| choice.message.content.as_str())
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Choice {
    pub message: Message,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ModelsResponse {
    pub data: Option<Vec<Model>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Model {
    pub id: String,
}

#[derive(Debug)]
pub enum RouterBaseError {
    Http { status: u16, body: String },
    Json(serde_json::Error),
    Transport { message: String },
}

impl RouterBaseError {
    fn from_ureq(error: ureq::Error) -> Self {
        match error {
            ureq::Error::Status(status, response) => {
                let body = response.into_string().unwrap_or_default();
                Self::Http { status, body }
            }
            ureq::Error::Transport(error) => Self::Transport {
                message: error.to_string(),
            },
        }
    }
}

impl fmt::Display for RouterBaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http { status, body } => {
                write!(formatter, "RouterBase request failed ({status}): {body}")
            }
            Self::Json(error) => write!(formatter, "RouterBase JSON error: {error}"),
            Self::Transport { message } => {
                write!(formatter, "RouterBase transport error: {message}")
            }
        }
    }
}
impl Error for RouterBaseError {}

impl From<serde_json::Error> for RouterBaseError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_chat_request() {
        let request = ChatCompletionRequest::new(vec![Message::user("hello")])
            .model("openai/gpt-4.1-mini")
            .temperature(0.2);

        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["model"], "openai/gpt-4.1-mini");
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"], "hello");
        assert_eq!(json["temperature"], 0.2);
    }

    #[test]
    fn reads_first_text() {
        let response = ChatCompletionResponse {
            id: Some("chatcmpl-test".to_string()),
            choices: vec![Choice {
                message: Message::assistant("hello from RouterBase"),
            }],
        };

        assert_eq!(response.first_text(), Some("hello from RouterBase"));
    }
}
