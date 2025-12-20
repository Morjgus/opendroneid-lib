use clap::{Parser, Subcommand};
use opendroneid::{OpenDroneIdMessage, BasicIdMessage, LocationMessage, AuthMessage, SelfIdMessage, SystemMessage, OperatorIdMessage};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opendroneid-cli")]
#[command(about = "OpenDroneID message encoder/decoder CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode an OpenDroneID message
    Encode {
        #[command(subcommand)]
        message_type: EncodeMessageType,
        /// Output file (optional, prints to stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Decode an OpenDroneID message
    Decode {
        /// Input file or hex string
        input: String,
        /// Treat input as hex string
        #[arg(short = 'x', long)]
        hex: bool,
    },
}

#[derive(Subcommand)]
enum EncodeMessageType {
    /// Encode a Basic ID message
    BasicId {
        /// UAS ID type (0-7)
        #[arg(short = 't', long)]
        uas_id_type: u8,
        /// UAS ID string
        #[arg(short = 'i', long)]
        uas_id: String,
    },
    /// Encode a Location message
    Location {
        /// Status (0-7)
        #[arg(short, long)]
        status: u8,
        /// Direction (0-2047)
        #[arg(short, long)]
        direction: u16,
        /// Speed (0-1023)
        #[arg(short, long)]
        speed: u16,
        /// Latitude
        #[arg(short, long)]
        latitude: i32,
        /// Longitude
        #[arg(short, long)]
        longitude: i32,
        /// Altitude
        #[arg(short, long)]
        altitude: i16,
        /// Height
        #[arg(short, long)]
        height: i16,
    },
    /// Encode an Auth message
    Auth {
        /// Auth type (0-7)
        #[arg(short, long)]
        auth_type: u8,
        /// Auth data as hex string
        #[arg(short, long)]
        auth_data: String,
    },
    /// Encode a Self ID message
    SelfId {
        /// Description text (max 23 chars)
        #[arg(short, long)]
        description: String,
    },
    /// Encode a System message
    System {
        /// Operator location type (0-7)
        #[arg(short, long)]
        operator_location_type: u8,
        /// Classification type (0-7)
        #[arg(short, long)]
        classification_type: u8,
        /// Operator latitude
        #[arg(short, long)]
        operator_latitude: i32,
        /// Operator longitude
        #[arg(short, long)]
        operator_longitude: i32,
        /// Area count (0-7)
        #[arg(short, long)]
        area_count: u8,
        /// Area radius (0-4095)
        #[arg(short, long)]
        area_radius: u16,
        /// Area ceiling
        #[arg(short, long)]
        area_ceiling: i16,
        /// Area floor
        #[arg(short, long)]
        area_floor: i16,
    },
    /// Encode an Operator ID message
    OperatorId {
        /// Operator ID type (0-7)
        #[arg(short, long)]
        operator_id_type: u8,
        /// Operator ID string
        #[arg(short, long)]
        operator_id: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Encode { message_type, output } => {
            let message = match message_type {
                EncodeMessageType::BasicId { uas_id_type, uas_id } => {
                    OpenDroneIdMessage::BasicId(BasicIdMessage {
                        uas_id_type: *uas_id_type,
                        uas_id: uas_id.clone(),
                    })
                }
                EncodeMessageType::Location { status, direction, speed, latitude, longitude, altitude, height } => {
                    OpenDroneIdMessage::Location(LocationMessage {
                        status: *status,
                        direction: *direction,
                        speed: *speed,
                        latitude: *latitude,
                        longitude: *longitude,
                        altitude: *altitude,
                        height: *height,
                    })
                }
                EncodeMessageType::Auth { auth_type, auth_data } => {
                    let auth_bytes = hex::decode(auth_data).expect("Invalid hex string for auth data");
                    OpenDroneIdMessage::Auth(AuthMessage {
                        auth_type: *auth_type,
                        auth_data: auth_bytes,
                    })
                }
                EncodeMessageType::SelfId { description } => {
                    OpenDroneIdMessage::SelfId(SelfIdMessage {
                        description: description.clone(),
                    })
                }
                EncodeMessageType::System { operator_location_type, classification_type, operator_latitude, operator_longitude, area_count, area_radius, area_ceiling, area_floor } => {
                    OpenDroneIdMessage::System(SystemMessage {
                        operator_location_type: *operator_location_type,
                        classification_type: *classification_type,
                        operator_latitude: *operator_latitude,
                        operator_longitude: *operator_longitude,
                        area_count: *area_count,
                        area_radius: *area_radius,
                        area_ceiling: *area_ceiling,
                        area_floor: *area_floor,
                    })
                }
                EncodeMessageType::OperatorId { operator_id_type, operator_id } => {
                    OpenDroneIdMessage::OperatorId(OperatorIdMessage {
                        operator_id_type: *operator_id_type,
                        operator_id: operator_id.clone(),
                    })
                }
            };
            
            match message.encode() {
                Ok(encoded) => {
                    let hex_string = hex::encode(&encoded);
                    
                    if let Some(output_path) = output {
                        fs::write(output_path, hex_string).expect("Failed to write output file");
                        println!("Encoded message written to: {:?}", output_path);
                    } else {
                        println!("Encoded message (hex): {}", hex_string);
                        println!("Message type: {:?}", message.message_type());
                        println!("Binary length: {} bytes", encoded.len());
                    }
                }
                Err(e) => {
                    eprintln!("Encoding error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Decode { input, hex } => {
            let bytes = if *hex {
                hex::decode(input).expect("Invalid hex string")
            } else {
                // Try to read as file first
                if let Ok(content) = fs::read(input) {
                    content
                } else {
                    // If not a valid file, try as hex string
                    hex::decode(input).expect("Invalid hex string and file not found")
                }
            };
            
            match OpenDroneIdMessage::decode(&bytes) {
                Ok(decoded) => {
                    println!("Decoded OpenDroneID message:");
                    println!("Message type: {:?}", decoded.message_type());
                    
                    match decoded {
                        OpenDroneIdMessage::BasicId(msg) => {
                            println!("Basic ID Message:");
                            println!("  UAS ID Type: {}", msg.uas_id_type);
                            println!("  UAS ID: {}", msg.uas_id);
                        }
                        OpenDroneIdMessage::Location(msg) => {
                            println!("Location Message:");
                            println!("  Status: {}", msg.status);
                            println!("  Direction: {}", msg.direction);
                            println!("  Speed: {}", msg.speed);
                            println!("  Latitude: {}", msg.latitude);
                            println!("  Longitude: {}", msg.longitude);
                            println!("  Altitude: {}", msg.altitude);
                            println!("  Height: {}", msg.height);
                        }
                        OpenDroneIdMessage::Auth(msg) => {
                            println!("Auth Message:");
                            println!("  Auth Type: {}", msg.auth_type);
                            println!("  Auth Data: {}", hex::encode(&msg.auth_data));
                        }
                        OpenDroneIdMessage::SelfId(msg) => {
                            println!("Self ID Message:");
                            println!("  Description: {}", msg.description);
                        }
                        OpenDroneIdMessage::System(msg) => {
                            println!("System Message:");
                            println!("  Operator Location Type: {}", msg.operator_location_type);
                            println!("  Classification Type: {}", msg.classification_type);
                            println!("  Operator Latitude: {}", msg.operator_latitude);
                            println!("  Operator Longitude: {}", msg.operator_longitude);
                            println!("  Area Count: {}", msg.area_count);
                            println!("  Area Radius: {}", msg.area_radius);
                            println!("  Area Ceiling: {}", msg.area_ceiling);
                            println!("  Area Floor: {}", msg.area_floor);
                        }
                        OpenDroneIdMessage::OperatorId(msg) => {
                            println!("Operator ID Message:");
                            println!("  Operator ID Type: {}", msg.operator_id_type);
                            println!("  Operator ID: {}", msg.operator_id);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Decoding error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}