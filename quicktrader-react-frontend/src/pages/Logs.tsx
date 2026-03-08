import React, { useEffect, useState, useCallback } from 'react';
import {
  PageHeader,
  InputField,
  PageInfo,
  LoadingSpinner,
  Button,
  PillBadge,
} from '../components/UI';
import { getLogs } from '../api';
import type { LogsPage, LogEntry } from '../types';
import type { RemoteData } from '../types';
import { NotAsked, Loading, Success, Failure, isLoading } from '../types';
import { colors, fonts, formatDateTime } from '../theme';
import { Pagination } from '../components/Pagination';

const PAGE_SIZE = 50;

function levelBadgeStyle(level: string): { bgColor: string; textColor: string } {
  switch (level.toLowerCase()) {
    case 'debug':
      return { bgColor: colors.borderLight, textColor: colors.textMuted };
    case 'info':
      return { bgColor: colors.accentDim, textColor: colors.accent };
    case 'warn':
      return { bgColor: 'rgba(251, 191, 36, 0.12)', textColor: colors.warning };
    case 'error':
      return { bgColor: colors.errorDim, textColor: colors.error };
    default:
      return { bgColor: colors.borderLight, textColor: colors.textMuted };
  }
}

export function Logs(): React.ReactElement {
  const [logs, setLogs] = useState<RemoteData<LogsPage>>(NotAsked);
  const [search, setSearch] = useState('');
  const [page, setPage] = useState(0);

  const loadLogs = useCallback(async () => {
    setLogs(Loading);
    try {
      const data = await getLogs(page, PAGE_SIZE, search || undefined);
      setLogs(Success(data));
    } catch (e) {
      setLogs(Failure(e instanceof Error ? e.message : 'Failed to load logs'));
    }
  }, [page, search]);

  useEffect(() => {
    loadLogs();
  }, [loadLogs]);

  const refresh = useCallback(() => {
    loadLogs();
  }, [loadLogs]);

  const logsData = logs.tag === 'Success' ? logs.data : null;

  return (
    <div>
      <PageHeader
        title="Logs"
        actions={<Button label="Refresh" onClick={refresh} disabled={isLoading(logs)} />}
      />
      <div style={{ marginBottom: '1rem' }}>
        <InputField
          value={search}
          onChange={setSearch}
          placeholder="Search logs..."
        />
      </div>
      {logs.tag === 'Loading' ? (
        <LoadingSpinner />
      ) : logs.tag === 'Failure' ? (
        <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
          {logs.error}
        </div>
      ) : logsData ? (
        <>
          <PageInfo
            page={page}
            pageSize={PAGE_SIZE}
            total={logsData.total}
            totalPages={logsData.total_pages}
          />
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
            {logsData.entries.map((entry) => (
              <LogRow key={`${entry.timestamp}-${entry.component}-${entry.message}`} entry={entry} />
            ))}
          </div>
          <Pagination
            page={page}
            totalPages={logsData.total_pages}
            onPageChange={setPage}
          />
        </>
      ) : null}
    </div>
  );
}

function LogRow({ entry }: { entry: LogEntry }): React.ReactElement {
  const { bgColor, textColor } = levelBadgeStyle(entry.level);

  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '140px 80px 120px 1fr',
        gap: '1rem',
        alignItems: 'center',
        padding: '0.5rem 1rem',
        backgroundColor: colors.bgSurface,
        border: `1px solid ${colors.border}`,
        borderRadius: '4px',
        fontFamily: fonts.mono,
        fontSize: '0.8125rem',
      }}
    >
      <span style={{ color: colors.textMuted }}>{formatDateTime(entry.timestamp)}</span>
      <PillBadge bgColor={bgColor} textColor={textColor} label={entry.level.toUpperCase()} />
      <span style={{ color: colors.textSecondary }}>{entry.component}</span>
      <span style={{ color: colors.textPrimary }}>{entry.message}</span>
    </div>
  );
}
