use serde::{Deserialize, Serialize};

/// Telegram API response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct TelegramResponse<T> {
    pub ok: bool,
    #[serde(default)]
    pub result: Option<T>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub error_code: Option<i32>,
}

/// Update from Telegram getUpdates
#[derive(Debug, Clone, Deserialize)]
pub struct Update {
    pub update_id: i64,
    #[serde(default)]
    pub message: Option<Message>,
    #[serde(default)]
    pub edited_message: Option<Message>,
    #[serde(default)]
    pub callback_query: Option<CallbackQuery>,
}

/// Telegram Message
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub date: i64,
    pub chat: Chat,
    #[serde(default)]
    pub message_thread_id: Option<i64>,
    /// DM threaded mode topic (BotFather → Threaded Mode ON).
    /// Uses a different field than forum groups' message_thread_id.
    #[serde(default)]
    pub direct_messages_topic: Option<DirectMessagesTopic>,
    #[serde(default)]
    pub is_topic_message: bool,
    #[serde(default)]
    pub from: Option<User>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub voice: Option<Voice>,
    #[serde(default)]
    pub audio: Option<Audio>,
    #[serde(default)]
    pub video: Option<Video>,
    #[serde(default)]
    pub video_note: Option<VideoNote>,
    #[serde(default)]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(default)]
    pub document: Option<Document>,
    #[serde(default)]
    pub sticker: Option<Sticker>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub contact: Option<Contact>,
    #[serde(default)]
    pub location: Option<Location>,
    #[serde(default)]
    pub venue: Option<Venue>,
    #[serde(default)]
    pub poll: Option<Poll>,
    #[serde(default)]
    pub dice: Option<Dice>,
    #[serde(default)]
    pub game: Option<Game>,
    #[serde(default)]
    pub reply_to_message: Option<Box<Message>>,
}

impl Message {
    /// Get the effective thread ID for topic routing.
    /// Prefers direct_messages_topic if present, falls back to message_thread_id.
    pub fn effective_thread_id(&self) -> Option<i64> {
        self.direct_messages_topic
            .as_ref()
            .map(|t| t.topic_id)
            .or(self.message_thread_id)
    }

    /// Get the text content (text or caption)
    pub fn content(&self) -> Option<&str> {
        self.text.as_deref().or(self.caption.as_deref())
    }

    /// Get the user ID if available
    pub fn user_id(&self) -> Option<i64> {
        self.from.as_ref().map(|u| u.id)
    }

    /// Check if message has any media (downloadable files)
    pub fn has_media(&self) -> bool {
        self.voice.is_some()
            || self.audio.is_some()
            || self.video.is_some()
            || self.video_note.is_some()
            || self.photo.is_some()
            || self.document.is_some()
            || self.sticker.is_some()
            || self.animation.is_some()
    }

    /// Check if message has special content (non-file media)
    pub fn has_special_content(&self) -> bool {
        self.contact.is_some()
            || self.location.is_some()
            || self.venue.is_some()
            || self.poll.is_some()
            || self.dice.is_some()
            || self.game.is_some()
    }

    /// Get the media type as a string
    /// Note: animation is checked before document because Telegram sends both for GIFs
    pub fn media_type(&self) -> Option<&'static str> {
        if self.voice.is_some() {
            Some("voice")
        } else if self.audio.is_some() {
            Some("audio")
        } else if self.video.is_some() {
            Some("video")
        } else if self.video_note.is_some() {
            Some("video_note")
        } else if self.photo.is_some() {
            Some("photo")
        } else if self.animation.is_some() {
            // Check animation BEFORE document - Telegram sends both for GIFs
            Some("animation")
        } else if self.document.is_some() {
            Some("document")
        } else if self.sticker.is_some() {
            Some("sticker")
        } else {
            None
        }
    }

    /// Get the special content type as a string (non-file media)
    pub fn special_content_type(&self) -> Option<&'static str> {
        if self.contact.is_some() {
            Some("contact")
        } else if self.location.is_some() {
            Some("location")
        } else if self.venue.is_some() {
            Some("venue")
        } else if self.poll.is_some() {
            Some("poll")
        } else if self.dice.is_some() {
            Some("dice")
        } else if self.game.is_some() {
            Some("game")
        } else {
            None
        }
    }

    /// Get the file_id of the media (if any)
    /// Note: animation is checked before document because Telegram sends both for GIFs
    pub fn media_file_id(&self) -> Option<&str> {
        if let Some(ref voice) = self.voice {
            return Some(&voice.file_id);
        }
        if let Some(ref audio) = self.audio {
            return Some(&audio.file_id);
        }
        if let Some(ref video) = self.video {
            return Some(&video.file_id);
        }
        if let Some(ref video_note) = self.video_note {
            return Some(&video_note.file_id);
        }
        if let Some(ref photos) = self.photo {
            // Get largest photo
            return photos
                .iter()
                .max_by_key(|p| p.file_size.unwrap_or(0))
                .map(|p| p.file_id.as_str());
        }
        // Check animation BEFORE document - Telegram sends both for GIFs
        if let Some(ref animation) = self.animation {
            return Some(&animation.file_id);
        }
        if let Some(ref document) = self.document {
            return Some(&document.file_id);
        }
        if let Some(ref sticker) = self.sticker {
            return Some(&sticker.file_id);
        }
        None
    }

    /// Get the MIME type from the media if available
    pub fn media_mime_type(&self) -> Option<&str> {
        if let Some(ref voice) = self.voice {
            return voice.mime_type.as_deref();
        }
        if let Some(ref audio) = self.audio {
            return audio.mime_type.as_deref();
        }
        if let Some(ref video) = self.video {
            return video.mime_type.as_deref();
        }
        if let Some(ref animation) = self.animation {
            return animation.mime_type.as_deref();
        }
        if let Some(ref document) = self.document {
            return document.mime_type.as_deref();
        }
        None
    }
}

/// Telegram Chat
#[derive(Debug, Clone, Deserialize)]
pub struct Chat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    #[serde(default)]
    pub is_forum: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

/// Telegram User
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub language_code: Option<String>,
}

/// Voice message
#[derive(Debug, Clone, Deserialize)]
pub struct Voice {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: i32,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
}

/// Audio file
#[derive(Debug, Clone, Deserialize)]
pub struct Audio {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: i32,
    #[serde(default)]
    pub performer: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
}

/// Video file
#[derive(Debug, Clone, Deserialize)]
pub struct Video {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub duration: i32,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
}

/// Video note (round video)
#[derive(Debug, Clone, Deserialize)]
pub struct VideoNote {
    pub file_id: String,
    pub file_unique_id: String,
    pub length: i32,
    pub duration: i32,
    #[serde(default)]
    pub file_size: Option<i64>,
}

/// Photo size
#[derive(Debug, Clone, Deserialize)]
pub struct PhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    #[serde(default)]
    pub file_size: Option<i64>,
}

/// Document
#[derive(Debug, Clone, Deserialize)]
pub struct Document {
    pub file_id: String,
    pub file_unique_id: String,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
}

/// Sticker
#[derive(Debug, Clone, Deserialize)]
pub struct Sticker {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub is_animated: bool,
    pub is_video: bool,
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
}

/// Animation (GIF or H.264/MPEG-4 AVC video without sound)
#[derive(Debug, Clone, Deserialize)]
pub struct Animation {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub duration: i32,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
    #[serde(default)]
    pub file_name: Option<String>,
}

/// Contact shared in message
#[derive(Debug, Clone, Deserialize)]
pub struct Contact {
    pub phone_number: String,
    pub first_name: String,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub vcard: Option<String>,
}

/// Location shared in message
#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
    #[serde(default)]
    pub horizontal_accuracy: Option<f64>,
    #[serde(default)]
    pub live_period: Option<i32>,
    #[serde(default)]
    pub heading: Option<i32>,
    #[serde(default)]
    pub proximity_alert_radius: Option<i32>,
}

/// Venue (location with name/address)
#[derive(Debug, Clone, Deserialize)]
pub struct Venue {
    pub location: Location,
    pub title: String,
    pub address: String,
    #[serde(default)]
    pub foursquare_id: Option<String>,
    #[serde(default)]
    pub foursquare_type: Option<String>,
    #[serde(default)]
    pub google_place_id: Option<String>,
    #[serde(default)]
    pub google_place_type: Option<String>,
}

/// Poll in message
#[derive(Debug, Clone, Deserialize)]
pub struct Poll {
    pub id: String,
    pub question: String,
    pub options: Vec<PollOption>,
    pub total_voter_count: i32,
    pub is_closed: bool,
    pub is_anonymous: bool,
    #[serde(rename = "type")]
    pub poll_type: String,
    #[serde(default)]
    pub allows_multiple_answers: bool,
    #[serde(default)]
    pub correct_option_id: Option<i32>,
    #[serde(default)]
    pub explanation: Option<String>,
}

/// Poll option
#[derive(Debug, Clone, Deserialize)]
pub struct PollOption {
    pub text: String,
    pub voter_count: i32,
}

/// Dice with random value
#[derive(Debug, Clone, Deserialize)]
pub struct Dice {
    pub emoji: String,
    pub value: i32,
}

/// Game
#[derive(Debug, Clone, Deserialize)]
pub struct Game {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub photo: Option<Vec<PhotoSize>>,
    #[serde(default)]
    pub text: Option<String>,
}

/// Callback query (from inline keyboard)
#[derive(Debug, Clone, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub from: User,
    #[serde(default)]
    pub message: Option<Message>,
    #[serde(default)]
    pub data: Option<String>,
}

/// File info from getFile
#[derive(Debug, Clone, Default, Deserialize)]
pub struct File {
    pub file_id: String,
    pub file_unique_id: String,
    #[serde(default)]
    pub file_size: Option<i64>,
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Reaction type for setMessageReaction
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ReactionType {
    #[serde(rename = "emoji")]
    Emoji { emoji: String },
}

impl ReactionType {
    pub fn emoji(emoji: impl Into<String>) -> Self {
        ReactionType::Emoji {
            emoji: emoji.into(),
        }
    }
}

/// DM threaded mode topic identifier.
/// Sent when a bot has Threaded Mode enabled via BotFather.
#[derive(Debug, Clone, Deserialize)]
pub struct DirectMessagesTopic {
    pub topic_id: i64,
}

/// Chat action for sendChatAction
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatAction {
    Typing,
    UploadPhoto,
    RecordVideo,
    UploadVideo,
    RecordVoice,
    UploadVoice,
    UploadDocument,
    FindLocation,
    RecordVideoNote,
    UploadVideoNote,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_message() {
        let json = r#"{
            "update_id": 123456789,
            "message": {
                "message_id": 1,
                "date": 1234567890,
                "chat": {"id": 123, "type": "private"},
                "from": {"id": 456, "is_bot": false, "first_name": "Test"},
                "text": "Hello, world!"
            }
        }"#;

        let update: Update = serde_json::from_str(json).unwrap();
        assert_eq!(update.update_id, 123456789);

        let msg = update.message.unwrap();
        assert_eq!(msg.content(), Some("Hello, world!"));
        assert_eq!(msg.user_id(), Some(456));
        assert!(!msg.has_media());
    }

    #[test]
    fn test_parse_voice_message() {
        let json = r#"{
            "update_id": 123456789,
            "message": {
                "message_id": 1,
                "date": 1234567890,
                "chat": {"id": 123, "type": "private"},
                "from": {"id": 456, "is_bot": false, "first_name": "Test"},
                "voice": {
                    "file_id": "abc123",
                    "file_unique_id": "xyz",
                    "duration": 5
                }
            }
        }"#;

        let update: Update = serde_json::from_str(json).unwrap();
        let msg = update.message.unwrap();

        assert!(msg.has_media());
        assert_eq!(msg.media_type(), Some("voice"));
        assert_eq!(msg.media_file_id(), Some("abc123"));
    }

    #[test]
    fn test_parse_photo_message() {
        let json = r#"{
            "update_id": 123456789,
            "message": {
                "message_id": 1,
                "date": 1234567890,
                "chat": {"id": 123, "type": "private"},
                "from": {"id": 456, "is_bot": false, "first_name": "Test"},
                "photo": [
                    {"file_id": "small", "file_unique_id": "a", "width": 100, "height": 100, "file_size": 1000},
                    {"file_id": "large", "file_unique_id": "b", "width": 800, "height": 800, "file_size": 50000}
                ],
                "caption": "Check this out!"
            }
        }"#;

        let update: Update = serde_json::from_str(json).unwrap();
        let msg = update.message.unwrap();

        assert!(msg.has_media());
        assert_eq!(msg.media_type(), Some("photo"));
        assert_eq!(msg.media_file_id(), Some("large")); // Should get largest
        assert_eq!(msg.content(), Some("Check this out!"));
    }

    #[cfg(test)]
    mod prop_tests {
        use super::*;
        use proptest::prelude::*;
        use serde_json::json;

        fn arb_chat_type() -> impl Strategy<Value = String> {
            prop::sample::select(vec![
                "private".to_string(),
                "group".to_string(),
                "supergroup".to_string(),
                "channel".to_string(),
            ])
        }

        fn arb_text() -> impl Strategy<Value = String> {
            prop::collection::vec(
                prop::sample::select(vec![
                    "Hello",
                    "World",
                    "Test",
                    "Message",
                    "Bot",
                    "User",
                    "How",
                    "are",
                    "you",
                    "today",
                    "?",
                    "!",
                    ".",
                    ",",
                    " ",
                    "\n",
                    "123",
                    "abc",
                    "Test emoji: OK",
                ]),
                1..20,
            )
            .prop_map(|parts| parts.join(""))
        }

        fn arb_username() -> impl Strategy<Value = String> {
            prop::string::string_regex("[a-zA-Z][a-zA-Z0-9_]{4,31}").unwrap()
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]

            #[test]
            fn prop_update_json_roundtrip(
                update_id in any::<i64>(),
                message_id in any::<i64>(),
                chat_id in any::<i64>(),
                chat_type in arb_chat_type(),
                user_id in any::<i64>(),
                first_name in arb_text(),
                text in arb_text(),
            ) {
                let json_value = json!({
                    "update_id": update_id,
                    "message": {
                        "message_id": message_id,
                        "date": 1234567890i64,
                        "chat": {"id": chat_id, "type": chat_type},
                        "from": {"id": user_id, "is_bot": false, "first_name": first_name},
                        "text": text
                    }
                });

                let json_str = json_value.to_string();
                let parsed: Result<Update, _> = serde_json::from_str(&json_str);

                assert!(parsed.is_ok(), "Failed to parse: {}", json_str);
                let update = parsed.unwrap();

                assert_eq!(update.update_id, update_id);

                let msg = update.message.unwrap();
                assert_eq!(msg.message_id, message_id);
                assert_eq!(msg.chat.id, chat_id);
                assert_eq!(msg.chat.chat_type, chat_type);
                assert_eq!(msg.content(), Some(text.as_str()));
            }

            #[test]
            fn prop_voice_message_roundtrip(
                update_id in any::<i64>(),
                message_id in any::<i64>(),
                chat_id in any::<i64>(),
                file_id in arb_username(),
                file_unique_id in arb_username(),
                duration in 1i32..3600i32,
            ) {
                let json_value = json!({
                    "update_id": update_id,
                    "message": {
                        "message_id": message_id,
                        "date": 1234567890i64,
                        "chat": {"id": chat_id, "type": "private"},
                        "voice": {
                            "file_id": file_id,
                            "file_unique_id": file_unique_id,
                            "duration": duration
                        }
                    }
                });

                let json_str = json_value.to_string();
                let parsed: Update = serde_json::from_str(&json_str).unwrap();
                let msg = parsed.message.unwrap();

                assert!(msg.has_media());
                assert_eq!(msg.media_type(), Some("voice"));
                assert_eq!(msg.media_file_id(), Some(file_id.as_str()));
                assert_eq!(msg.voice.unwrap().duration, duration);
            }

            #[test]
            fn prop_photo_selects_largest(
                update_id in any::<i64>(),
                message_id in any::<i64>(),
                chat_id in any::<i64>(),
                small_size in 1i64..1000i64,
                large_size in 1000i64..100000i64,
            ) {
                let json_value = json!({
                    "update_id": update_id,
                    "message": {
                        "message_id": message_id,
                        "date": 1234567890i64,
                        "chat": {"id": chat_id, "type": "private"},
                        "photo": [
                            {"file_id": "small_id", "file_unique_id": "a", "width": 100, "height": 100, "file_size": small_size},
                            {"file_id": "large_id", "file_unique_id": "b", "width": 800, "height": 800, "file_size": large_size}
                        ]
                    }
                });

                let json_str = json_value.to_string();
                let parsed: Update = serde_json::from_str(&json_str).unwrap();
                let msg = parsed.message.unwrap();

                assert!(msg.has_media());
                assert_eq!(msg.media_type(), Some("photo"));
                // Should always pick the larger one
                assert_eq!(msg.media_file_id(), Some("large_id"));
            }

            #[test]
            fn prop_message_with_optional_fields(
                update_id in any::<i64>(),
                message_id in any::<i64>(),
                chat_id in any::<i64>(),
                has_username in any::<bool>(),
                has_last_name in any::<bool>(),
            ) {
                let mut json_value = json!({
                    "update_id": update_id,
                    "message": {
                        "message_id": message_id,
                        "date": 1234567890i64,
                        "chat": {"id": chat_id, "type": "private"},
                        "from": {"id": 123, "is_bot": false, "first_name": "Test"}
                    }
                });

                if has_username {
                    json_value["message"]["from"]["username"] = json!("test_user");
                }
                if has_last_name {
                    json_value["message"]["from"]["last_name"] = json!("LastName");
                }

                let json_str = json_value.to_string();
                let parsed: Result<Update, _> = serde_json::from_str(&json_str);

                assert!(parsed.is_ok(), "Should parse with optional fields");
                let update = parsed.unwrap();
                let from = update.message.unwrap().from.unwrap();

                assert_eq!(from.username.is_some(), has_username);
                assert_eq!(from.last_name.is_some(), has_last_name);
            }

            #[test]
            fn prop_media_type_priority(
                has_voice in any::<bool>(),
                has_video in any::<bool>(),
                has_photo in any::<bool>(),
            ) {
                // Test that media_type returns in consistent priority order
                let mut json_value = json!({
                    "update_id": 1,
                    "message": {
                        "message_id": 1,
                        "date": 1234567890i64,
                        "chat": {"id": 123, "type": "private"}
                    }
                });

                if has_voice {
                    json_value["message"]["voice"] = json!({
                        "file_id": "voice_id",
                        "file_unique_id": "v",
                        "duration": 5
                    });
                }
                if has_video {
                    json_value["message"]["video"] = json!({
                        "file_id": "video_id",
                        "file_unique_id": "vid",
                        "width": 640,
                        "height": 480,
                        "duration": 10
                    });
                }
                if has_photo {
                    json_value["message"]["photo"] = json!([{
                        "file_id": "photo_id",
                        "file_unique_id": "p",
                        "width": 100,
                        "height": 100
                    }]);
                }

                let json_str = json_value.to_string();
                let parsed: Update = serde_json::from_str(&json_str).unwrap();
                let msg = parsed.message.unwrap();

                // Priority: voice > video > photo
                let expected_type = if has_voice {
                    Some("voice")
                } else if has_video {
                    Some("video")
                } else if has_photo {
                    Some("photo")
                } else {
                    None
                };

                assert_eq!(msg.media_type(), expected_type);
            }
        }
    }
}
