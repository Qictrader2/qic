import React, { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  PageHeader,
  BackButton,
  InputField,
  PageInfo,
  AccentedItem,
  RoleBadge,
  MediaTypeBadge,
  LoadingSpinner,
  EmptyState,
} from '../components/UI';
import { getChats, getMessages } from '../api';
import type { ChatSummary, MessagesPage, StoredMessage } from '../types';
import type { RemoteData } from '../types';
import { NotAsked, Loading, Success, Failure, isLoading } from '../types';
import { colors, fonts, truncateText, formatDateTime } from '../theme';
import { Pagination as PaginationComponent } from '../components/Pagination';

const PAGE_SIZE = 20;

function chatDisplayName(chat: ChatSummary): string {
  return chat.display_name ?? chat.username ?? chat.chat_id;
}

export function Messages(): React.ReactElement {
  const { chatId } = useParams<{ chatId: string }>();
  const navigate = useNavigate();
  const [chats, setChats] = useState<RemoteData<ChatSummary[]>>(NotAsked);
  const [messages, setMessages] = useState<RemoteData<MessagesPage>>(NotAsked);
  const [search, setSearch] = useState('');
  const [page, setPage] = useState(0);

  const loadChats = useCallback(async () => {
    try {
      const data = await getChats();
      setChats(Success(data));
    } catch (e) {
      setChats(Failure(e instanceof Error ? e.message : 'Failed to load chats'));
    }
  }, []);

  const loadMessages = useCallback(async () => {
    if (!chatId) return;
    setMessages(Loading);
    try {
      const data = await getMessages(chatId, page, PAGE_SIZE, search || undefined);
      setMessages(Success(data));
    } catch (e) {
      setMessages(Failure(e instanceof Error ? e.message : 'Failed to load messages'));
    }
  }, [chatId, page, search]);

  useEffect(() => {
    loadChats();
  }, [loadChats]);

  useEffect(() => {
    if (chatId) {
      loadMessages();
    } else {
      setMessages(NotAsked);
    }
  }, [chatId, loadMessages]);

  const messagesData = messages.tag === 'Success' ? messages.data : null;
  const chatsData = chats.tag === 'Success' ? chats.data : null;

  if (!chatId) {
    return (
      <div>
        <PageHeader title="Messages" />
        {chats.tag === 'Loading' || chats.tag === 'NotAsked' ? (
          <LoadingSpinner />
        ) : chats.tag === 'Failure' ? (
          <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
            {chats.error}
          </div>
        ) : chatsData ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            {chatsData.map((chat) => (
              <div
                key={chat.chat_id}
                onClick={() => navigate(`/messages/${chat.chat_id}`)}
                style={{
                  padding: '1rem 1.25rem',
                  backgroundColor: colors.bgSurface,
                  border: `1px solid ${colors.border}`,
                  borderRadius: '4px',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  flexWrap: 'wrap',
                  gap: '0.5rem',
                }}
              >
                <span style={{ fontFamily: fonts.body, fontSize: '0.875rem', color: colors.textPrimary }}>
                  {chatDisplayName(chat)}
                </span>
                <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                  <span
                    style={{
                      fontFamily: fonts.mono,
                      fontSize: '0.625rem',
                      padding: '0.25rem 0.5rem',
                      backgroundColor: colors.bgTertiary,
                      borderRadius: '2px',
                      color: colors.textSecondary,
                    }}
                  >
                    {chat.message_count} msgs
                  </span>
                  {chat.topic_id != null && (
                    <span
                      style={{
                        fontFamily: fonts.mono,
                        fontSize: '0.625rem',
                        padding: '0.25rem 0.5rem',
                        backgroundColor: colors.accentDim,
                        borderRadius: '2px',
                        color: colors.accent,
                      }}
                    >
                      topic {chat.topic_id}
                    </span>
                  )}
                </div>
              </div>
            ))}
            {chatsData.length === 0 && <EmptyState message="No chats" />}
          </div>
        ) : null}
      </div>
    );
  }

  const selectedChat = chatsData?.find((c) => c.chat_id === chatId);
  const chatName = selectedChat ? chatDisplayName(selectedChat) : chatId;

  return (
    <div>
      <BackButton onClick={() => navigate('/messages')} />
      <PageHeader title={chatName} />
      <div style={{ marginBottom: '1rem' }}>
        <InputField
          value={search}
          onChange={setSearch}
          placeholder="Search messages..."
        />
      </div>
      {messages.tag === 'Loading' ? (
        <LoadingSpinner />
      ) : messages.tag === 'Failure' ? (
        <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
          {messages.error}
        </div>
      ) : messagesData ? (
        <>
          <PageInfo
            page={page}
            pageSize={PAGE_SIZE}
            total={messagesData.total}
            totalPages={messagesData.total_pages}
          />
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            {messagesData.messages.map((msg) => (
              <MessageItem key={msg.id} msg={msg} />
            ))}
            {messagesData.messages.length === 0 && (
              <EmptyState message="No messages" />
            )}
          </div>
          <PaginationComponent
            page={page}
            totalPages={messagesData.total_pages}
            onPageChange={setPage}
          />
        </>
      ) : null}
    </div>
  );
}

function MessageItem({ msg }: { msg: StoredMessage }): React.ReactElement {
  const isInbound = msg.direction === 'inbound';
  const borderColor = isInbound ? colors.accent : colors.success;

  return (
    <AccentedItem borderColor={borderColor}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
        <RoleBadge
          label={isInbound ? 'USER' : 'BOT'}
          color={isInbound ? colors.accent : colors.success}
        />
        {msg.media_type && (
          <MediaTypeBadge
            icon={mediaIcon(msg.media_type)}
            label={msg.media_type}
          />
        )}
        <span
          style={{
            fontFamily: fonts.mono,
            fontSize: '0.6875rem',
            color: colors.textMuted,
            marginLeft: 'auto',
          }}
        >
          {formatDateTime(msg.timestamp)}
        </span>
      </div>
      <div style={{ fontSize: '0.875rem', color: colors.textPrimary }}>
        {truncateText(200, msg.content) || '(no text)'}
      </div>
    </AccentedItem>
  );
}

function mediaIcon(mediaType: string): string {
  if (mediaType.startsWith('audio') || mediaType === 'voice') return '🎤';
  if (mediaType.startsWith('video')) return '🎬';
  if (mediaType.startsWith('image') || mediaType === 'photo') return '🖼';
  return '📎';
}
