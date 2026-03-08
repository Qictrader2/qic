import React, { useEffect, useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  PageHeader,
  SectionHeader,
  Card,
  PrimaryButton,
  Button,
  MiniStat,
  PillBadge,
  TagChip,
  LoadingSpinner,
} from '../components/UI';
import {
  getLiveBoard,
  listProjects,
  selectTasks,
  deselectTask,
  clearCompleted,
  moveSelection,
  startAgentLoop,
  stopAgentLoop,
  ensureAgentLoop,
} from '../api';
import { useWorkEvents } from '../hooks/useWorkEvents';
import type { LiveBoard as LiveBoardType, LiveBoardStats, WorkTask, RemoteData } from '../types';
import { NotAsked, Loading, Success, Failure } from '../types';
import { colors, fonts, taskStatusColor, taskStatusLabel, taskPriorityLabel, truncateText } from '../theme';

function priorityBadgeColor(priority: string): string {
  switch (priority) {
    case 'critical': return colors.error;
    case 'high': return colors.warning;
    case 'medium': return colors.accent;
    case 'low': return colors.textMuted;
    default: return colors.textMuted;
  }
}

export function LiveBoard(): React.ReactElement {
  const navigate = useNavigate();
  const [liveBoard, setLiveBoard] = useState<RemoteData<LiveBoardType>>(NotAsked);
  const [projects, setProjects] = useState<RemoteData<import('../types').WorkProject[]>>(NotAsked);
  const [busy, setBusy] = useState(false);
  const [selectedBacklogIds, setSelectedBacklogIds] = useState<Set<number>>(new Set());

  const [stats, setStats] = useState<LiveBoardStats | null>(null);

  const loadLiveBoard = useCallback(async () => {
    setLiveBoard(Loading);
    try {
      const data = await getLiveBoard();
      setLiveBoard(Success(data));
      setStats(data.stats);
    } catch (e) {
      setLiveBoard(Failure(e instanceof Error ? e.message : 'Failed to load live board'));
    }
  }, []);

  const loadProjects = useCallback(async () => {
    try {
      const data = await listProjects();
      setProjects(Success(data));
    } catch (e) {
      setProjects(Failure(e instanceof Error ? e.message : 'Failed to load projects'));
    }
  }, []);

  useEffect(() => {
    loadLiveBoard();
    loadProjects();
  }, [loadLiveBoard, loadProjects]);

  useWorkEvents((event) => {
    if (event.type === 'selection_updated') {
      setStats(event.data);
      loadLiveBoard();
    }
  });

  const handleStartAgent = useCallback(async () => {
    setBusy(true);
    try {
      await startAgentLoop();
      loadLiveBoard();
    } finally {
      setBusy(false);
    }
  }, [loadLiveBoard]);

  const handleStopAgent = useCallback(async () => {
    setBusy(true);
    try {
      await stopAgentLoop();
      loadLiveBoard();
    } finally {
      setBusy(false);
    }
  }, [loadLiveBoard]);

  const handleEnsureAgent = useCallback(async () => {
    setBusy(true);
    try {
      await ensureAgentLoop();
      loadLiveBoard();
    } finally {
      setBusy(false);
    }
  }, [loadLiveBoard]);

  const handleClearCompleted = useCallback(async () => {
    setBusy(true);
    try {
      await clearCompleted();
      loadLiveBoard();
    } finally {
      setBusy(false);
    }
  }, [loadLiveBoard]);

  const handleDeselect = useCallback(async (taskId: number) => {
    try {
      await deselectTask(taskId);
      loadLiveBoard();
    } catch (e) {
      console.error(e);
    }
  }, [loadLiveBoard]);

  const handleMoveSelection = useCallback(async (taskId: number, position: string) => {
    try {
      await moveSelection(taskId, position);
      loadLiveBoard();
    } catch (e) {
      console.error(e);
    }
  }, [loadLiveBoard]);

  const handleSelectTasks = useCallback(async () => {
    const ids = Array.from(selectedBacklogIds);
    if (ids.length === 0) return;
    setBusy(true);
    try {
      await selectTasks(ids);
      setSelectedBacklogIds(new Set());
      loadLiveBoard();
    } finally {
      setBusy(false);
    }
  }, [selectedBacklogIds, loadLiveBoard]);

  const toggleBacklogSelection = useCallback((taskId: number) => {
    setSelectedBacklogIds((prev) => {
      const next = new Set(prev);
      if (next.has(taskId)) next.delete(taskId);
      else next.add(taskId);
      return next;
    });
  }, []);

  const liveBoardData = liveBoard.tag === 'Success' ? liveBoard.data : null;
  const displayStats = stats ?? liveBoardData?.stats ?? null;
  const agentState = displayStats?.agent_loop_state ?? 'idle';

  return (
    <div>
      <PageHeader
        title="Live Board"
        actions={
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
            <PrimaryButton label="Start Agent" onClick={handleStartAgent} disabled={busy} />
            <Button label="Stop Agent" onClick={handleStopAgent} disabled={busy} />
            <Button label="Ensure Agent" onClick={handleEnsureAgent} disabled={busy} />
            <Button label="Clear Completed" onClick={handleClearCompleted} disabled={busy} />
            <Button label="Refresh" onClick={loadLiveBoard} disabled={busy} />
          </div>
        }
      />

      {displayStats && (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '1rem', marginBottom: '1.5rem' }}>
          <MiniStat label="Backlog" count={displayStats.total_backlog} color={colors.textMuted} />
          <MiniStat label="Selected" count={displayStats.total_selected} color={colors.accent} />
          <MiniStat label="Queued" count={displayStats.queued} color={colors.warning} />
          <MiniStat label="Completed" count={displayStats.completed} color={colors.success} />
          <MiniStat label="Failed" count={displayStats.failed} color={colors.error} />
          <div style={{ padding: '1rem', backgroundColor: colors.bgSurface, borderRadius: '4px', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span
              style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor:
                  agentState === 'running' ? colors.success
                  : agentState === 'paused' ? colors.warning
                  : colors.textMuted,
                animation: agentState === 'running' ? 'statusPulse 2s infinite' : 'none',
              }}
            />
            <span style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textSecondary }}>
              Agent: {agentState}
            </span>
          </div>
        </div>
      )}

      {liveBoard.tag === 'Loading' ? (
        <LoadingSpinner />
      ) : liveBoard.tag === 'Failure' ? (
        <div style={{ color: colors.error }}>{liveBoard.error}</div>
      ) : liveBoardData ? (
        <>
          <div style={{ marginBottom: '2rem' }}>
            <h3 style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', letterSpacing: '0.12em', color: colors.textMuted, marginBottom: '1rem' }}>
              SELECTED TASKS
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
              {[...liveBoardData.selected]
                .sort((a, b) => a.selection.sort_order - b.selection.sort_order)
                .map(({ selection, task, comments }) => (
                  <Card key={task.id} style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                    <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', flexWrap: 'wrap', gap: '0.5rem' }}>
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <span
                          style={{ fontWeight: 500, cursor: 'pointer', color: colors.textPrimary }}
                          onClick={() => navigate(`/tasks/${task.id}`)}
                        >
                          {task.title}
                        </span>
                        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.375rem', marginTop: '0.25rem' }}>
                          <PillBadge
                            bgColor={colors.borderLight}
                            textColor={colors.textSecondary}
                            label={selection.status}
                          />
                          <PillBadge
                            bgColor={colors.borderLight}
                            textColor={priorityBadgeColor(task.priority)}
                            label={taskPriorityLabel(task.priority)}
                          />
                        </div>
                        {comments.length > 0 && (
                          <div style={{ fontSize: '0.8125rem', color: colors.textMuted, marginTop: '0.25rem' }}>
                            {truncateText(80, comments[comments.length - 1]?.content ?? '')}
                          </div>
                        )}
                        <div style={{ fontSize: '0.6875rem', color: colors.textMuted, marginTop: '0.25rem' }}>
                          {selection.started_at && `Started: ${selection.started_at}`}
                          {selection.completed_at && ` | Completed: ${selection.completed_at}`}
                        </div>
                      </div>
                      <div style={{ display: 'flex', gap: '0.25rem' }}>
                        <Button label="↑" onClick={() => handleMoveSelection(task.id, 'top')} />
                        <Button label="↓" onClick={() => handleMoveSelection(task.id, 'bottom')} />
                        <Button label="Deselect" onClick={() => handleDeselect(task.id)} />
                      </div>
                    </div>
                  </Card>
                ))}
            </div>
          </div>

          <SectionHeader title="Backlog" />
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1rem' }}>
            {liveBoardData.backlog.map((t) => (
              <div
                key={t.id}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.75rem',
                  padding: '0.75rem',
                  backgroundColor: colors.bgSurface,
                  border: `1px solid ${colors.border}`,
                  borderRadius: '4px',
                }}
              >
                <input
                  type="checkbox"
                  checked={selectedBacklogIds.has(t.id)}
                  onChange={() => toggleBacklogSelection(t.id)}
                  style={{ accentColor: colors.accent }}
                />
                <span
                  style={{ flex: 1, cursor: 'pointer', fontWeight: 500 }}
                  onClick={() => navigate(`/tasks/${t.id}`)}
                >
                  {t.title}
                </span>
                <PillBadge
                  bgColor={colors.borderLight}
                  textColor={priorityBadgeColor(t.priority)}
                  label={taskPriorityLabel(t.priority)}
                />
                <PillBadge
                  bgColor={colors.borderLight}
                  textColor={taskStatusColor(t.status)}
                  label={taskStatusLabel(t.status)}
                />
                {t.tags.map((tag) => (
                  <TagChip key={tag} tag={tag} />
                ))}
              </div>
            ))}
          </div>
          <PrimaryButton label="Select Tasks" onClick={handleSelectTasks} disabled={selectedBacklogIds.size === 0 || busy} />
        </>
      ) : null}
    </div>
  );
}
