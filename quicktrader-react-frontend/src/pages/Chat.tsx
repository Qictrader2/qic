import React, { useEffect, useState, useCallback, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  PageHeader,
  PrimaryButton,
  Button,
  EmptyState,
  MediaTypeBadge,
  LoadingSpinner,
} from '../components/UI';
import { MarkdownView } from '../components/Markdown';
import {
  getConversations,
  getChatMessages,
  createConversation,
  sendChatMessage,
  renameConversation,
  deleteConversation,
  uploadChatMedia,
} from '../api';
import { useChatWebSocket } from '../hooks/useChatWebSocket';
import { useVoiceRecording } from '../hooks/useVoiceRecording';
import { useVideoRecording } from '../hooks/useVideoRecording';
import { useFileAttachment } from '../hooks/useFileAttachment';
import type {
  Conversation,
  ChatMessage,
  ChatConversationState,
  ChatWsEvent,
  RemoteData,
} from '../types';
import { NotAsked, Loading, Success, Failure, emptyChatConversationState } from '../types';
import { mediaUrl, filenameFromPath } from '../types';
import { colors, fonts, truncateText, formatDateTime } from '../theme';

function conversationDisplayName(c: Conversation): string {
  return c.display_name ?? c.custom_name ?? c.auto_name ?? c.name ?? c.id;
}

export function Chat(): React.ReactElement {
  const { conversationId } = useParams<{ conversationId?: string }>();
  const navigate = useNavigate();
  const activeChatId = conversationId ?? null;

  const [conversations, setConversations] = useState<RemoteData<Conversation[]>>(NotAsked);
  const [conversationStates, setConversationStates] = useState<Record<string, ChatConversationState>>({});
  const [inputText, setInputText] = useState('');
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameText, setRenameText] = useState('');
  const [confirmingDeleteId, setConfirmingDeleteId] = useState<string | null>(null);
  const [contextMenuId, setContextMenuId] = useState<string | null>(null);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const scrollToBottom = () => messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });

  const updateConvState = useCallback((convId: string, updater: (s: ChatConversationState) => ChatConversationState) => {
    setConversationStates((prev) => ({
      ...prev,
      [convId]: updater(prev[convId] ?? emptyChatConversationState),
    }));
  }, []);

  const handleWsMessage = useCallback((event: ChatWsEvent) => {
    if (event.type === 'message_chunk') {
      updateConvState(event.conversation_id, (s) => {
        if (event.is_final) {
          const newMsg: ChatMessage = {
            id: `stream-${event.sequence}`,
            direction: 'inbound',
            content: s.activity.tag === 'streaming' ? s.activity.buffer + event.content : event.content,
            timestamp: new Date().toISOString(),
            attachments: [],
          };
          return {
            ...s,
            messages: [...s.messages, newMsg],
            activity: { tag: 'idle' as const },
          };
        }
        return {
          ...s,
          activity: { tag: 'streaming' as const, buffer: (s.activity.tag === 'streaming' ? s.activity.buffer : '') + event.content },
        };
      });
    } else if (event.type === 'conversation_renamed') {
      setConversations((prev) => {
        if (prev.tag !== 'Success') return prev;
        return Success(prev.data.map((c) => (c.id === event.conversation_id ? { ...c, custom_name: event.name, display_name: event.name } : c)));
      });
    } else if (event.type === 'typing_indicator') {
      updateConvState(event.conversation_id, (s) => ({
        ...s,
        activity: event.is_typing ? { tag: 'observing' as const } : { tag: 'idle' as const },
      }));
    } else if (event.type === 'file_message') {
      updateConvState(event.conversation_id, (s) => {
        const newMsg: ChatMessage = {
          id: event.message_id,
          direction: 'inbound',
          content: event.caption ?? '',
          timestamp: new Date().toISOString(),
          attachments: [{
            type: 'file',
            path: event.media_path,
            name: event.filename,
            mimeType: event.mime_type,
          }],
        };
        return { ...s, messages: [...s.messages, newMsg] };
      });
    }
  }, [updateConvState]);

  const { subscribe, unsubscribe } = useChatWebSocket(handleWsMessage);
  const voice = useVoiceRecording();
  const video = useVideoRecording();
  const fileAttachment = useFileAttachment((file) => {
    if (!activeChatId) return;
    uploadChatMedia(activeChatId, file.data, file.name, file.mimeType)
      .then((res) => {
        if (res.success && res.message_id) {
          updateConvState(activeChatId, (s) => {
            const newMsg: ChatMessage = {
              id: res.message_id!,
              direction: 'outbound',
              content: res.transcription ?? '',
              timestamp: new Date().toISOString(),
              attachments: res.media_path ? [{ type: 'file', path: res.media_path, name: file.name, mimeType: file.mimeType }] : [],
            };
            return { ...s, messages: [...s.messages, newMsg] };
          });
        }
      })
      .catch(() => {});
  });

  const loadConversations = useCallback(async () => {
    setConversations(Loading);
    try {
      const data = await getConversations();
      setConversations(Success(data));
    } catch (e) {
      setConversations(Failure(e instanceof Error ? e.message : 'Failed to load conversations'));
    }
  }, []);

  const loadMessages = useCallback(async (convId: string) => {
    updateConvState(convId, (s) => ({ ...s, messagesLoaded: { tag: 'loading' as const } }));
    try {
      const data = await getChatMessages(convId);
      updateConvState(convId, (s) => ({
        ...s,
        messages: data,
        messagesLoaded: { tag: 'loaded' as const, hasMore: false },
      }));
    } catch (e) {
      updateConvState(convId, (s) => ({
        ...s,
        messagesLoaded: { tag: 'error' as const, message: e instanceof Error ? e.message : 'Failed' },
      }));
    }
  }, [updateConvState]);

  useEffect(() => {
    loadConversations();
  }, [loadConversations]);

  useEffect(() => {
    if (activeChatId) {
      subscribe(activeChatId);
      const state = conversationStates[activeChatId];
      if (!state || state.messagesLoaded.tag === 'not_loaded') {
        loadMessages(activeChatId);
      }
    }
    return () => unsubscribe();
  }, [activeChatId, subscribe, unsubscribe]);

  useEffect(() => {
    scrollToBottom();
  }, [conversationStates, activeChatId]);

  const handleNewChat = useCallback(async () => {
    try {
      const res = await createConversation();
      navigate(`/chat/${res.conversation_id}`);
    } catch (e) {
      console.error(e);
    }
  }, [navigate]);

  const handleSend = useCallback(async () => {
    if (!activeChatId || !inputText.trim()) return;
    const content = inputText.trim();
    setInputText('');
    try {
      await sendChatMessage(activeChatId, content);
      updateConvState(activeChatId, (s) => {
        const newMsg: ChatMessage = {
          id: `pending-${Date.now()}`,
          direction: 'outbound',
          content,
          timestamp: new Date().toISOString(),
          attachments: [],
        };
        return { ...s, messages: [...s.messages, newMsg] };
      });
      scrollToBottom();
    } catch (e) {
      console.error(e);
    }
  }, [activeChatId, inputText, updateConvState]);

  const handleVoiceStop = useCallback(async () => {
    if (!activeChatId) return;
    try {
      const { data, mimeType } = await voice.stop();
      if (data) {
        const res = await uploadChatMedia(activeChatId, data, 'voice.webm', mimeType);
        if (res.success && res.message_id) {
          updateConvState(activeChatId, (s) => {
            const newMsg: ChatMessage = {
              id: res.message_id!,
              direction: 'outbound',
              content: res.transcription ?? '',
              timestamp: new Date().toISOString(),
              attachments: res.media_path ? [{ type: 'audio', path: res.media_path }] : [],
            };
            return { ...s, messages: [...s.messages, newMsg] };
          });
        }
      }
    } catch (e) {
      console.error(e);
    }
  }, [activeChatId, voice, updateConvState]);

  const handleVideoStop = useCallback(async () => {
    if (!activeChatId) return;
    try {
      const { data, mimeType } = await video.stop();
      if (data) {
        const res = await uploadChatMedia(activeChatId, data, 'video.webm', mimeType);
        if (res.success && res.message_id) {
          updateConvState(activeChatId, (s) => {
            const newMsg: ChatMessage = {
              id: res.message_id!,
              direction: 'outbound',
              content: '',
              timestamp: new Date().toISOString(),
              attachments: res.media_path ? [{ type: 'video', path: res.media_path }] : [],
            };
            return { ...s, messages: [...s.messages, newMsg] };
          });
        }
      }
    } catch (e) {
      console.error(e);
    }
  }, [activeChatId, video, updateConvState]);

  const handleRename = useCallback(async () => {
    if (!renamingId || !renameText.trim()) return;
    try {
      await renameConversation(renamingId, renameText.trim());
      setConversations((prev) => {
        if (prev.tag !== 'Success') return prev;
        return Success(prev.data.map((c) => (c.id === renamingId ? { ...c, custom_name: renameText.trim(), display_name: renameText.trim() } : c)));
      });
      setRenamingId(null);
      setRenameText('');
    } catch (e) {
      console.error(e);
    }
  }, [renamingId, renameText]);

  const handleDelete = useCallback(async () => {
    if (!confirmingDeleteId) return;
    try {
      await deleteConversation(confirmingDeleteId);
      setConversations((prev) => {
        if (prev.tag !== 'Success') return prev;
        return Success(prev.data.filter((c) => c.id !== confirmingDeleteId));
      });
      setConversationStates((prev) => {
        const next = { ...prev };
        delete next[confirmingDeleteId];
        return next;
      });
      if (activeChatId === confirmingDeleteId) {
        navigate('/chat');
      }
      setConfirmingDeleteId(null);
    } catch (e) {
      console.error(e);
    }
  }, [confirmingDeleteId, activeChatId, navigate]);

  const convState = activeChatId ? (conversationStates[activeChatId] ?? emptyChatConversationState) : null;
  const messages = convState?.messages ?? [];
  const activity = convState?.activity ?? { tag: 'idle' as const };
  const isStreaming = activity.tag === 'streaming';
  const isTyping = activity.tag === 'observing';
  const streamingContent = activity.tag === 'streaming' ? activity.buffer : '';
  const canSend = !isStreaming && inputText.trim().length > 0;

  const conversationsData = conversations.tag === 'Success' ? conversations.data : null;

  return (
    <div style={{ display: 'flex', minHeight: 'calc(100vh - 120px)' }}>
      <div
        style={{
          width: '280px',
          flexShrink: 0,
          borderRight: `1px solid ${colors.border}`,
          display: 'flex',
          flexDirection: 'column',
          backgroundColor: colors.bgSecondary,
        }}
      >
        <div style={{ padding: '1rem', borderBottom: `1px solid ${colors.border}` }}>
          <PrimaryButton label="New Chat" onClick={handleNewChat} />
        </div>
        <div style={{ flex: 1, overflowY: 'auto' }}>
          {conversations.tag === 'Loading' ? (
            <LoadingSpinner />
          ) : conversations.tag === 'Failure' ? (
            <div style={{ color: colors.error, padding: '1rem', fontSize: '0.75rem' }}>{conversations.error}</div>
          ) : conversationsData ? (
            <div style={{ display: 'flex', flexDirection: 'column' }}>
              {[...conversationsData]
                .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
                .map((c) => (
                  <div
                    key={c.id}
                    onClick={() => navigate(`/chat/${c.id}`)}
                    onContextMenu={(e) => {
                      e.preventDefault();
                      setContextMenuId(c.id);
                    }}
                    style={{
                      padding: '0.75rem 1rem',
                      cursor: 'pointer',
                      backgroundColor: activeChatId === c.id ? colors.bgTertiary : 'transparent',
                      borderBottom: `1px solid ${colors.border}`,
                    }}
                  >
                    {contextMenuId === c.id ? (
                      <div onClick={(e) => e.stopPropagation()} style={{ display: 'flex', gap: '0.5rem' }}>
                        <Button
                          label="Rename"
                          onClick={() => {
                            setRenamingId(c.id);
                            setRenameText(conversationDisplayName(c));
                            setContextMenuId(null);
                          }}
                        />
                        <Button
                          label="Delete"
                          onClick={() => {
                            setConfirmingDeleteId(c.id);
                            setContextMenuId(null);
                          }}
                        />
                      </div>
                    ) : renamingId === c.id ? (
                      <input
                        value={renameText}
                        onChange={(e) => setRenameText(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') handleRename();
                          if (e.key === 'Escape') {
                            setRenamingId(null);
                            setRenameText('');
                            setContextMenuId(null);
                          }
                        }}
                        onClick={(e) => e.stopPropagation()}
                        autoFocus
                        style={{
                          width: '100%',
                          backgroundColor: colors.bgPrimary,
                          color: colors.textPrimary,
                          border: `1px solid ${colors.border}`,
                          padding: '0.25rem 0.5rem',
                          fontFamily: fonts.body,
                          fontSize: '0.875rem',
                        }}
                      />
                    ) : confirmingDeleteId === c.id ? (
                      <div onClick={(e) => e.stopPropagation()} style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        <span style={{ fontSize: '0.75rem', color: colors.textMuted }}>Delete this conversation?</span>
                        <div style={{ display: 'flex', gap: '0.25rem' }}>
                          <Button label="Yes" onClick={handleDelete} />
                          <Button label="No" onClick={() => setConfirmingDeleteId(null)} />
                        </div>
                      </div>
                    ) : (
                      <>
                        <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>{conversationDisplayName(c)}</div>
                        {c.last_message_preview && (
                          <div style={{ fontSize: '0.75rem', color: colors.textMuted, marginTop: '0.25rem' }}>
                            {truncateText(50, c.last_message_preview)}
                          </div>
                        )}
                        <div style={{ fontSize: '0.6875rem', color: colors.textMuted, marginTop: '0.25rem' }}>
                          {formatDateTime(c.updated_at)}
                        </div>
                        {c.protocol === 'telegram' && (
                          <span style={{ fontFamily: fonts.mono, fontSize: '0.5625rem', color: colors.accent }}>TELEGRAM</span>
                        )}
                      </>
                    )}
                  </div>
                ))}
            </div>
          ) : null}
        </div>
      </div>

      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        {!activeChatId ? (
          <EmptyState message="Select a conversation or start a new one" />
        ) : (
          <>
            <div
              style={{
                flex: 1,
                overflowY: 'auto',
                padding: '1rem',
                display: 'flex',
                flexDirection: 'column',
                gap: '1rem',
              }}
            >
              {convState?.messagesLoaded.tag === 'loading' ? (
                <LoadingSpinner />
              ) : (
                <>
                  {messages.map((m) => (
                    <div
                      key={m.id}
                      style={{
                        alignSelf: m.direction === 'outbound' ? 'flex-end' : 'flex-start',
                        maxWidth: '80%',
                        padding: '0.75rem 1rem',
                        borderRadius: '4px',
                        backgroundColor: m.direction === 'outbound' ? colors.accentDim : colors.bgSurface,
                        border: `1px solid ${colors.border}`,
                      }}
                    >
                      <MarkdownView content={m.content || '_No content_'} />
                      {m.attachments.map((a, i) => (
                        <div key={i} style={{ marginTop: '0.5rem' }}>
                          <MediaTypeBadge
                            icon={a.type === 'audio' ? '🎤' : a.type === 'video' ? '🎬' : a.type === 'image' ? '🖼' : '📎'}
                            label={a.type}
                          />
                          {a.path && (
                            <a
                              href={activeChatId ? mediaUrl(activeChatId, filenameFromPath(a.path) ?? a.path) : '#'}
                              target="_blank"
                              rel="noopener noreferrer"
                              style={{ fontSize: '0.75rem', color: colors.accent, marginLeft: '0.25rem' }}
                            >
                              {a.name ?? 'View'}
                            </a>
                          )}
                        </div>
                      ))}
                      <div style={{ fontSize: '0.6875rem', color: colors.textMuted, marginTop: '0.25rem' }}>
                        {formatDateTime(m.timestamp)}
                      </div>
                    </div>
                  ))}
                  {isStreaming && (
                    <div
                      style={{
                        alignSelf: 'flex-start',
                        maxWidth: '80%',
                        padding: '0.75rem 1rem',
                        backgroundColor: colors.bgSurface,
                        border: `1px solid ${colors.border}`,
                        borderRadius: '4px',
                      }}
                    >
                      <MarkdownView content={streamingContent} />
                      <span style={{ animation: 'blink 1s infinite', marginLeft: '2px' }}>|</span>
                    </div>
                  )}
                  {isTyping && (
                    <div style={{ alignSelf: 'flex-start', padding: '0.5rem', color: colors.textMuted }}>
                      <span style={{ animation: 'statusPulse 1.5s infinite' }}>●</span>
                      <span style={{ animation: 'statusPulse 1.5s infinite', animationDelay: '0.2s', marginLeft: '4px' }}>●</span>
                      <span style={{ animation: 'statusPulse 1.5s infinite', animationDelay: '0.4s', marginLeft: '4px' }}>●</span>
                    </div>
                  )}
                  <div ref={messagesEndRef} />
                </>
              )}
            </div>

            <div
              style={{
                padding: '1rem',
                borderTop: `1px solid ${colors.border}`,
                backgroundColor: colors.bgSecondary,
                display: 'flex',
                flexDirection: 'column',
                gap: '0.5rem',
              }}
            >
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'flex-end' }}>
                <textarea
                  value={inputText}
                  onChange={(e) => setInputText(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      handleSend();
                    }
                  }}
                  placeholder="Type a message..."
                  rows={2}
                  style={{
                    flex: 1,
                    backgroundColor: colors.bgPrimary,
                    color: colors.textPrimary,
                    border: `1px solid ${colors.border}`,
                    borderRadius: '2px',
                    padding: '0.5rem 0.75rem',
                    fontFamily: fonts.body,
                    fontSize: '0.875rem',
                    resize: 'none',
                    minHeight: '44px',
                  }}
                />
                <PrimaryButton label="Send" onClick={handleSend} disabled={!canSend} />
                <Button
                  label={voice.isRecording ? '⏹' : '🎤'}
                  onClick={() => (voice.isRecording ? handleVoiceStop() : voice.start())}
                  disabled={video.isRecording}
                />
                <Button
                  label={video.isRecording ? '⏹' : '🎬'}
                  onClick={() => (video.isRecording ? handleVideoStop() : video.start())}
                  disabled={voice.isRecording}
                />
                <Button label="📎" onClick={fileAttachment.trigger} disabled={fileAttachment.pending} />
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
