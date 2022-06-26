# p2p-chatter

A p2p chatter built upon rust-libp2p (now ONLY for test)

## Usage

You will need two or more machines within a LAN, or similarly two terminals on one machine. They must support mDNS for this version.

Each of them should run `RUST_LOG=info cargo run`

## Future works

- *Relay Mode* to enable P2P connection through NAT/firewalls (Working)

- *Human-friendly Identification* to enhance user experience

- *wasm support* to enable in-browser usage
