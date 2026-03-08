import React, { useEffect, useState, useCallback } from 'react';
import {
  PageHeader,
  Card,
  MiniStat,
  StatusBadge,
  LoadingSpinner,
  Button,
  IconButton,
  EmptyState,
} from '../components/UI';
import { getCronJobs, getCronStatus, pauseCronJob, resumeCronJob, cancelCronJob } from '../api';
import type { CronJob, CronStatus } from '../types';
import type { RemoteData } from '../types';
import { NotAsked, Loading, Success, Failure, isLoading } from '../types';
import { colors, fonts, formatDateTime } from '../theme';

function jobStatusColor(status: string): string {
  switch (status.toLowerCase()) {
    case 'active':
      return colors.success;
    case 'paused':
      return colors.warning;
    case 'cancelled':
      return colors.textMuted;
    default:
      return colors.textSecondary;
  }
}

export function CronJobs(): React.ReactElement {
  const [jobs, setJobs] = useState<RemoteData<CronJob[]>>(NotAsked);
  const [status, setStatus] = useState<RemoteData<CronStatus>>(NotAsked);

  const loadJobs = useCallback(async () => {
    try {
      const [jobsData, statusData] = await Promise.all([getCronJobs(), getCronStatus()]);
      setJobs(Success(jobsData));
      setStatus(Success(statusData));
    } catch (e) {
      setJobs(Failure(e instanceof Error ? e.message : 'Failed to load jobs'));
      setStatus(Failure(e instanceof Error ? e.message : 'Failed to load status'));
    }
  }, []);

  useEffect(() => {
    setJobs(Loading);
    setStatus(Loading);
    loadJobs();
  }, [loadJobs]);

  const refresh = useCallback(() => {
    setJobs(Loading);
    setStatus(Loading);
    loadJobs();
  }, [loadJobs]);

  const handlePauseResume = useCallback(
    async (job: CronJob) => {
      try {
        if (job.status === 'active') {
          await pauseCronJob(job.id);
        } else if (job.status === 'paused') {
          await resumeCronJob(job.id);
        }
        loadJobs();
      } catch {
        loadJobs();
      }
    },
    [loadJobs]
  );

  const handleCancel = useCallback(
    async (job: CronJob) => {
      try {
        await cancelCronJob(job.id);
        loadJobs();
      } catch {
        loadJobs();
      }
    },
    [loadJobs]
  );

  const jobsData = jobs.tag === 'Success' ? jobs.data : null;
  const statusData = status.tag === 'Success' ? status.data : null;

  return (
    <div>
      <PageHeader
        title="Scheduled Jobs"
        actions={<Button label="Refresh" onClick={refresh} disabled={isLoading(jobs)} />}
      />
      {statusData && (
        <div
          style={{
            display: 'flex',
            gap: '1rem',
            marginBottom: '1.5rem',
            flexWrap: 'wrap',
          }}
        >
          <MiniStat label="Active" count={statusData.active_jobs} color={colors.success} />
          <MiniStat label="Paused" count={statusData.paused_jobs} color={colors.warning} />
          <MiniStat label="Waiting" count={statusData.waiting_executions} color={colors.accent} />
        </div>
      )}
      {jobs.tag === 'Loading' ? (
        <LoadingSpinner />
      ) : jobs.tag === 'Failure' ? (
        <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
          {jobs.error}
        </div>
      ) : jobsData ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          {jobsData.map((job) => (
            <Card key={job.id}>
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'flex-start',
                  flexWrap: 'wrap',
                  gap: '1rem',
                }}
              >
                <div>
                  <div
                    style={{
                      fontFamily: fonts.display,
                      fontSize: '1rem',
                      fontWeight: 600,
                      color: colors.textPrimary,
                      marginBottom: '0.25rem',
                    }}
                  >
                    {job.name ?? 'Unnamed Job'}
                  </div>
                  <div
                    style={{
                      fontFamily: fonts.mono,
                      fontSize: '0.6875rem',
                      color: colors.textMuted,
                      marginBottom: '0.5rem',
                    }}
                  >
                    {job.schedule}
                  </div>
                  <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                    <StatusBadge status={job.status} />
                    {job.next_run && (
                      <span
                        style={{
                          fontFamily: fonts.mono,
                          fontSize: '0.6875rem',
                          color: colors.textSecondary,
                        }}
                      >
                        next: {formatDateTime(job.next_run)}
                      </span>
                    )}
                    {job.last_run && (
                      <span
                        style={{
                          fontFamily: fonts.mono,
                          fontSize: '0.6875rem',
                          color: colors.textMuted,
                        }}
                      >
                        last: {formatDateTime(job.last_run)}
                      </span>
                    )}
                  </div>
                </div>
                <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                  {(job.status === 'active' || job.status === 'paused') && (
                    <Button
                      label={job.status === 'active' ? 'Pause' : 'Resume'}
                      onClick={() => handlePauseResume(job)}
                    />
                  )}
                  <IconButton
                    icon="✕"
                    onClick={() => handleCancel(job)}
                    title="Cancel"
                  />
                </div>
              </div>
            </Card>
          ))}
          {jobsData.length === 0 && <EmptyState message="No cron jobs" />}
        </div>
      ) : null}
    </div>
  );
}
