use super::*;

#[test]
fn test_basic_id_encode_decode() {
    let original = OpenDroneIdMessage::BasicId(BasicIdMessage {
        uas_id_type: 1,
        uas_id: "TEST123".to_string(),
    });
    
    let encoded = original.encode().unwrap();
    let decoded = OpenDroneIdMessage::decode(&encoded).unwrap();
    
    match (original, decoded) {
        (OpenDroneIdMessage::BasicId(original_msg), OpenDroneIdMessage::BasicId(decoded_msg)) => {
            assert_eq!(original_msg.uas_id_type, decoded_msg.uas_id_type);
            assert_eq!(original_msg.uas_id, decoded_msg.uas_id);
        }
        _ => panic!("Message type mismatch"),
    }
}

#[test]
fn test_location_encode_decode() {
    let original = OpenDroneIdMessage::Location(LocationMessage {
        status: 2,
        direction: 45,
        speed: 10,
        latitude: 123456789,
        longitude: -987654321,
        altitude: 500,
        height: 100,
    });
    
    let encoded = original.encode().unwrap();
    let decoded = OpenDroneIdMessage::decode(&encoded).unwrap();
    
    match (original, decoded) {
        (OpenDroneIdMessage::Location(original_msg), OpenDroneIdMessage::Location(decoded_msg)) => {
            assert_eq!(original_msg.status, decoded_msg.status);
            assert_eq!(original_msg.direction, decoded_msg.direction);
            assert_eq!(original_msg.speed, decoded_msg.speed);
            assert_eq!(original_msg.latitude, decoded_msg.latitude);
            assert_eq!(original_msg.longitude, decoded_msg.longitude);
            assert_eq!(original_msg.altitude, decoded_msg.altitude);
            assert_eq!(original_msg.height, decoded_msg.height);
        }
        _ => panic!("Message type mismatch"),
    }
}

#[test]
fn test_auth_encode_decode() {
    let auth_data = vec![0x01, 0x02, 0x03, 0x04];
    let original = OpenDroneIdMessage::Auth(AuthMessage {
        auth_type: 3,
        auth_data: auth_data.clone(),
    });
    
    let encoded = original.encode().unwrap();
    let decoded = OpenDroneIdMessage::decode(&encoded).unwrap();
    
    match (original, decoded) {
        (OpenDroneIdMessage::Auth(original_msg), OpenDroneIdMessage::Auth(decoded_msg)) => {
            assert_eq!(original_msg.auth_type, decoded_msg.auth_type);
            assert_eq!(original_msg.auth_data, decoded_msg.auth_data);
        }
        _ => panic!("Message type mismatch"),
    }
}

#[test]
fn test_self_id_encode_decode() {
    let original = OpenDroneIdMessage::SelfId(SelfIdMessage {
        description: "Test drone description".to_string(),
    });
    
    let encoded = original.encode().unwrap();
    let decoded = OpenDroneIdMessage::decode(&encoded).unwrap();
    
    match (original, decoded) {
        (OpenDroneIdMessage::SelfId(original_msg), OpenDroneIdMessage::SelfId(decoded_msg)) => {
            assert_eq!(original_msg.description, decoded_msg.description);
        }
        _ => panic!("Message type mismatch"),
    }
}

#[test]
fn test_system_encode_decode() {
    let original = OpenDroneIdMessage::System(SystemMessage {
        operator_location_type: 1,
        classification_type: 2,
        operator_latitude: 12345678,
        operator_longitude: -87654321,
        area_count: 3,
        area_radius: 1000,
        area_ceiling: 500,
        area_floor: 100,
    });
    
    let encoded = original.encode().unwrap();
    let decoded = OpenDroneIdMessage::decode(&encoded).unwrap();
    
    match (original, decoded) {
        (OpenDroneIdMessage::System(original_msg), OpenDroneIdMessage::System(decoded_msg)) => {
            assert_eq!(original_msg.operator_location_type, decoded_msg.operator_location_type);
            assert_eq!(original_msg.classification_type, decoded_msg.classification_type);
            assert_eq!(original_msg.operator_latitude, decoded_msg.operator_latitude);
            assert_eq!(original_msg.operator_longitude, decoded_msg.operator_longitude);
            assert_eq!(original_msg.area_count, decoded_msg.area_count);
            assert_eq!(original_msg.area_radius, decoded_msg.area_radius);
            assert_eq!(original_msg.area_ceiling, decoded_msg.area_ceiling);
            assert_eq!(original_msg.area_floor, decoded_msg.area_floor);
        }
        _ => panic!("Message type mismatch"),
    }
}

#[test]
fn test_operator_id_encode_decode() {
    let original = OpenDroneIdMessage::OperatorId(OperatorIdMessage {
        operator_id_type: 1,
        operator_id: "OPERATOR123".to_string(),
    });
    
    let encoded = original.encode().unwrap();
    let decoded = OpenDroneIdMessage::decode(&encoded).unwrap();
    
    match (original, decoded) {
        (OpenDroneIdMessage::OperatorId(original_msg), OpenDroneIdMessage::OperatorId(decoded_msg)) => {
            assert_eq!(original_msg.operator_id_type, decoded_msg.operator_id_type);
            assert_eq!(original_msg.operator_id, decoded_msg.operator_id);
        }
        _ => panic!("Message type mismatch"),
    }
}

#[test]
fn test_message_type_conversion() {
    // Test all message type conversions
    for i in 0..=5 {
        let msg_type = MessageType::from(i);
        let value: u8 = msg_type.into();
        assert_eq!(value, i);
    }
}

#[test]
fn test_invalid_message_type() {
    // Test that invalid message type causes panic (as per our implementation)
    // This is expected behavior for our simple implementation
    // In production, you might want to handle this more gracefully
    let result = std::panic::catch_unwind(|| {
        let _ = MessageType::from(10); // Invalid value
    });
    assert!(result.is_err());
}

#[test]
fn test_empty_decode() {
    let result = OpenDroneIdMessage::decode(&[]);
    assert!(matches!(result, Err(OpenDroneIdError::DecodingError(_))));
}

#[test]
fn test_incomplete_message() {
    // Test decoding with incomplete data
    let incomplete_bytes = vec![0b10100000]; // Just message type, no data
    let result = OpenDroneIdMessage::decode(&incomplete_bytes);
    assert!(matches!(result, Err(OpenDroneIdError::DecodingError(_))));
}