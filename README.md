# RouterBase Rust Client

[RouterBase](https://routerbase.com) provides an OpenAI-compatible API gateway at `https://routerbase.com/v1`.
This crate is a small blocking Rust client for chat completions and model listing.

## Install

```toml
[dependencies]
routerbase = "0.1.0"
```

## Usage

```rust
use routerbase::{ChatCompletionRequest, Client, Message};

let client = Client::new(std::env::var("ROUTERBASE_API_KEY")?);
let response = client.chat_completion(ChatCompletionRequest::new(vec![
    Message::user("Explain RouterBase in one sentence."),
]))?;

println!("{}", response.first_text().unwrap_or_default());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Links

- [RouterBase](https://routerbase.com)
- [RouterBase docs](https://docs.routerbase.com/)
- [Chat completions docs](https://docs.routerbase.com/api-reference/chat-completions)

## License

MIT

