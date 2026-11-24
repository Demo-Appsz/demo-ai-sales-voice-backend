use std::sync::Arc;
use crate::domain::entities::CallSession;
use crate::domain::ports::SessionRepository;
use crate::domain::value_objects::{SessionId, Speaker};
use super::dtos::{CallDetailsDto, ChatResponseDto, EndCallResponseDto, StartCallResponseDto};

pub struct CallService {
    repo: Arc<dyn SessionRepository>,
}

impl CallService {
    pub fn new(repo: Arc<dyn SessionRepository>) -> Self {
        Self { repo }
    }

    pub async fn start_call(&self) -> Result<StartCallResponseDto, String> {
        let session = CallSession::new();
        let session_id_str = session.id.as_str().to_string();

        self.repo.save(session).await?;

        Ok(StartCallResponseDto {
            success: true,
            call_details: CallDetailsDto {
                call_id: session_id_str.clone(),
                status: "connected".to_string(),
                join_url: format!("wss://livekit.demo-sales.ai/join?token={}", session_id_str),
            },
        })
    }

    pub async fn end_call(&self, session_id: &str) -> Result<EndCallResponseDto, String> {
        let id = SessionId::from_string(session_id);
        let maybe_session = self.repo.find_by_id(&id).await?;

        if let Some(mut session) = maybe_session {
            session.disconnect();
            self.repo.update(session).await?;
            Ok(EndCallResponseDto {
                success: true,
                message: "Panggilan telah diakhiri.".to_string(),
                status: "disconnected".to_string(),
            })
        } else {
            Err("Session not found".to_string())
        }
    }

    pub async fn process_chat(&self, session_id: &str, user_message: &str) -> Result<ChatResponseDto, String> {
        let id = SessionId::from_string(session_id);
        let maybe_session = self.repo.find_by_id(&id).await?;

        let mut session = match maybe_session {
            Some(s) => s,
            None => {
                // If not found, start new session
                let s = CallSession::new();
                self.repo.save(s.clone()).await?;
                s
            }
        };

        // Add user message
        session.add_transcript(Speaker::User, user_message);

        // Generate AI customer service response
        let ai_reply = Self::generate_ai_reply(user_message);
        session.add_transcript(Speaker::Agent, &ai_reply);

        let transcripts = session.transcripts.clone();
        self.repo.update(session).await?;

        Ok(ChatResponseDto {
            success: true,
            reply: ai_reply,
            transcripts,
        })
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<CallSession>, String> {
        let id = SessionId::from_string(session_id);
        self.repo.find_by_id(&id).await
    }

    fn generate_ai_reply(input: &str) -> String {
        let lower = input.to_lowercase();
        if lower.contains("harga") || lower.contains("biaya") || lower.contains("paket") || lower.contains("price") {
            "Untuk paket layanan AI kami tersedia mulai dari Paket Starter (Rp 499.000/bulan) untuk UMKM hingga Enterprise dengan integrasi custom CRM dan WhatsApp API. Apakah Anda tertarik mencoba demo uji coba gratis selama 14 hari?".to_string()
        } else if lower.contains("fitur") || lower.contains("bisa apa") || lower.contains("keunggulan") {
            "Asisten AI kami mampu menangani customer service suara & chat otomatis 24/7, kualifikasi prospek penjualan (leads), integrasi langsung ke sistem CRM, dan analitik sentimen pelanggan secara real-time.".to_string()
        } else if lower.contains("integrasi") || lower.contains("whatsapp") || lower.contains("crm") {
            "AI kami dapat diintegrasikan dengan mudah ke WhatsApp Business, Telegram, Website Widget, serta CRM seperti Salesforce, HubSpot, atau database internal via Webhook & REST API.".to_string()
        } else if lower.contains("halo") || lower.contains("selamat") || lower.contains("pagi") || lower.contains("siang") || lower.contains("sore") || lower.contains("malam") {
            "Halo! Senang bisa berbicara dengan Anda. Saya siap membantu menjawab pertanyaan Anda terkait solusi AI Customer Service & Sales untuk bisnis Anda.".to_string()
        } else if lower.contains("terima kasih") || lower.contains("makasih") || lower.contains("thanks") {
            "Sama-sama! Senang bisa membantu Anda. Jika ada hal lain yang ingin ditanyakan, jangan ragu untuk berbicara kembali ya.".to_string()
        } else {
            format!(
                "Baik, terkait '{}', asisten AI kami dapat mengotomatisasi proses tersebut dengan respon cepat di bawah 1 detik dan akurasi tinggi. Apakah ada use case khusus yang ingin Anda terapkan di bisnis Anda?",
                input.trim()
            )
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::adapters::in_memory_repository::InMemorySessionRepository;

    #[tokio::test]
    async fn test_start_and_end_call_flow() {
        let repo = Arc::new(InMemorySessionRepository::new());
        let service = CallService::new(repo);

        let start_res = service.start_call().await.unwrap();
        assert!(start_res.success);
        assert!(!start_res.call_details.call_id.is_empty());
        assert_eq!(start_res.call_details.status, "connected");

        let chat_res = service
            .process_chat(&start_res.call_details.call_id, "Berapa harga paketnya?")
            .await
            .unwrap();
        assert!(chat_res.success);
        assert!(chat_res.reply.contains("Starter") || chat_res.reply.contains("harga"));

        let end_res = service
            .end_call(&start_res.call_details.call_id)
            .await
            .unwrap();
        assert!(end_res.success);
        assert_eq!(end_res.status, "disconnected");
    }
}
