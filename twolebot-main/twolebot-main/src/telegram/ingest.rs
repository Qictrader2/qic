use crate::error::Result;
use crate::logging::SharedLogger;
use crate::storage::{
    ActiveChatRegistry, ChatMetadataStore, MainTopicStore, MediaStore, MessageStore, PromptFeed,
    PromptItem, PromptSource, Protocol, SettingsStore, StoredMessage,
};
use crate::telegram::{TelegramPoller, TelegramSender, Update};
use crate::transcription::GeminiTranscriber;
use std::sync::Arc;

/// Process a single Telegram update into the prompt feed (+ message/media persistence).
pub async fn process_update(
    update: &Update,
    prompt_feed: &Arc<PromptFeed>,
    message_store: &Arc<MessageStore>,
    media_store: &Arc<MediaStore>,
    active_chats: &Arc<ActiveChatRegistry>,
    telegram_sender: &Arc<TelegramSender>,
    telegram_poller: &Arc<TelegramPoller>,
    gemini: Option<&Arc<GeminiTranscriber>>,
    main_topic_store: &Arc<MainTopicStore>,
    logger: &SharedLogger,
    settings_store: &Arc<SettingsStore>,
    chat_metadata_store: &Arc<ChatMetadataStore>,
) -> Result<()> {
    let Some(ref msg) = update.message else {
        return Ok(());
    };

    let chat_id = msg.chat.id;
    let user_id = msg.user_id().unwrap_or(0);
    let message_id = msg.message_id;

    // Enforce allowed username whitelist (mandatory — None blocks all messages)
    let allowed_username = settings_store.get().allowed_username;
    match allowed_username {
        Some(ref allowed) => {
            let sender_username = msg.from.as_ref().and_then(|u| u.username.as_deref());
            // Strip leading @ from both sides for robust comparison
            let allowed_bare = allowed.strip_prefix('@').unwrap_or(allowed);
            let matches = sender_username
                .map(|u| {
                    let u_bare = u.strip_prefix('@').unwrap_or(u);
                    u_bare.eq_ignore_ascii_case(allowed_bare)
                })
                .unwrap_or(false);
            if !matches {
                logger.info(
                    "telegram",
                    format!(
                        "Ignoring message {} from user {:?} (allowed: @{})",
                        message_id,
                        sender_username.unwrap_or("unknown"),
                        allowed_bare
                    ),
                );
                return Ok(());
            }
        }
        None => {
            logger.info(
                "telegram",
                format!(
                    "Ignoring message {} — no allowed username configured",
                    message_id
                ),
            );
            return Ok(());
        }
    }

    // Track this as the active Telegram chat for routing responses/typing back to the user
    active_chats.set_active(user_id, Protocol::Telegram, chat_id.to_string())?;

    let thread_id = msg.effective_thread_id();
    logger.info(
        "telegram",
        format!(
            "Received message {} from chat {} (thread: {:?}, msg_thread_id: {:?}, dm_topic: {:?})",
            message_id, chat_id, thread_id,
            msg.message_thread_id,
            msg.direct_messages_topic.as_ref().map(|t| t.topic_id),
        ),
    );

    // Handle media (voice, video, photo, animation, etc.)
    let (prompt_text, media_path) = if msg.has_media() {
        // Check if Gemini is available for transcription
        let Some(gemini) = gemini else {
            // No Gemini API key - inform user
            telegram_sender
                .send_message(
                    chat_id,
                    msg.effective_thread_id(),
                    "Voice/media transcription is not available. Configure Gemini in Setup/Settings to enable this feature.",
                )
                .await?;
            return Ok(());
        };

        let media_type = msg.media_type().unwrap_or("unknown");
        let file_id = msg.media_file_id().unwrap_or("");

        logger.info(
            "telegram",
            format!("Processing {} media: {}", media_type, file_id),
        );

        // Download media
        let media_data = telegram_poller.download_file_by_id(file_id).await?;

        // Determine file extension based on media type
        // Use MIME type from message if available, otherwise default based on type
        let extension = match media_type {
            "voice" => "ogg",
            "audio" => msg
                .media_mime_type()
                .and_then(mime_to_extension)
                .unwrap_or("mp3"),
            "video" | "video_note" => "mp4",
            "photo" => "jpg",
            "animation" => msg
                .media_mime_type()
                .and_then(mime_to_extension)
                .unwrap_or("mp4"), // GIFs are usually converted to MP4 by Telegram
            "document" => msg
                .media_mime_type()
                .and_then(mime_to_extension)
                .unwrap_or("bin"),
            "sticker" => {
                // Check if it's animated/video sticker
                if let Some(ref sticker) = msg.sticker {
                    if sticker.is_video {
                        "webm"
                    } else if sticker.is_animated {
                        "tgs" // Telegram animated sticker format
                    } else {
                        "webp"
                    }
                } else {
                    "webp"
                }
            }
            _ => "bin",
        };
        let filename = format!("{}_{}.{}", message_id, media_type, extension);
        let media_path = media_store.store(&chat_id.to_string(), &filename, &media_data)?;

        // Get MIME type for Gemini - use from message or infer from media type
        let mime_type = msg
            .media_mime_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| media_type_to_mime(media_type).to_string());

        // Transcribe/describe using proper MIME type
        let transcription = gemini
            .transcribe_with_mime(&media_data, media_type, &mime_type)
            .await?;

        // Combine with caption if present
        let prompt = if let Some(caption) = msg.caption.as_deref() {
            format!(
                "[{}]\n\nTranscription: {}\n\nUser message: {}",
                media_type, transcription, caption
            )
        } else {
            format!("[{}]\n\nTranscription: {}", media_type, transcription)
        };

        (prompt, Some(media_path.to_string_lossy().to_string()))
    } else if msg.has_special_content() {
        // Handle special content types (contact, location, poll, etc.)
        let content_type = msg.special_content_type().unwrap_or("unknown");
        let description = describe_special_content(msg);

        logger.info("telegram", format!("Processing {} content", content_type));

        let prompt = if let Some(caption) = msg.caption.as_deref() {
            format!(
                "[{}]\n\n{}\n\nUser message: {}",
                content_type, description, caption
            )
        } else {
            format!("[{}]\n\n{}", content_type, description)
        };

        (prompt, None)
    } else if let Some(text) = msg.text.as_deref() {
        // /start is sent when a user first opens the bot or taps Start after a topic is deleted.
        // Acknowledge it without spawning an agent session.
        if text.trim() == "/start" {
            telegram_sender
                .send_message(chat_id, msg.effective_thread_id(), "Ready.")
                .await?;
            return Ok(());
        }
        (text.to_string(), None)
    } else {
        // No text or media we can handle
        return Ok(());
    };

    // Store inbound message (with topic_id)
    let stored_msg = StoredMessage::inbound(
        format!("tg-{}-{}", chat_id, message_id),
        chat_id.to_string(),
        user_id,
        &prompt_text,
    )
    .with_topic_id(thread_id);
    let stored_msg = if let Some(ref path) = media_path {
        stored_msg.with_media(msg.media_type().unwrap_or("unknown"), path)
    } else {
        stored_msg
    };
    message_store.store(stored_msg.with_telegram_id(message_id))?;

    // Persist chat display metadata (username, display name) for the UI
    let sender_username = msg.from.as_ref().and_then(|u| u.username.as_deref());
    let display_name = msg
        .chat
        .title
        .as_deref()
        .or(msg.chat.first_name.as_deref())
        .or(msg.from.as_ref().map(|u| u.first_name.as_str()));
    if let Err(e) = chat_metadata_store.upsert_full(
        &chat_id.to_string(),
        thread_id,
        sender_username,
        display_name,
        Some("telegram"),
        None,
    ) {
        tracing::warn!("Failed to upsert chat metadata for chat {}: {}", chat_id, e);
    }

    // Set reaction to show we received it
    let _ = telegram_sender
        .set_reaction(chat_id, message_id, "👀")
        .await;

    // Record this topic as the "main" topic for the chat so cron/autowork
    // can route into it later. Only stores the first one observed per chat.
    if let Some(tid) = thread_id {
        if let Err(e) = main_topic_store.set_if_absent(chat_id, tid) {
            tracing::warn!("Failed to record main topic for chat {}: {}", chat_id, e);
        }
    }

    let source = PromptSource::telegram(
        update.update_id,
        message_id,
        chat_id,
        thread_id,
    );
    let mut prompt_item = PromptItem::new(source, user_id, prompt_text);
    if let Some(path) = media_path {
        prompt_item = prompt_item.with_media(path);
    }

    // Enqueue prompt
    prompt_feed.enqueue(prompt_item)?;

    logger.info("telegram", format!("Queued prompt for chat {}", chat_id));

    // Typing indicator automatically activates when prompt is marked running
    // and stops when it leaves the running/ directory

    Ok(())
}

/// Convert MIME type to file extension
fn mime_to_extension(mime: &str) -> Option<&'static str> {
    match mime {
        // Audio
        "audio/ogg" | "audio/opus" => Some("ogg"),
        "audio/mpeg" | "audio/mp3" => Some("mp3"),
        "audio/mp4" | "audio/m4a" => Some("m4a"),
        "audio/wav" | "audio/x-wav" => Some("wav"),
        "audio/flac" => Some("flac"),
        "audio/aac" => Some("aac"),
        // Video
        "video/mp4" => Some("mp4"),
        "video/webm" => Some("webm"),
        "video/quicktime" => Some("mov"),
        "video/x-msvideo" => Some("avi"),
        "video/x-matroska" => Some("mkv"),
        // Images
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/bmp" => Some("bmp"),
        "image/tiff" => Some("tiff"),
        // Documents
        "application/pdf" => Some("pdf"),
        "text/plain" => Some("txt"),
        "application/json" => Some("json"),
        "application/xml" | "text/xml" => Some("xml"),
        "application/zip" => Some("zip"),
        // Telegram specific
        "application/x-tgsticker" => Some("tgs"),
        _ => None,
    }
}

/// Convert media type to default MIME type for Gemini
fn media_type_to_mime(media_type: &str) -> &'static str {
    match media_type {
        "voice" => "audio/ogg",
        "audio" => "audio/mpeg",
        "video" | "video_note" => "video/mp4",
        "photo" => "image/jpeg",
        "animation" => "video/mp4", // Telegram converts GIFs to MP4
        "document" => "application/octet-stream",
        "sticker" => "image/webp",
        _ => "application/octet-stream",
    }
}

/// Describe special content (contact, location, poll, etc.) as text
fn describe_special_content(msg: &crate::telegram::types::Message) -> String {
    if let Some(ref contact) = msg.contact {
        let mut desc = format!("Contact shared:\nName: {}", contact.first_name);
        if let Some(ref last) = contact.last_name {
            desc.push_str(&format!(" {}", last));
        }
        desc.push_str(&format!("\nPhone: {}", contact.phone_number));
        if let Some(user_id) = contact.user_id {
            desc.push_str(&format!("\nTelegram User ID: {}", user_id));
        }
        return desc;
    }

    if let Some(ref venue) = msg.venue {
        return format!(
            "Venue shared:\nName: {}\nAddress: {}\nLocation: {}, {}",
            venue.title, venue.address, venue.location.latitude, venue.location.longitude
        );
    }

    if let Some(ref location) = msg.location {
        return format!(
            "Location shared:\nLatitude: {}\nLongitude: {}",
            location.latitude, location.longitude
        );
    }

    if let Some(ref poll) = msg.poll {
        let options: Vec<String> = poll
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| format!("{}. {} ({} votes)", i + 1, opt.text, opt.voter_count))
            .collect();
        return format!(
            "Poll: {}\nType: {}\nOptions:\n{}",
            poll.question,
            poll.poll_type,
            options.join("\n")
        );
    }

    if let Some(ref dice) = msg.dice {
        return format!("Dice {} rolled: {}", dice.emoji, dice.value);
    }

    if let Some(ref game) = msg.game {
        return format!("Game: {}\n{}", game.title, game.description);
    }

    "Unknown special content".to_string()
}
