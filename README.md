# hashserve
a simple networked hashmap that operates over websockets

## 🚨PROBABLY NOT SECURE🚨
I need to implement some sort of obsfucation for requests since nothing is encrypted and I do not really have plans to implement taht.

## Run it yourself
### MSRV: `1.74.1` (based on the analysis of [cargo-msrv](https://github.com/foresterre/cargo-msrv))

### Server
1. Clone the repository
2. Run `cargo run --release -- -h` to see the help message
3. Run `cargo run --release -- -p <password>` to start the server

(or you could just `cargo install --path .`, but that's up to you)

### Client
Any websocket client should work, if you need an example checkout `/bench` for an example in TypeScript

Connect to the server at `ws://<server-ip>:<port>?password=<password>`

## Usage
### Commands
- `ADD <key> <value>`: Adds a key-value pair to the hashmap
- `GET <key>`: Gets the value of a key
- `DEL <key>`: Deletes a key-value pair