import React, { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  PageHeader,
  BackButton,
  Card,
  SectionHeader,
  PrimaryButton,
  Button,
  FormField,
  TextareaField,
  PillBadge,
  TagChip,
  AccentedItem,
  LoadingSpinner,
  Timestamp,
} from '../components/UI';
import { MarkdownView } from '../components/Markdown';
import {
  getTask,
  listComments,
  updateTask,
  rejectReview,
  moveTaskToTopOrBottom,
  upsertComment,
} from '../api';
import type { WorkTask, WorkComment, RemoteData, EditingField } from '../types';
import { NotAsked, Loading, Success, Failure, isLoading } from '../types';
import { colors, fonts, taskStatusColor, taskStatusLabel, taskPriorityLabel } from '../theme';

const STATUS_OPTIONS = ['todo', 'in_progress', 'ready_for_review', 'under_review', 'done', 'blocked', 'abandoned'];
const PRIORITY_CYCLE: Record<string, string> = { low: 'medium', medium: 'high', high: 'critical', critical: 'low' };

function priorityBadgeColor(priority: string): string {
  switch (priority) {
    case 'critical': return colors.error;
    case 'high': return colors.warning;
    case 'medium': return colors.accent;
    case 'low': return colors.textMuted;
    default: return colors.textMuted;
  }
}

export function TaskDetail(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const taskId = id ? parseInt(id, 10) : NaN;

  const [task, setTask] = useState<RemoteData<WorkTask>>(NotAsked);
  const [comments, setComments] = useState<RemoteData<WorkComment[]>>(NotAsked);
  const [editingField, setEditingField] = useState<EditingField>({ tag: 'none' });
  const [commentForm, setCommentForm] = useState('');
  const [rejectReviewComment, setRejectReviewComment] = useState('');
  const [statusDropdownOpen, setStatusDropdownOpen] = useState(false);

  const loadTask = useCallback(async () => {
    if (isNaN(taskId)) return;
    setTask(Loading);
    try {
      const data = await getTask(taskId);
      setTask(Success(data));
    } catch (e) {
      setTask(Failure(e instanceof Error ? e.message : 'Failed to load task'));
    }
  }, [taskId]);

  const loadComments = useCallback(async () => {
    if (isNaN(taskId)) return;
    setComments(Loading);
    try {
      const data = await listComments(taskId);
      setComments(Success(data));
    } catch (e) {
      setComments(Failure(e instanceof Error ? e.message : 'Failed to load comments'));
    }
  }, [taskId]);

  useEffect(() => {
    loadTask();
    loadComments();
  }, [loadTask, loadComments]);

  const handleUpdateTask = useCallback(async (fields: { title?: string; description?: string; status?: string; priority?: string; tags?: string[] }) => {
    if (isNaN(taskId)) return;
    try {
      const updated = await updateTask(taskId, fields);
      setTask(Success(updated));
      setEditingField({ tag: 'none' });
      setStatusDropdownOpen(false);
    } catch (e) {
      console.error(e);
    }
  }, [taskId]);

  const handleRejectReview = useCallback(async () => {
    if (isNaN(taskId)) return;
    try {
      const updated = await rejectReview(taskId, rejectReviewComment);
      setTask(Success(updated));
      setRejectReviewComment('');
      loadComments();
    } catch (e) {
      console.error(e);
    }
  }, [taskId, rejectReviewComment, loadComments]);

  const handleAddComment = useCallback(async () => {
    if (isNaN(taskId) || !commentForm.trim()) return;
    try {
      await upsertComment({ task_id: taskId, content: commentForm.trim() });
      setCommentForm('');
      loadComments();
    } catch (e) {
      console.error(e);
    }
  }, [taskId, commentForm, loadComments]);

  const handleMove = useCallback(async (position: 'top' | 'bottom') => {
    if (isNaN(taskId)) return;
    try {
      const updated = await moveTaskToTopOrBottom(taskId, position);
      setTask(Success(updated));
    } catch (e) {
      console.error(e);
    }
  }, [taskId]);

  const taskData = task.tag === 'Success' ? task.data : null;
  const commentsData = comments.tag === 'Success' ? comments.data : null;
  const projectId = taskData?.project_id;

  if (isNaN(taskId)) {
    return (
      <div>
        <BackButton onClick={() => navigate(projectId ? `/projects/${projectId}` : '/projects')} />
        <div style={{ color: colors.error, marginTop: '1rem' }}>Invalid task ID</div>
      </div>
    );
  }

  if (task.tag === 'Failure') {
    return (
      <div>
        <BackButton onClick={() => navigate(projectId ? `/projects/${projectId}` : '/projects')} />
        <div style={{ color: colors.error, marginTop: '1rem' }}>{task.error}</div>
      </div>
    );
  }

  if (!taskData) {
    return (
      <div>
        <BackButton onClick={() => navigate('/projects')} />
        <LoadingSpinner />
      </div>
    );
  }

  type CommentNode = { comment: WorkComment; children: CommentNode[] };
  const buildCommentTree = (items: WorkComment[]): CommentNode[] => {
    const byParent = new Map<number | null, WorkComment[]>();
    for (const c of items) {
      const pid = c.parent_comment_id;
      if (!byParent.has(pid)) byParent.set(pid, []);
      byParent.get(pid)!.push(c);
    }
    const build = (pid: number | null): CommentNode[] => {
      const list = byParent.get(pid) ?? [];
      return list.map((c) => ({ comment: c, children: build(c.id) }));
    };
    return build(null);
  };

  const commentTree = commentsData ? buildCommentTree(commentsData) : [];

  const renderComment = (c: WorkComment, depth: number) => (
    <AccentedItem key={c.id} borderColor={colors.accent} style={{ marginLeft: depth > 0 ? '1.5rem' : 0, marginBottom: '0.5rem' }}>
      <MarkdownView content={c.content} />
      <div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.5rem', fontSize: '0.6875rem', color: colors.textMuted }}>
        <Timestamp ts={c.created_at} />
      </div>
    </AccentedItem>
  );

  const renderTree = (nodes: CommentNode[], depth: number): React.ReactNode[] =>
    nodes.flatMap((n) => [
      renderComment(n.comment, depth),
      ...renderTree(n.children, depth + 1),
    ]);

  return (
    <div>
      <BackButton onClick={() => navigate(projectId ? `/projects/${projectId}` : '/projects')} />
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          flexWrap: 'wrap',
          gap: '1rem',
          marginBottom: 'clamp(1.25rem, 4vw, 2rem)',
          paddingTop: 'clamp(1.25rem, 4vw, 2rem)',
          paddingBottom: 'clamp(1rem, 3vw, 1.5rem)',
          borderBottom: `1px solid ${colors.border}`,
        }}
      >
        <div style={{ flex: 1, minWidth: 0 }}>
          {editingField.tag === 'title' ? (
            <input
              value={editingField.value}
              onChange={(e) => setEditingField((f) => f.tag === 'title' ? { ...f, value: e.target.value } : f)}
              onBlur={() => editingField.tag === 'title' && handleUpdateTask({ title: editingField.value })}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && editingField.tag === 'title') {
                  handleUpdateTask({ title: editingField.value });
                }
              }}
              autoFocus
              style={{
                backgroundColor: colors.bgPrimary,
                color: colors.textPrimary,
                border: `1px solid ${colors.border}`,
                padding: '0.25rem 0.5rem',
                fontFamily: fonts.display,
                fontSize: '1.25rem',
                width: '100%',
              }}
            />
          ) : (
            <h2
              style={{
                fontFamily: fonts.display,
                fontSize: 'clamp(1.25rem, 5vw, 1.75rem)',
                fontWeight: 600,
                letterSpacing: '0.02em',
                textTransform: 'uppercase',
                margin: 0,
                color: colors.textPrimary,
                cursor: 'pointer',
              }}
              onClick={() => setEditingField({ tag: 'title', value: taskData.title })}
            >
              {taskData.title}
            </h2>
          )}
        </div>
      </div>

      <Card>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexWrap: 'wrap' }}>
            <span style={{ fontFamily: fonts.mono, fontSize: '0.625rem', color: colors.textMuted }}>STATUS</span>
            <div style={{ position: 'relative' }}>
              <button
                type="button"
                onClick={() => setStatusDropdownOpen((o) => !o)}
                style={{
                  padding: '0.25rem 0.5rem',
                  fontFamily: fonts.mono,
                  fontSize: '0.6875rem',
                  backgroundColor: colors.borderLight,
                  color: taskStatusColor(taskData.status),
                  border: `1px solid ${colors.border}`,
                  borderRadius: '2px',
                  cursor: 'pointer',
                }}
              >
                {taskStatusLabel(taskData.status)}
              </button>
              {statusDropdownOpen && (
                <div
                  style={{
                    position: 'absolute',
                    top: '100%',
                    left: 0,
                    marginTop: '0.25rem',
                    backgroundColor: colors.bgTertiary,
                    border: `1px solid ${colors.border}`,
                    borderRadius: '2px',
                    zIndex: 10,
                    minWidth: '140px',
                  }}
                >
                  {STATUS_OPTIONS.map((s) => (
                    <button
                      key={s}
                      type="button"
                      onClick={() => handleUpdateTask({ status: s })}
                      style={{
                        display: 'block',
                        width: '100%',
                        padding: '0.5rem 0.75rem',
                        textAlign: 'left',
                        backgroundColor: 'transparent',
                        color: colors.textPrimary,
                        border: 'none',
                        fontFamily: fonts.mono,
                        fontSize: '0.75rem',
                        cursor: 'pointer',
                      }}
                    >
                      {taskStatusLabel(s)}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span style={{ fontFamily: fonts.mono, fontSize: '0.625rem', color: colors.textMuted }}>PRIORITY</span>
            <button
              type="button"
              onClick={() => handleUpdateTask({ priority: PRIORITY_CYCLE[taskData.priority] ?? 'medium' })}
            >
              <PillBadge
                bgColor={colors.borderLight}
                textColor={priorityBadgeColor(taskData.priority)}
                label={taskPriorityLabel(taskData.priority)}
              />
            </button>
          </div>

          <div>
            <span style={{ fontFamily: fonts.mono, fontSize: '0.625rem', color: colors.textMuted }}>DESCRIPTION</span>
            {editingField.tag === 'description' ? (
              <div style={{ marginTop: '0.5rem' }}>
                <TextareaField
                  value={editingField.value}
                  onChange={(v) => setEditingField((f) => f.tag === 'description' ? { ...f, value: v } : f)}
                  placeholder="Description"
                />
                <div style={{ marginTop: '0.5rem' }}>
                  <Button label="Save" onClick={() => editingField.tag === 'description' && handleUpdateTask({ description: editingField.value })} />
                  <Button label="Cancel" onClick={() => setEditingField({ tag: 'none' })} />
                </div>
              </div>
            ) : (
              <div
                style={{ marginTop: '0.5rem', cursor: 'pointer' }}
                onClick={() => setEditingField({ tag: 'description', value: taskData.description })}
              >
                <MarkdownView content={taskData.description || '_No description_'} />
              </div>
            )}
          </div>

          <div>
            <span style={{ fontFamily: fonts.mono, fontSize: '0.625rem', color: colors.textMuted }}>TAGS</span>
            {editingField.tag === 'tags' ? (
              <div style={{ marginTop: '0.5rem' }}>
                <input
                  value={editingField.value}
                  onChange={(e) => setEditingField((f) => f.tag === 'tags' ? { ...f, value: e.target.value } : f)}
                  onBlur={() => {
                    if (editingField.tag === 'tags') {
                      const tags = editingField.value.split(',').map((t) => t.trim()).filter(Boolean);
                      handleUpdateTask({ tags });
                    }
                  }}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && editingField.tag === 'tags') {
                      const tags = editingField.value.split(',').map((t) => t.trim()).filter(Boolean);
                      handleUpdateTask({ tags });
                    }
                  }}
                  placeholder="tag1, tag2"
                  style={{
                    width: '100%',
                    backgroundColor: colors.bgPrimary,
                    color: colors.textPrimary,
                    border: `1px solid ${colors.border}`,
                    padding: '0.5rem',
                    fontFamily: fonts.body,
                  }}
                />
              </div>
            ) : (
              <div
                style={{ marginTop: '0.5rem', display: 'flex', flexWrap: 'wrap', gap: '0.375rem', cursor: 'pointer' }}
                onClick={() => setEditingField({ tag: 'tags', value: taskData.tags.join(', ') })}
              >
                {taskData.tags.length > 0 ? taskData.tags.map((t) => <TagChip key={t} tag={t} />) : <span style={{ color: colors.textMuted }}>Click to add tags</span>}
              </div>
            )}
          </div>

          <div style={{ display: 'flex', gap: '1rem', fontSize: '0.6875rem', color: colors.textMuted }}>
            <span>Created: <Timestamp ts={taskData.created_at} /></span>
            <span>Updated: <Timestamp ts={taskData.updated_at} /></span>
          </div>

          {(taskData.blocked_by.length > 0 || taskData.blocks.length > 0) && (
            <div style={{ fontSize: '0.875rem' }}>
              {taskData.blocked_by.length > 0 && (
                <span style={{ marginRight: '1rem' }}>Blocked by: {taskData.blocked_by.join(', ')}</span>
              )}
              {taskData.blocks.length > 0 && (
                <span>Blocks: {taskData.blocks.join(', ')}</span>
              )}
            </div>
          )}
        </div>
      </Card>

      <div style={{ marginTop: '1rem', display: 'flex', gap: '0.5rem' }}>
        <Button label="Move to Top" onClick={() => handleMove('top')} />
        <Button label="Move to Bottom" onClick={() => handleMove('bottom')} />
      </div>

      {taskData.status === 'under_review' && (
        <Card style={{ marginTop: '1rem' }}>
          <SectionHeader title="Reject Review" />
          <TextareaField
            value={rejectReviewComment}
            onChange={setRejectReviewComment}
            placeholder="Reviewer comment..."
          />
          <PrimaryButton label="Reject Review" onClick={handleRejectReview} style={{ marginTop: '0.5rem' }} />
        </Card>
      )}

      <SectionHeader title="Comments" />
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1rem' }}>
        {renderTree(commentTree, 0)}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
        <TextareaField value={commentForm} onChange={setCommentForm} placeholder="Add a comment..." />
        <PrimaryButton label="Add Comment" onClick={handleAddComment} disabled={!commentForm.trim()} />
      </div>
    </div>
  );
}
