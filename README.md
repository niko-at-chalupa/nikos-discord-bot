# Niko's Discord Bot

Fun discord bot written in Rust. Made for [Nikolandia](https://discord.gg/rtFfPaYQFP), and I don't see this being used anywhere else.

> [!NOTE]
> This is *only* on GitHub for transparency. While you *could* use this for your own server, it's not a good idea to do so.

## Setup

Setup is easy using `run.sh`. Make a new `.env` file and fill in the following:

```shell
TOKEN=your-discord-bot-token
RULE34_API_KEY=(optional)-your-rule34.xxx-api-key
RULE34_USER_ID=(optional)-your-rule34.xxx-user-id
```

Install the following dependencies:
- Python
- Rust
- Cargo

After writing a `.env`, run the bot by running the following in shell:
```shell
chmod +x run.sh && ./run.sh
```

<details><summary>Manual</summary>

Set the following environment variables:
- **TOKEN** - your Discord bot's token
- **RULE34_API_KEY** (optional) - An API key towards rule34.xxx
- **RULE34_USER_ID** (optional) - A user ID that aligns with the API key (for rule34.xxx)

Leaving any optional environment variable unset will make the bot not explicitly register commands. Unregistered commands can be identified through a cross appearing before its name in logs.


Then, run the bot through
```shell
cargo run --release
```
or
```shell
cargo build --release && ./target/release/nikos-discord-bot
```

</details>