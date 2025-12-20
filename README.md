# OpenDroneID Rust Library

A Rust implementation of the ASTM F3411 OpenDroneID standard for drone remote identification.

## Overview

This library provides encoding and decoding functionality for OpenDroneID messages, which are used for drone remote identification and tracking. The implementation follows the ASTM F3411 standard and supports all 6 message types.

## Features

- ✅ **Complete ASTM F3411 Implementation**: Supports all 6 message types
- ✅ **Message Pack Support**: Container for multiple messages in single transmission
- ✅ **Bit-Level Encoding**: Efficient bit-packing for minimal message size
- ✅ **Type Safety**: Strong typing with Rust enums and structs
- ✅ **Comprehensive Error Handling**: Clear error messages for invalid data
- ✅ **CLI Tool**: Command-line interface for easy encoding/decoding
- ✅ **Full Test Coverage**: Unit tests for all message types and message packs

## Message Types Supported

1. **Basic ID**: Drone identification information
2. **Location**: GPS coordinates and flight data
3. **Auth**: Authentication data
4. **Self ID**: Text description of the drone
5. **System**: Operator and operational area information
6. **Operator ID**: Operator identification

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
opendroneid = { git = "https://github.com/your-repo/opendroneid-lib.git" }
```

## Usage

### Basic Example

```rust
use opendroneid::{OpenDroneIdMessage, BasicIdMessage};

// Create a Basic ID message
let message = OpenDroneIdMessage::BasicId(BasicIdMessage {
    uas_id_type: 1,
    uas_id: "DRONE123".to_string(),
});

// Encode to bytes
let encoded = message.encode()?;
println!("Encoded: {:?}", encoded);

// Decode back to message
let decoded = OpenDroneIdMessage::decode(&encoded)?;
println!("Decoded: {:?}", decoded);
```

### Message Pack Example

```rust
use opendroneid::{MessagePack, OpenDroneIdMessage, BasicIdMessage, LocationMessage};

// Create multiple messages
let basic_id = OpenDroneIdMessage::BasicId(BasicIdMessage {
    uas_id_type: 1,
    uas_id: "DRONE123".to_string(),
});

let location = OpenDroneIdMessage::Location(LocationMessage {
    status: 2,
    direction: 45,
    speed: 10,
    latitude: 123456789,
    longitude: -987654321,
    altitude: 500,
    height: 100,
});

// Create a message pack
let message_pack = MessagePack::new(vec![basic_id, location]);

// Encode the message pack
let encoded = message_pack.encode()?;
println!("Encoded pack: {:?}", encoded);

// Decode the message pack
let decoded_pack = MessagePack::decode(&encoded)?;
println!("Decoded pack with {} messages", decoded_pack.message_count());
```

### CLI Usage

The library includes a CLI tool for easy encoding/decoding:

```bash
# Navigate to CLI directory
cd opendroneid-cli

# Build and run
cargo run --help
```

#### Encode a message

```bash
# Encode Basic ID
cargo run -- encode basic-id --uas-id-type 1 --uas-id "DRONE123"

# Encode Location
cargo run -- encode location --status 2 --direction 45 --speed 10 \
  --latitude 123456789 --longitude -987654321 --altitude 500 --height 100
```

#### Decode a message

```bash
# Decode from hex string
cargo run -- decode --hex "05088a49e9c8a6264660"

# Decode from file
cargo run -- decode message.bin
```

## API Documentation

### Message Pack

```rust
pub struct MessagePack {
    pub version: u8,
    pub messages: Vec<OpenDroneIdMessage>,
}

impl MessagePack {
    /// Create a new Message Pack with the current version
    pub fn new(messages: Vec<OpenDroneIdMessage>) -> Self;
    
    /// Get the number of messages in the pack
    pub fn message_count(&self) -> usize;
    
    /// Check if the pack is empty
    pub fn is_empty(&self) -> bool;
    
    /// Encode the message pack into bytes
    pub fn encode(&self) -> Result<Vec<u8>, OpenDroneIdError>;
    
    /// Decode bytes into a Message Pack
    pub fn decode(bytes: &[u8]) -> Result<MessagePack, OpenDroneIdError>;
}
```

### Message Types

```rust
pub enum OpenDroneIdMessage {
    BasicId(BasicIdMessage),
    Location(LocationMessage),
    Auth(AuthMessage),
    SelfId(SelfIdMessage),
    System(SystemMessage),
    OperatorId(OperatorIdMessage),
}
```

### Basic ID Message

```rust
pub struct BasicIdMessage {
    pub uas_id_type: u8,      // 3 bits (0-7)
    pub uas_id: String,       // Variable length, max 20 bytes
}
```

### Location Message

```rust
pub struct LocationMessage {
    pub status: u8,           // 3 bits (0-7)
    pub direction: u16,       // 11 bits (0-2047)
    pub speed: u16,           // 10 bits (0-1023)
    pub latitude: i32,        // 32 bits
    pub longitude: i32,       // 32 bits
    pub altitude: i16,        // 16 bits
    pub height: i16,          // 16 bits
}
```

### Main Functions

```rust
impl OpenDroneIdMessage {
    /// Get the message type
    pub fn message_type(&self) -> MessageType;
    
    /// Encode message to bytes
    pub fn encode(&self) -> Result<Vec<u8>, OpenDroneIdError>;
    
    /// Decode bytes to message
    pub fn decode(bytes: &[u8]) -> Result<OpenDroneIdMessage, OpenDroneIdError>;
}
```

## Error Handling

```rust
pub enum OpenDroneIdError {
    InvalidMessageType,
    InvalidMessageLength,
    EncodingError(String),
    DecodingError(String),
}
```

## Technical Details

### Message Pack Format

The Message Pack format follows ASTM F3411 specification:

- **Version**: 3 bits (currently version 0)
- **Message Count**: 4 bits (supports up to 15 messages per pack)
- **Messages**: Concatenated encoded messages
- **Maximum Size**: 15 messages per pack

### Bit Packing

The library uses efficient bit-level encoding:

- **Message Type**: 3 bits
- **Fields**: Packed according to ASTM F3411 specification
- **Variable Length**: Length prefixes for variable-length fields
- **MSB First**: Most significant bit first ordering

### Performance

- **Minimal Memory**: Bit-packing reduces message size
- **Fast Processing**: Optimized bit manipulation
- **Zero Dependencies**: Pure Rust implementation

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build --release
```

### CLI Development

```bash
cd opendroneid-cli
cargo build --release
```

## Contributing

Contributions are welcome! Please open issues or pull requests for:

- Bug fixes
- Performance improvements
- Additional features
- Documentation enhancements

## License

This project is licensed under the MIT License.

## Standards Compliance

This implementation follows the **ASTM F3411** standard for Remote ID and Tracking of Unmanned Aircraft Systems.

## Contact

For questions or support, please open a GitHub issue.

---

*Built with ❤️ for the drone community* 🚁