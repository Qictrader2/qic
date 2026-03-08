import React, { useEffect, useState, useCallback } from 'react';
import {
  PageHeader,
  Card,
  CardWithHeader,
  StatCard,
  GridTwo,
  StatusBadge,
  LoadingSpinner,
  Button,
  EmptyState,
} from '../components/UI';
import {
  getFeed,
  getResponses,
  getSemanticStatus,
  getTunnelStatus,
  toggleSemantic,
  triggerSemanticReindex,
} from '../api';
import type { FeedData, ResponseFeedData, SemanticStatus, TunnelStatus } from '../types';
import type { RemoteData } from '../types';
import { NotAsked, Loading, Success, Failure, isLoading } from '../types';
import { usePolling } from '../hooks/usePolling';
import { colors, fonts, truncateText, formatDateTime } from '../theme';

const POLL_INTERVAL_MS = 10000;

function truncateId(id: string): string {
  return id.length > 12 ? id.slice(0, 8) + '…' : id;
}

export function Dashboard(): React.ReactElement {
  const [feed, setFeed] = useState<RemoteData<FeedData>>(NotAsked);
  const [responses, setResponses] = useState<RemoteData<ResponseFeedData>>(NotAsked);
  const [semantic, setSemantic] = useState<RemoteData<SemanticStatus>>(NotAsked);
  const [tunnel, setTunnel] = useState<RemoteData<TunnelStatus>>(NotAsked);

  const loadFeed = useCallback(async () => {
    setFeed((prev) => (prev.tag === 'NotAsked' ? Loading : prev));
    try {
      const data = await getFeed();
      setFeed(Success(data));
    } catch (e) {
      setFeed(Failure(e instanceof Error ? e.message : 'Failed to load feed'));
    }
  }, []);

  const loadResponses = useCallback(async () => {
    setResponses((prev) => (prev.tag === 'NotAsked' ? Loading : prev));
    try {
      const data = await getResponses();
      setResponses(Success(data));
    } catch (e) {
      setResponses(Failure(e instanceof Error ? e.message : 'Failed to load responses'));
    }
  }, []);

  const loadSemantic = useCallback(async () => {
    setSemantic((prev) => (prev.tag === 'NotAsked' ? Loading : prev));
    try {
      const data = await getSemanticStatus();
      setSemantic(Success(data));
    } catch (e) {
      setSemantic(Failure(e instanceof Error ? e.message : 'Failed to load semantic status'));
    }
  }, []);

  const loadTunnel = useCallback(async () => {
    setTunnel((prev) => (prev.tag === 'NotAsked' ? Loading : prev));
    try {
      const data = await getTunnelStatus();
      setTunnel(Success(data));
    } catch (e) {
      setTunnel(Failure(e instanceof Error ? e.message : 'Failed to load tunnel status'));
    }
  }, []);

  useEffect(() => {
    loadFeed();
    loadResponses();
    loadSemantic();
    loadTunnel();
  }, [loadFeed, loadResponses, loadSemantic, loadTunnel]);

  usePolling(loadFeed, POLL_INTERVAL_MS);
  usePolling(loadResponses, POLL_INTERVAL_MS);

  const refresh = useCallback(() => {
    setFeed(Loading);
    setResponses(Loading);
    setSemantic(Loading);
    setTunnel(Loading);
    loadFeed();
    loadResponses();
    loadSemantic();
    loadTunnel();
  }, [loadFeed, loadResponses, loadSemantic, loadTunnel]);

  const feedData = feed.tag === 'Success' ? feed.data : null;
  const responsesData = responses.tag === 'Success' ? responses.data : null;
  const semanticData = semantic.tag === 'Success' ? semantic.data : null;
  const tunnelData = tunnel.tag === 'Success' ? tunnel.data : null;

  const loading = isLoading(feed) || isLoading(responses) || isLoading(semantic) || isLoading(tunnel);

  return (
    <div>
      <PageHeader
        title="Dashboard"
        actions={
          <Button
            label="Refresh"
            onClick={refresh}
            disabled={loading}
          />
        }
      />
      {loading && feed.tag === 'NotAsked' && responses.tag === 'NotAsked' ? (
        <LoadingSpinner />
      ) : (
        <GridTwo>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
            <CardWithHeader title="Queue Status">
              {feedData ? (
                <div
                  style={{
                    display: 'grid',
                    gridTemplateColumns: 'repeat(auto-fit, minmax(100px, 1fr))',
                    gap: '1rem',
                  }}
                >
                  <StatCard
                    label="Pending"
                    value={String(feedData.pending_count)}
                    accent={colors.textMuted}
                  />
                  <StatCard
                    label="Running"
                    value={feedData.running ? '1' : '0'}
                    accent={colors.warning}
                  />
                  <StatCard
                    label="Completed"
                    value={String(feedData.completed_count)}
                    accent={colors.success}
                  />
                </div>
              ) : feed.tag === 'Failure' ? (
                <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
                  {feed.error}
                </div>
              ) : null}
            </CardWithHeader>

            <CardWithHeader title="Feed">
              {feedData ? (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                  {feedData.running && (
                    <div
                      style={{
                        padding: '1rem',
                        backgroundColor: colors.warningDim,
                        border: `1px solid ${colors.warning}`,
                        borderRadius: '4px',
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
                        <span style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textMuted }}>
                          {truncateId(feedData.running.id)}
                        </span>
                        <StatusBadge status="running" />
                      </div>
                      <div style={{ fontSize: '0.875rem', color: colors.textPrimary }}>
                        {truncateText(80, feedData.running.prompt)}
                      </div>
                      <div style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textMuted, marginTop: '0.25rem' }}>
                        {formatDateTime(feedData.running.created_at)}
                      </div>
                    </div>
                  )}
                  {feedData.pending.map((item) => (
                    <div
                      key={item.id}
                      style={{
                        padding: '0.75rem 1rem',
                        backgroundColor: colors.bgSurface,
                        border: `1px solid ${colors.border}`,
                        borderRadius: '4px',
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.25rem' }}>
                        <span style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textMuted }}>
                          {truncateId(item.id)}
                        </span>
                        <StatusBadge status={item.status} />
                      </div>
                      <div style={{ fontSize: '0.875rem', color: colors.textPrimary }}>
                        {truncateText(80, item.prompt)}
                      </div>
                      <div style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textMuted, marginTop: '0.25rem' }}>
                        {formatDateTime(item.created_at)}
                      </div>
                    </div>
                  ))}
                  {!feedData.running && feedData.pending.length === 0 && (
                    <EmptyState message="No pending items" />
                  )}
                </div>
              ) : feed.tag === 'Failure' ? (
                <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
                  {feed.error}
                </div>
              ) : null}
            </CardWithHeader>

            <CardWithHeader title="Responses">
              {responsesData ? (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                  <div
                    style={{
                      display: 'grid',
                      gridTemplateColumns: 'repeat(3, 1fr)',
                      gap: '0.5rem',
                      marginBottom: '0.5rem',
                    }}
                  >
                    <span style={{ fontFamily: fonts.mono, fontSize: '0.625rem', color: colors.textMuted }}>
                      Pending: {responsesData.pending_count}
                    </span>
                    <span style={{ fontFamily: fonts.mono, fontSize: '0.625rem', color: colors.success }}>
                      Sent: {responsesData.sent_count}
                    </span>
                    <span style={{ fontFamily: fonts.mono, fontSize: '0.625rem', color: colors.error }}>
                      Failed: {responsesData.failed_count}
                    </span>
                  </div>
                  {[...responsesData.recent_sent, ...responsesData.recent_failed].slice(0, 10).map((item) => (
                    <div
                      key={item.id}
                      style={{
                        padding: '0.75rem 1rem',
                        backgroundColor: colors.bgSurface,
                        border: `1px solid ${colors.border}`,
                        borderRadius: '4px',
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.25rem' }}>
                        <span style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textMuted }}>
                          {truncateId(item.id)}
                        </span>
                        <StatusBadge status={item.status} />
                      </div>
                      <div style={{ fontSize: '0.875rem', color: colors.textPrimary }}>
                        {truncateText(80, item.content)}
                      </div>
                      <div style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textMuted, marginTop: '0.25rem' }}>
                        {formatDateTime(item.created_at)}
                      </div>
                    </div>
                  ))}
                  {responsesData.recent_sent.length === 0 &&
                    responsesData.recent_failed.length === 0 &&
                    responsesData.pending_count === 0 && (
                      <EmptyState message="No recent responses" />
                    )}
                </div>
              ) : responses.tag === 'Failure' ? (
                <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
                  {responses.error}
                </div>
              ) : null}
            </CardWithHeader>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
            <CardWithHeader title="Semantic Indexer">
              {semanticData ? (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                    <span style={{ fontFamily: fonts.mono, fontSize: '0.75rem', color: colors.textSecondary }}>
                      Status: {semanticData.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                    <Button
                      label={semanticData.enabled ? 'Disable' : 'Enable'}
                      onClick={async () => {
                        try {
                          const updated = await toggleSemantic(!semanticData.enabled);
                          setSemantic(Success(updated));
                        } catch {
                          loadSemantic();
                        }
                      }}
                    />
                  </div>
                  <div style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textMuted }}>
                    Memory: {semanticData.total_memory_chunks} chunks, {semanticData.total_memory_files} files
                  </div>
                  <div style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textMuted }}>
                    Conversations: {semanticData.total_conversation_chunks} chunks, {semanticData.total_conversation_sessions} sessions
                  </div>
                  <Button
                    label="Reindex"
                    onClick={async () => {
                      try {
                        await triggerSemanticReindex();
                        loadSemantic();
                      } catch {
                        loadSemantic();
                      }
                    }}
                  />
                </div>
              ) : semantic.tag === 'Failure' ? (
                <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
                  {semantic.error}
                </div>
              ) : null}
            </CardWithHeader>

            <CardWithHeader title="Tunnel">
              {tunnelData ? (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                    <span
                      style={{
                        width: '8px',
                        height: '8px',
                        borderRadius: '50%',
                        backgroundColor: tunnelData.active ? colors.success : colors.textMuted,
                      }}
                    />
                    <span style={{ fontFamily: fonts.mono, fontSize: '0.75rem', color: colors.textSecondary }}>
                      {tunnelData.active ? 'Active' : 'Inactive'}
                    </span>
                  </div>
                  {tunnelData.url && (
                    <a
                      href={tunnelData.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      style={{
                        fontFamily: fonts.mono,
                        fontSize: '0.75rem',
                        color: colors.accent,
                        wordBreak: 'break-all',
                      }}
                    >
                      {tunnelData.url}
                    </a>
                  )}
                  {tunnelData.qr_svg && (
                    <div
                      dangerouslySetInnerHTML={{ __html: tunnelData.qr_svg }}
                      style={{ maxWidth: '200px', marginTop: '0.5rem' }}
                    />
                  )}
                </div>
              ) : tunnel.tag === 'Failure' ? (
                <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
                  {tunnel.error}
                </div>
              ) : null}
            </CardWithHeader>
          </div>
        </GridTwo>
      )}
    </div>
  );
}
