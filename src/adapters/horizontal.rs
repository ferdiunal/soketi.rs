use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
// use crate::channels::presence_channel_manager::PresenceMemberInfo; (Removed)

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PresenceMemberInfo {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "user_info")]
    pub user_info: Value,
}

// TypeScript tarafında:
// export enum RequestType {
//     SOCKETS = 0,
//     CHANNELS = 1,
//     CHANNEL_SOCKETS = 2,
//     CHANNEL_MEMBERS = 3,
//     SOCKETS_COUNT = 4,
//     CHANNEL_MEMBERS_COUNT = 5,
//     CHANNEL_SOCKETS_COUNT = 6,
//     SOCKET_EXISTS_IN_CHANNEL = 7,
//     CHANNELS_WITH_SOCKETS_COUNT = 8,
//     TERMINATE_USER_CONNECTIONS = 9,
// }
// Basitlik için ve Soketi TS kodu JSON parse ederken `type` alanını number beklediği için custom serializer/deserializer yazabiliriz veya u8 kullanabiliriz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestType {
    Sockets = 0,
    Channels = 1,
    ChannelSockets = 2,
    ChannelMembers = 3,
    SocketsCount = 4,
    ChannelMembersCount = 5,
    ChannelSocketsCount = 6,
    SocketExistsInChannel = 7,
    ChannelsWithSocketsCount = 8,
    TerminateUserConnections = 9,
}

// RequestType serialization/deserialization helper
impl serde::Serialize for RequestType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> serde::Deserialize<'de> for RequestType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(RequestType::Sockets),
            1 => Ok(RequestType::Channels),
            2 => Ok(RequestType::ChannelSockets),
            3 => Ok(RequestType::ChannelMembers),
            4 => Ok(RequestType::SocketsCount),
            5 => Ok(RequestType::ChannelMembersCount),
            6 => Ok(RequestType::ChannelSocketsCount),
            7 => Ok(RequestType::SocketExistsInChannel),
            8 => Ok(RequestType::ChannelsWithSocketsCount),
            9 => Ok(RequestType::TerminateUserConnections),
            _ => Err(serde::de::Error::custom("Unknown RequestType")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestExtra {
    #[serde(rename = "numSub")]
    pub num_sub: Option<usize>,
    #[serde(rename = "msgCount")]
    pub msg_count: Option<usize>,
    pub sockets: Option<Vec<SocketInfo>>, // Socket yerine seri hale getirilebilir bir struct kullanmalıyız
    pub members: Option<HashMap<String, PresenceMemberInfo>>,
    pub channels: Option<HashMap<String, HashSet<String>>>,
    #[serde(rename = "channelsWithSocketsCount")]
    pub channels_with_sockets_count: Option<HashMap<String, usize>>,
    #[serde(rename = "totalCount")]
    pub total_count: Option<usize>,
    pub exists: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestOptions {
    #[serde(default)]
    pub opts: HashMap<String, Value>,
}

// TypeScript'teki `Request` ve `RequestBody` yapılarını birleştiren yapı
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestBody {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    #[serde(rename = "type")]
    pub request_type: RequestType,
    #[serde(flatten)]
    pub options: RequestOptions,
    #[serde(flatten)]
    pub extra: RequestExtra,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub sockets: Option<Vec<SocketInfo>>,
    pub members: Option<HashMap<String, PresenceMemberInfo>>, // TS: [string, PresenceMemberInfo][] -> Rust: HashMap veya Vec<(String, Info)>
    pub channels: Option<HashMap<String, HashSet<String>>>, // TS: [string, string[]][] -> Rust: HashMap<String, HashSet<String>>
    #[serde(rename = "channelsWithSocketsCount")]
    pub channels_with_sockets_count: Option<HashMap<String, usize>>, // TS: [string, number][]
    #[serde(rename = "totalCount")]
    pub total_count: Option<usize>,
    pub exists: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PubsubBroadcastedMessage {
    pub uuid: String,
    #[serde(rename = "appId")]
    pub app_id: String,
    pub channel: String,
    pub data: Value, // any -> serde_json::Value
    #[serde(rename = "exceptingId")]
    pub excepting_id: Option<String>,
}

// Socket nesnesinin serileştirilebilir hali (Referans: horizontal-adapter.ts satır 580-586)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocketInfo {
    pub id: String,
    #[serde(rename = "subscribedChannels")]
    pub subscribed_channels: HashSet<String>,
    pub presence: Option<String>, // veya detaylı presence bilgisi
    pub ip: Option<String>,
    pub ip2: Option<String>,
}
