use super::value_objects::{CallStatus, SessionId, Speaker, TranscriptEntry};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSession {
    pub id: SessionId,
    pub status: CallStatus,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub transcripts: Vec<TranscriptEntry>,
}

impl CallSession {
    pub fn new() -> Self {
        let mut session = Self {
            id: SessionId::new(),
            status: CallStatus::Connected,
            created_at: Utc::now(),
            ended_at: None,
            transcripts: Vec::new(),
        };

        // Welcome greeting from Asisten Layanan Pelanggan AI
        session.add_transcript(
            Speaker::Agent,
            "Halo! Saya asisten virtual layanan pelanggan AI. Ada yang bisa saya bantu hari ini?",
        );

        session
    }

    pub fn add_transcript(&mut self, speaker: Speaker, text: impl Into<String>) {
        self.transcripts.push(TranscriptEntry::new(speaker, text));
    }

    pub fn disconnect(&mut self) {
        self.status = CallStatus::Disconnected;
        self.ended_at = Some(Utc::now());
    }
}

impl Default for CallSession {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_session_creation() {
        let session = CallSession::new();
        assert_eq!(session.status, CallStatus::Connected);
        assert_eq!(session.transcripts.len(), 1);
        assert_eq!(session.transcripts[0].speaker, Speaker::Agent);
        assert!(session.ended_at.is_none());
    }

    #[test]
    fn test_call_session_disconnect() {
        let mut session = CallSession::new();
        session.disconnect();
        assert_eq!(session.status, CallStatus::Disconnected);
        assert!(session.ended_at.is_some());
    }

    #[test]
    fn test_add_transcript() {
        let mut session = CallSession::new();
        session.add_transcript(Speaker::User, "Berapa biaya layanannya?");
        assert_eq!(session.transcripts.len(), 2);
        assert_eq!(session.transcripts[1].speaker, Speaker::User);
        assert_eq!(session.transcripts[1].text, "Berapa biaya layanannya?");
    }
}
