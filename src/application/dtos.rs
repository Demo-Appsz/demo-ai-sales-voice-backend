use serde::{Deserialize, Serialize};
use crate::domain::value_objects::TranscriptEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct CallDetailsDto {
    #[serde(rename = "callId")]
    pub call_id: String,
    pub status: String,
    #[serde(rename = "joinUrl")]
    pub join_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartCallResponseDto {
    pub success: bool,
    #[serde(rename = "callDetails")]
    pub call_details: CallDetailsDto,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequestDto {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponseDto {
    pub success: bool,
    pub reply: String,
    pub transcripts: Vec<TranscriptEntry>,
}

#[derive(Debug, Serialize)]
pub struct EndCallResponseDto {
    pub success: bool,
    pub message: String,
    pub status: String,
}
