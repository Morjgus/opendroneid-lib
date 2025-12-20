// OpenDroneID message types and encoding/decoding

pub use thiserror::Error;

#[cfg(test)]
mod tests;

/// Message Pack version according to ASTM F3411 specification
const MESSAGE_PACK_VERSION: u8 = 0;

#[derive(Debug, Error)]
pub enum OpenDroneIdError {
    #[error("Invalid message type")]
    InvalidMessageType,
    #[error("Invalid message length")]
    InvalidMessageLength,
    #[error("Encoding error: {0}")]
    EncodingError(String),
    #[error("Decoding error: {0}")]
    DecodingError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    BasicId,
    Location,
    Auth,
    SelfId,
    System,
    OperatorId,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageType::BasicId,
            1 => MessageType::Location,
            2 => MessageType::Auth,
            3 => MessageType::SelfId,
            4 => MessageType::System,
            5 => MessageType::OperatorId,
            _ => panic!("Invalid message type value: {}", value),
        }
    }
}

impl From<MessageType> for u8 {
    fn from(msg_type: MessageType) -> Self {
        match msg_type {
            MessageType::BasicId => 0,
            MessageType::Location => 1,
            MessageType::Auth => 2,
            MessageType::SelfId => 3,
            MessageType::System => 4,
            MessageType::OperatorId => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasicIdMessage {
    pub uas_id_type: u8,
    pub uas_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocationMessage {
    pub status: u8,
    pub direction: u16,
    pub speed: u16,
    pub latitude: i32,
    pub longitude: i32,
    pub altitude: i16,
    pub height: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthMessage {
    pub auth_type: u8,
    pub auth_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelfIdMessage {
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemMessage {
    pub operator_location_type: u8,
    pub classification_type: u8,
    pub operator_latitude: i32,
    pub operator_longitude: i32,
    pub area_count: u8,
    pub area_radius: u16,
    pub area_ceiling: i16,
    pub area_floor: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorIdMessage {
    pub operator_id_type: u8,
    pub operator_id: String,
}

/// Message Pack structure for containing multiple OpenDroneID messages
#[derive(Debug, Clone, PartialEq)]
pub struct MessagePack {
    pub version: u8,
    pub messages: Vec<OpenDroneIdMessage>,
}

impl MessagePack {
    /// Create a new Message Pack with the current version
    pub fn new(messages: Vec<OpenDroneIdMessage>) -> Self {
        Self {
            version: MESSAGE_PACK_VERSION,
            messages,
        }
    }

    /// Get the number of messages in the pack
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if the pack is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Encode the message pack into bytes according to OpenDroneID specification
    pub fn encode(&self) -> Result<Vec<u8>, OpenDroneIdError> {
        if self.messages.is_empty() {
            return Err(OpenDroneIdError::EncodingError(
                "Message pack cannot be empty".to_string()
            ));
        }

        if self.messages.len() > 15 {
            return Err(OpenDroneIdError::EncodingError(
                "Message pack can contain maximum 15 messages".to_string()
            ));
        }

        let mut bit_buffer = Vec::new();
        
        // Helper function to add bits to buffer (MSB first)
        fn add_bits(buffer: &mut Vec<bool>, value: u64, num_bits: usize) {
            for i in (0..num_bits).rev() {
                buffer.push((value >> i) & 1 == 1);
            }
        }
        
        // Add version (3 bits)
        add_bits(&mut bit_buffer, self.version as u64, 3);
        
        // Add message count (4 bits - supports up to 15 messages)
        add_bits(&mut bit_buffer, self.messages.len() as u64, 4);
        
        // Encode each message
        for message in &self.messages {
            let encoded_message = message.encode()?;
            // Convert encoded bytes to bits and add to buffer
            for &byte in &encoded_message {
                for i in (0..8).rev() {
                    bit_buffer.push((byte >> i) & 1 == 1);
                }
            }
        }
        
        // Convert bit buffer to bytes
        let byte_len = (bit_buffer.len() + 7) / 8;
        let mut bytes = vec![0u8; byte_len];
        for (i, &bit) in bit_buffer.iter().enumerate() {
            if bit {
                let byte_idx = i / 8;
                let bit_idx = 7 - (i % 8); // MSB first within byte
                bytes[byte_idx] |= 1 << bit_idx;
            }
        }
        
        Ok(bytes)
    }

    /// Decode bytes into a Message Pack
    pub fn decode(bytes: &[u8]) -> Result<MessagePack, OpenDroneIdError> {
        if bytes.is_empty() {
            return Err(OpenDroneIdError::DecodingError(
                "Empty input bytes".to_string()
            ));
        }

        // Convert bytes to bit buffer (MSB first)
        let mut bit_buffer = Vec::new();
        for &byte in bytes {
            for i in (0..8).rev() {
                bit_buffer.push((byte >> i) & 1 == 1);
            }
        }

        let mut bit_pos = 0;
        
        // Helper function to read bits from buffer
        fn read_bits(buffer: &[bool], start: usize, num_bits: usize) -> Result<u64, OpenDroneIdError> {
            if start + num_bits > buffer.len() {
                return Err(OpenDroneIdError::DecodingError(
                    format!("Incomplete message pack - need {} bits but only have {}", num_bits, buffer.len() - start)
                ));
            }
            
            let mut value = 0u64;
            for i in 0..num_bits {
                if buffer[start + i] {
                    value |= 1 << (num_bits - 1 - i);
                }
            }
            Ok(value)
        }
        
        // Read version (3 bits)
        let version = read_bits(&bit_buffer, bit_pos, 3)? as u8;
        bit_pos += 3;
        
        if version != MESSAGE_PACK_VERSION {
            return Err(OpenDroneIdError::DecodingError(
                format!("Unsupported message pack version: {}", version)
            ));
        }
        
        // Read message count (4 bits)
        let message_count = read_bits(&bit_buffer, bit_pos, 4)? as usize;
        bit_pos += 4;
        
        if message_count == 0 {
            return Err(OpenDroneIdError::DecodingError(
                "Message pack cannot contain zero messages".to_string()
            ));
        }
        
        if message_count > 15 {
            return Err(OpenDroneIdError::DecodingError(
                format!("Invalid message count: {}", message_count)
            ));
        }
        
        // Decode each message
        let mut messages = Vec::new();
        for _ in 0..message_count {
            // We need to decode messages one by one, but we don't know their boundaries
            // So we'll use a different approach - convert remaining bits to bytes and decode
            
            // Calculate remaining bits
            let remaining_bits = bit_buffer.len() - bit_pos;
            if remaining_bits == 0 {
                return Err(OpenDroneIdError::DecodingError(
                    "Incomplete message pack - missing message data".to_string()
                ));
            }
            
            // Convert remaining bits to bytes (we'll try to decode the first message)
            let remaining_byte_len = (remaining_bits + 7) / 8;
            let mut remaining_bytes = vec![0u8; remaining_byte_len];
            
            for (i, &bit) in bit_buffer.iter().skip(bit_pos).enumerate() {
                if bit {
                    let byte_idx = i / 8;
                    let bit_idx = 7 - (i % 8); // MSB first within byte
                    remaining_bytes[byte_idx] |= 1 << bit_idx;
                }
            }
            
            // Try to decode the first message
            let message = OpenDroneIdMessage::decode(&remaining_bytes)?;
            
            // Calculate how many bits this message consumed
            let encoded_message = message.encode()?;
            let message_bits = encoded_message.len() * 8;
            
            messages.push(message);
            bit_pos += message_bits;
        }
        
        Ok(MessagePack {
            version,
            messages,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpenDroneIdMessage {
    BasicId(BasicIdMessage),
    Location(LocationMessage),
    Auth(AuthMessage),
    SelfId(SelfIdMessage),
    System(SystemMessage),
    OperatorId(OperatorIdMessage),
}

impl OpenDroneIdMessage {
    pub fn message_type(&self) -> MessageType {
        match self {
            OpenDroneIdMessage::BasicId(_) => MessageType::BasicId,
            OpenDroneIdMessage::Location(_) => MessageType::Location,
            OpenDroneIdMessage::Auth(_) => MessageType::Auth,
            OpenDroneIdMessage::SelfId(_) => MessageType::SelfId,
            OpenDroneIdMessage::System(_) => MessageType::System,
            OpenDroneIdMessage::OperatorId(_) => MessageType::OperatorId,
        }
    }

    /// Encode the message into bytes according to OpenDroneID specification
    pub fn encode(&self) -> Result<Vec<u8>, OpenDroneIdError> {
        let mut bit_buffer = Vec::new();
        
        // Helper function to add bits to buffer (MSB first)
        fn add_bits(buffer: &mut Vec<bool>, value: u64, num_bits: usize) {
            for i in (0..num_bits).rev() {
                buffer.push((value >> i) & 1 == 1);
            }
        }
        
        // Add message type (3 bits)
        let msg_type: u8 = self.message_type().into();
        add_bits(&mut bit_buffer, msg_type as u64, 3);
        
        match self {
            OpenDroneIdMessage::BasicId(msg) => {
                // Basic ID message format
                add_bits(&mut bit_buffer, msg.uas_id_type as u64, 3);
                
                // Convert UAS ID string to bytes (max 20 bytes)
                let uas_id_bytes = msg.uas_id.as_bytes();
                if uas_id_bytes.len() > 20 {
                    return Err(OpenDroneIdError::EncodingError(
                        "UAS ID too long (max 20 bytes)".to_string()
                    ));
                }
                
                // Add UAS ID length (5 bits)
                add_bits(&mut bit_buffer, uas_id_bytes.len() as u64, 5);
                
                // Add UAS ID bytes
                for &byte in uas_id_bytes {
                    add_bits(&mut bit_buffer, byte as u64, 8);
                }
            }
            OpenDroneIdMessage::Location(msg) => {
                // Location message format
                add_bits(&mut bit_buffer, msg.status as u64, 3);
                add_bits(&mut bit_buffer, msg.direction as u64, 11);
                add_bits(&mut bit_buffer, msg.speed as u64, 10);
                add_bits(&mut bit_buffer, msg.latitude as u64, 32);
                add_bits(&mut bit_buffer, msg.longitude as u64, 32);
                add_bits(&mut bit_buffer, msg.altitude as u64, 16);
                add_bits(&mut bit_buffer, msg.height as u64, 16);
            }
            OpenDroneIdMessage::Auth(msg) => {
                // Auth message format
                add_bits(&mut bit_buffer, msg.auth_type as u64, 3);
                
                // Add auth data length (8 bits)
                if msg.auth_data.len() > 255 {
                    return Err(OpenDroneIdError::EncodingError(
                        "Auth data too long (max 255 bytes)".to_string()
                    ));
                }
                
                add_bits(&mut bit_buffer, msg.auth_data.len() as u64, 8);
                
                // Add auth data bytes
                for &byte in &msg.auth_data {
                    add_bits(&mut bit_buffer, byte as u64, 8);
                }
            }
            OpenDroneIdMessage::SelfId(msg) => {
                // Self ID message format
                let desc_bytes = msg.description.as_bytes();
                if desc_bytes.len() > 23 {
                    return Err(OpenDroneIdError::EncodingError(
                        "Description too long (max 23 bytes)".to_string()
                    ));
                }
                
                // Add description bytes
                for &byte in desc_bytes {
                    add_bits(&mut bit_buffer, byte as u64, 8);
                }
                
                // Pad to 23 bytes if needed
                for _ in desc_bytes.len()..23 {
                    add_bits(&mut bit_buffer, 0, 8);
                }
            }
            OpenDroneIdMessage::System(msg) => {
                // System message format
                add_bits(&mut bit_buffer, msg.operator_location_type as u64, 3);
                add_bits(&mut bit_buffer, msg.classification_type as u64, 3);
                add_bits(&mut bit_buffer, msg.operator_latitude as u64, 32);
                add_bits(&mut bit_buffer, msg.operator_longitude as u64, 32);
                add_bits(&mut bit_buffer, msg.area_count as u64, 3);
                add_bits(&mut bit_buffer, msg.area_radius as u64, 12);
                add_bits(&mut bit_buffer, msg.area_ceiling as u64, 12);
                add_bits(&mut bit_buffer, msg.area_floor as u64, 12);
            }
            OpenDroneIdMessage::OperatorId(msg) => {
                // Operator ID message format
                add_bits(&mut bit_buffer, msg.operator_id_type as u64, 3);
                
                // Convert operator ID string to bytes (max 20 bytes)
                let operator_id_bytes = msg.operator_id.as_bytes();
                if operator_id_bytes.len() > 20 {
                    return Err(OpenDroneIdError::EncodingError(
                        "Operator ID too long (max 20 bytes)".to_string()
                    ));
                }
                
                // Add operator ID length (5 bits)
                add_bits(&mut bit_buffer, operator_id_bytes.len() as u64, 5);
                
                // Add operator ID bytes
                for &byte in operator_id_bytes {
                    add_bits(&mut bit_buffer, byte as u64, 8);
                }
            }
        }
        
        // Convert bit buffer to bytes
        let byte_len = (bit_buffer.len() + 7) / 8;
        let mut bytes = vec![0u8; byte_len];
        for (i, &bit) in bit_buffer.iter().enumerate() {
            if bit {
                let byte_idx = i / 8;
                let bit_idx = 7 - (i % 8); // MSB first within byte
                bytes[byte_idx] |= 1 << bit_idx;
            }
        }
        
        Ok(bytes)
    }

    /// Decode bytes into an OpenDroneID message
    #[allow(unused_assignments)]
    pub fn decode(bytes: &[u8]) -> Result<OpenDroneIdMessage, OpenDroneIdError> {
        if bytes.is_empty() {
            return Err(OpenDroneIdError::DecodingError(
                "Empty input bytes".to_string()
            ));
        }
        
        // Convert bytes to bit buffer (MSB first)
        let mut bit_buffer = Vec::new();
        for &byte in bytes {
            for i in (0..8).rev() {
                bit_buffer.push((byte >> i) & 1 == 1);
            }
        }
        
        let mut bit_pos = 0;
        
        // Helper function to read bits from buffer
        fn read_bits(buffer: &[bool], start: usize, num_bits: usize) -> Result<u64, OpenDroneIdError> {
            if start + num_bits > buffer.len() {
                return Err(OpenDroneIdError::DecodingError(
                    format!("Incomplete message - need {} bits but only have {}", num_bits, buffer.len() - start)
                ));
            }
            
            let mut value = 0u64;
            for i in 0..num_bits {
                if buffer[start + i] {
                    value |= 1 << (num_bits - 1 - i);
                }
            }
            Ok(value)
        }
        
        // Read message type (3 bits)
        let msg_type_value = read_bits(&bit_buffer, bit_pos, 3)?;
        let msg_type = MessageType::from(msg_type_value as u8);
        bit_pos += 3;
        
        match msg_type {
            MessageType::BasicId => {
                // Read UAS ID type (3 bits)
                let uas_id_type = read_bits(&bit_buffer, bit_pos, 3)? as u8;
                bit_pos += 3;
                
                // Read UAS ID length (5 bits)
                let uas_id_len = read_bits(&bit_buffer, bit_pos, 5)? as u8;
                bit_pos += 5;
                
                // Read UAS ID bytes
                let mut uas_id_bytes = Vec::new();
                for _ in 0..uas_id_len {
                    let byte = read_bits(&bit_buffer, bit_pos, 8)? as u8;
                    uas_id_bytes.push(byte);
                    bit_pos += 8;
                }
                
                let uas_id = String::from_utf8(uas_id_bytes)
                    .map_err(|e| OpenDroneIdError::DecodingError(format!(
                        "Invalid UTF-8 in UAS ID: {}", e
                    )))?;
                
                Ok(OpenDroneIdMessage::BasicId(BasicIdMessage {
                    uas_id_type,
                    uas_id,
                }))
            }
            MessageType::Location => {
                // Read status (3 bits)
                let status = read_bits(&bit_buffer, bit_pos, 3)? as u8;
                bit_pos += 3;
                
                // Read direction (11 bits)
                let direction = read_bits(&bit_buffer, bit_pos, 11)? as u16;
                bit_pos += 11;
                
                // Read speed (10 bits)
                let speed = read_bits(&bit_buffer, bit_pos, 10)? as u16;
                bit_pos += 10;
                
                // Read latitude (32 bits)
                let latitude = read_bits(&bit_buffer, bit_pos, 32)? as i32;
                bit_pos += 32;
                
                // Read longitude (32 bits)
                let longitude = read_bits(&bit_buffer, bit_pos, 32)? as i32;
                bit_pos += 32;
                
                // Read altitude (16 bits)
                let altitude = read_bits(&bit_buffer, bit_pos, 16)? as i16;
                bit_pos += 16;
                
                // Read height (16 bits)
                let height = read_bits(&bit_buffer, bit_pos, 16)? as i16;
                bit_pos += 16;
                
                Ok(OpenDroneIdMessage::Location(LocationMessage {
                    status,
                    direction,
                    speed,
                    latitude,
                    longitude,
                    altitude,
                    height,
                }))
            }
            MessageType::Auth => {
                // Read auth type (3 bits)
                let auth_type = read_bits(&bit_buffer, bit_pos, 3)? as u8;
                bit_pos += 3;
                
                // Read auth data length (8 bits)
                let auth_data_len = read_bits(&bit_buffer, bit_pos, 8)? as u8;
                bit_pos += 8;
                
                // Read auth data bytes
                let mut auth_data = Vec::new();
                for _ in 0..auth_data_len {
                    let byte = read_bits(&bit_buffer, bit_pos, 8)? as u8;
                    auth_data.push(byte);
                    bit_pos += 8;
                }
                
                Ok(OpenDroneIdMessage::Auth(AuthMessage {
                    auth_type,
                    auth_data,
                }))
            }
            MessageType::SelfId => {
                // Self ID message is fixed 23 bytes (184 bits)
                if bit_pos + 184 > bit_buffer.len() {
                    return Err(OpenDroneIdError::DecodingError(
                        "Incomplete Self ID message".to_string()
                    ));
                }
                
                let mut description_bytes = Vec::new();
                for _ in 0..23 {
                    let byte = read_bits(&bit_buffer, bit_pos, 8)? as u8;
                    if byte != 0 {
                        description_bytes.push(byte);
                    }
                    bit_pos += 8;
                }
                
                let description = String::from_utf8(description_bytes)
                    .map_err(|e| OpenDroneIdError::DecodingError(format!(
                        "Invalid UTF-8 in Self ID description: {}", e
                    )))?;
                
                Ok(OpenDroneIdMessage::SelfId(SelfIdMessage {
                    description,
                }))
            }
            MessageType::System => {
                // Read operator location type (3 bits)
                let operator_location_type = read_bits(&bit_buffer, bit_pos, 3)? as u8;
                bit_pos += 3;
                
                // Read classification type (3 bits)
                let classification_type = read_bits(&bit_buffer, bit_pos, 3)? as u8;
                bit_pos += 3;
                
                // Read operator latitude (32 bits)
                let operator_latitude = read_bits(&bit_buffer, bit_pos, 32)? as i32;
                bit_pos += 32;
                
                // Read operator longitude (32 bits)
                let operator_longitude = read_bits(&bit_buffer, bit_pos, 32)? as i32;
                bit_pos += 32;
                
                // Read area count (3 bits)
                let area_count = read_bits(&bit_buffer, bit_pos, 3)? as u8;
                bit_pos += 3;
                
                // Read area radius (12 bits)
                let area_radius = read_bits(&bit_buffer, bit_pos, 12)? as u16;
                bit_pos = bit_pos.wrapping_add(12);
                
                // Read area ceiling (12 bits)
                let area_ceiling = read_bits(&bit_buffer, bit_pos, 12)? as i16;
                bit_pos += 12;
                
                // Read area floor (12 bits)
                let area_floor = read_bits(&bit_buffer, bit_pos, 12)? as i16;
                bit_pos += 12;
                
                Ok(OpenDroneIdMessage::System(SystemMessage {
                    operator_location_type,
                    classification_type,
                    operator_latitude,
                    operator_longitude,
                    area_count,
                    area_radius,
                    area_ceiling,
                    area_floor,
                }))
            }
            MessageType::OperatorId => {
                // Read operator ID type (3 bits)
                let operator_id_type = read_bits(&bit_buffer, bit_pos, 3)? as u8;
                bit_pos += 3;
                
                // Read operator ID length (5 bits)
                let operator_id_len = read_bits(&bit_buffer, bit_pos, 5)? as u8;
                bit_pos += 5;
                
                // Read operator ID bytes
                let mut operator_id_bytes = Vec::new();
                for _ in 0..operator_id_len {
                    let byte = read_bits(&bit_buffer, bit_pos, 8)? as u8;
                    operator_id_bytes.push(byte);
                    bit_pos += 8;
                }
                
                let operator_id = String::from_utf8(operator_id_bytes)
                    .map_err(|e| OpenDroneIdError::DecodingError(format!(
                        "Invalid UTF-8 in Operator ID: {}", e
                    )))?;
                
                Ok(OpenDroneIdMessage::OperatorId(OperatorIdMessage {
                    operator_id_type,
                    operator_id,
                }))
            }
        }
    }
}