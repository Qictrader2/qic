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
  InputField,
  TextareaField,
  SelectField,
  TagChip,
  PillBadge,
  DocTypeBadge,
  StatusDot,
  MiniStat,
  LoadingSpinner,
  Timestamp,
} from '../components/UI';
import { MarkdownView } from '../components/Markdown';
import {
  getProject,
  listTasks,
  searchDocuments,
  getRecentActivity,
  getTaskAnalytics,
  createTask,
  createDocument,
  moveTaskToTopOrBottom,
} from '../api';
import type {
  WorkProject,
  WorkTask,
  WorkDocument,
  ActivityLog,
  TaskAnalytics,
  RemoteData,
  ProjectTab,
  TaskViewMode,
  TaskFilters,
  TaskForm,
  DocumentForm,
} from '../types';
import { NotAsked, Loading, Success, Failure, isLoading, emptyTaskForm, emptyDocumentForm, emptyTaskFilters } from '../types';
import { colors, fonts, formatDateTime, taskStatusColor, taskStatusLabel, taskPriorityLabel, truncateText } from '../theme';

const STATUS_OPTIONS: [string, string][] = [
  ['todo', 'Todo'],
  ['in_progress', 'In Progress'],
  ['ready_for_review', 'Ready for Review'],
  ['under_review', 'Under Review'],
  ['done', 'Done'],
  ['blocked', 'Blocked'],
  ['abandoned', 'Abandoned'],
];

const PRIORITY_OPTIONS: [string, string][] = [
  ['low', 'Low'],
  ['medium', 'Medium'],
  ['high', 'High'],
  ['critical', 'Critical'],
];

const DOC_TYPE_OPTIONS: [string, string][] = [
  ['plan', 'Plan'],
  ['specification', 'Specification'],
  ['notes', 'Notes'],
  ['code', 'Code'],
  ['other', 'Other'],
];

const BOARD_COLUMNS = ['todo', 'in_progress', 'ready_for_review', 'under_review', 'done'];

function priorityBadgeColor(priority: string): string {
  switch (priority) {
    case 'critical': return colors.error;
    case 'high': return colors.warning;
    case 'medium': return colors.accent;
    case 'low': return colors.textMuted;
    default: return colors.textMuted;
  }
}

function TabButton({ label, active, onClick }: { label: string; active: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      style={{
        backgroundColor: active ? colors.accentDim : 'transparent',
        color: active ? colors.accent : colors.textSecondary,
        border: `1px solid ${active ? 'rgba(0, 212, 170, 0.45)' : colors.border}`,
        borderRadius: '2px',
        padding: '0.5rem 1rem',
        fontFamily: fonts.mono,
        fontSize: '0.6875rem',
        fontWeight: 600,
        letterSpacing: '0.08em',
        textTransform: 'uppercase',
        cursor: 'pointer',
      }}
    >
      {label}
    </button>
  );
}

export function ProjectDetail(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const projectId = id ? parseInt(id, 10) : NaN;

  const [project, setProject] = useState<RemoteData<WorkProject>>(NotAsked);
  const [tasks, setTasks] = useState<RemoteData<WorkTask[]>>(NotAsked);
  const [documents, setDocuments] = useState<RemoteData<WorkDocument[]>>(NotAsked);
  const [activity, setActivity] = useState<RemoteData<ActivityLog[]>>(NotAsked);
  const [analytics, setAnalytics] = useState<RemoteData<TaskAnalytics>>(NotAsked);

  const [activeTab, setActiveTab] = useState<ProjectTab>('tasks');
  const [taskViewMode, setTaskViewMode] = useState<TaskViewMode>('list');
  const [taskFilters, setTaskFilters] = useState<TaskFilters>(emptyTaskFilters);
  const [showTaskForm, setShowTaskForm] = useState(false);
  const [taskForm, setTaskForm] = useState<TaskForm>(emptyTaskForm);
  const [showDocumentForm, setShowDocumentForm] = useState(false);
  const [documentForm, setDocumentForm] = useState<DocumentForm>(emptyDocumentForm);

  const loadAll = useCallback(async () => {
    if (isNaN(projectId)) return;
    setProject(Loading);
    setTasks(Loading);
    setDocuments(Loading);
    setActivity(Loading);
    setAnalytics(Loading);

    try {
      const [proj, taskList, docList, actList, anal] = await Promise.all([
        getProject(projectId),
        listTasks(projectId, taskFilters.statusFilter.length > 0 ? taskFilters.statusFilter : undefined),
        searchDocuments('', projectId),
        getRecentActivity(20, projectId),
        getTaskAnalytics(projectId),
      ]);
      setProject(Success(proj));
      setTasks(Success(taskList));
      setDocuments(Success(docList));
      setActivity(Success(actList));
      setAnalytics(Success(anal));
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Failed to load';
      setProject(Failure(msg));
      setTasks(Failure(msg));
      setDocuments(Failure(msg));
      setActivity(Failure(msg));
      setAnalytics(Failure(msg));
    }
  }, [projectId, taskFilters.statusFilter]);

  useEffect(() => {
    loadAll();
  }, [loadAll]);

  const toggleStatusFilter = useCallback((status: string) => {
    setTaskFilters((f) => ({
      ...f,
      statusFilter: f.statusFilter.includes(status)
        ? f.statusFilter.filter((s) => s !== status)
        : [...f.statusFilter, status],
    }));
  }, []);

  const handleCreateTask = useCallback(async () => {
    if (isNaN(projectId)) return;
    const tags: string[] = [];
    try {
      await createTask(projectId, taskForm.title, taskForm.description, taskForm.priority, tags);
      setShowTaskForm(false);
      setTaskForm(emptyTaskForm);
      loadAll();
    } catch (e) {
      console.error(e);
    }
  }, [projectId, taskForm, loadAll]);

  const handleCreateDocument = useCallback(async () => {
    if (isNaN(projectId)) return;
    try {
      await createDocument(projectId, documentForm.title, documentForm.content, documentForm.documentType);
      setShowDocumentForm(false);
      setDocumentForm(emptyDocumentForm);
      loadAll();
    } catch (e) {
      console.error(e);
    }
  }, [projectId, documentForm, loadAll]);

  const handleMoveTask = useCallback(async (taskId: number, position: 'top' | 'bottom') => {
    try {
      await moveTaskToTopOrBottom(taskId, position);
      loadAll();
    } catch (e) {
      console.error(e);
    }
  }, [loadAll]);

  const projectData = project.tag === 'Success' ? project.data : null;
  const tasksData = tasks.tag === 'Success' ? tasks.data : null;
  const documentsData = documents.tag === 'Success' ? documents.data : null;
  const activityData = activity.tag === 'Success' ? activity.data : null;
  const analyticsData = analytics.tag === 'Success' ? analytics.data : null;

  if (isNaN(projectId)) {
    return (
      <div>
        <BackButton onClick={() => navigate('/projects')} />
        <div style={{ color: colors.error, marginTop: '1rem' }}>Invalid project ID</div>
      </div>
    );
  }

  if (project.tag === 'Failure') {
    return (
      <div>
        <BackButton onClick={() => navigate('/projects')} />
        <div style={{ color: colors.error, marginTop: '1rem' }}>{project.error}</div>
      </div>
    );
  }

  if (!projectData) {
    return (
      <div>
        <BackButton onClick={() => navigate('/projects')} />
        <LoadingSpinner />
      </div>
    );
  }

  return (
    <div>
      <BackButton onClick={() => navigate('/projects')} />
      <PageHeader
        title={projectData.name}
        actions={null}
      />
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem', marginBottom: '1rem' }}>
        {projectData.tags.map((t) => (
          <TagChip key={t} tag={t} />
        ))}
      </div>

      <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1.5rem' }}>
        <TabButton label="Tasks" active={activeTab === 'tasks'} onClick={() => setActiveTab('tasks')} />
        <TabButton label="Documents" active={activeTab === 'documents'} onClick={() => setActiveTab('documents')} />
        <TabButton label="Activity" active={activeTab === 'activity'} onClick={() => setActiveTab('activity')} />
      </div>

      {activeTab === 'tasks' && (
        <>
          {analyticsData && (
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '1rem', marginBottom: '1rem' }}>
              {analyticsData.status_counts.map((sc) => (
                <MiniStat key={sc.status} label={taskStatusLabel(sc.status)} count={sc.count} color={taskStatusColor(sc.status)} />
              ))}
              {analyticsData.avg_completion_hours != null && (
                <div style={{ padding: '1rem', backgroundColor: colors.bgSurface, borderRadius: '4px' }}>
                  <span style={{ fontFamily: fonts.mono, fontSize: '0.5625rem', color: colors.textMuted }}>AVG COMPLETION</span>
                  <div style={{ fontFamily: fonts.display, fontSize: '1.25rem', color: colors.accent }}>{analyticsData.avg_completion_hours.toFixed(1)}h</div>
                </div>
              )}
            </div>
          )}

          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem', marginBottom: '1rem', alignItems: 'center' }}>
            <PrimaryButton label="New Task" onClick={() => setShowTaskForm((s) => !s)} />
            <Button
              label={taskViewMode === 'list' ? 'Board' : 'List'}
              onClick={() => setTaskViewMode((m) => (m === 'list' ? 'board' : 'list'))}
            />
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.375rem' }}>
              {STATUS_OPTIONS.map(([v]) => (
                <button
                  key={v}
                  type="button"
                  onClick={() => toggleStatusFilter(v)}
                  style={{
                    padding: '0.25rem 0.5rem',
                    fontSize: '0.625rem',
                    fontFamily: fonts.mono,
                    backgroundColor: taskFilters.statusFilter.includes(v) ? taskStatusColor(v) : colors.bgSurface,
                    color: taskFilters.statusFilter.includes(v) ? colors.bgPrimary : colors.textSecondary,
                    border: `1px solid ${colors.border}`,
                    borderRadius: '2px',
                    cursor: 'pointer',
                  }}
                >
                  {taskStatusLabel(v)}
                </button>
              ))}
            </div>
          </div>

          {showTaskForm && (
            <Card style={{ marginBottom: '1rem' }}>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                <FormField label="Title">
                  <InputField value={taskForm.title} onChange={(v) => setTaskForm((f) => ({ ...f, title: v }))} placeholder="Task title" />
                </FormField>
                <FormField label="Description">
                  <TextareaField value={taskForm.description} onChange={(v) => setTaskForm((f) => ({ ...f, description: v }))} placeholder="Description" />
                </FormField>
                <FormField label="Priority">
                  <SelectField value={taskForm.priority} onChange={(v) => setTaskForm((f) => ({ ...f, priority: v }))} options={PRIORITY_OPTIONS} />
                </FormField>
                <div style={{ display: 'flex', gap: '0.75rem' }}>
                  <PrimaryButton label="Submit" onClick={handleCreateTask} />
                  <Button label="Cancel" onClick={() => { setShowTaskForm(false); setTaskForm(emptyTaskForm); }} />
                </div>
              </div>
            </Card>
          )}

          {tasks.tag === 'Loading' ? (
            <LoadingSpinner />
          ) : tasks.tag === 'Failure' ? (
            <div style={{ color: colors.error }}>{tasks.error}</div>
          ) : tasksData ? (
            taskViewMode === 'list' ? (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                {tasksData.map((t) => (
                  <Card key={t.id} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '0.5rem' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', flex: 1, minWidth: 0 }}>
                      <StatusDot color={taskStatusColor(t.status)} />
                      <span
                        style={{ cursor: 'pointer', color: colors.textPrimary, fontWeight: 500 }}
                        onClick={() => navigate(`/tasks/${t.id}`)}
                      >
                        {t.title}
                      </span>
                      <PillBadge
                        bgColor={priorityBadgeColor(t.priority) === colors.error ? colors.errorDim : colors.borderLight}
                        textColor={priorityBadgeColor(t.priority)}
                        label={taskPriorityLabel(t.priority)}
                      />
                      {t.tags.map((tag) => (
                        <TagChip key={tag} tag={tag} />
                      ))}
                    </div>
                    <div style={{ display: 'flex', gap: '0.25rem' }}>
                      <Button label="↑" onClick={() => handleMoveTask(t.id, 'top')} />
                      <Button label="↓" onClick={() => handleMoveTask(t.id, 'bottom')} />
                    </div>
                  </Card>
                ))}
              </div>
            ) : (
              <div style={{ display: 'flex', gap: '1rem', overflowX: 'auto', paddingBottom: '0.5rem' }}>
                {BOARD_COLUMNS.map((status) => {
                  const colTasks = tasksData.filter((t) => t.status === status);
                  return (
                    <div
                      key={status}
                      style={{
                        minWidth: '280px',
                        width: '280px',
                        backgroundColor: colors.bgSurface,
                        borderRadius: '4px',
                        border: `1px solid ${colors.border}`,
                        padding: '0.75rem',
                      }}
                    >
                      <div style={{ marginBottom: '0.75rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                        <span style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', fontWeight: 600, color: colors.textSecondary }}>
                          {taskStatusLabel(status)}
                        </span>
                        <span style={{ fontFamily: fonts.mono, fontSize: '0.625rem', color: colors.textMuted }}>({colTasks.length})</span>
                      </div>
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        {colTasks.map((t) => (
                          <Card
                            key={t.id}
                            style={{ padding: '0.75rem', cursor: 'pointer' }}
                            onClick={() => navigate(`/tasks/${t.id}`)}
                          >
                            <div style={{ fontSize: '0.875rem', fontWeight: 500, marginBottom: '0.25rem' }}>{t.title}</div>
                            <PillBadge
                              bgColor={colors.borderLight}
                              textColor={priorityBadgeColor(t.priority)}
                              label={taskPriorityLabel(t.priority)}
                            />
                            {t.tags.length > 0 && (
                              <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.25rem', marginTop: '0.25rem' }}>
                                {t.tags.map((tag) => (
                                  <TagChip key={tag} tag={tag} />
                                ))}
                              </div>
                            )}
                          </Card>
                        ))}
                      </div>
                    </div>
                  );
                })}
              </div>
            )
          ) : null}
        </>
      )}

      {activeTab === 'documents' && (
        <>
          <div style={{ marginBottom: '1rem' }}>
            <PrimaryButton label="New Document" onClick={() => setShowDocumentForm((s) => !s)} />
          </div>
          {showDocumentForm && (
            <Card style={{ marginBottom: '1rem' }}>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                <FormField label="Title">
                  <InputField value={documentForm.title} onChange={(v) => setDocumentForm((f) => ({ ...f, title: v }))} placeholder="Document title" />
                </FormField>
                <FormField label="Content">
                  <TextareaField value={documentForm.content} onChange={(v) => setDocumentForm((f) => ({ ...f, content: v }))} placeholder="Content" />
                </FormField>
                <FormField label="Type">
                  <SelectField value={documentForm.documentType} onChange={(v) => setDocumentForm((f) => ({ ...f, documentType: v }))} options={DOC_TYPE_OPTIONS} />
                </FormField>
                <div style={{ display: 'flex', gap: '0.75rem' }}>
                  <PrimaryButton label="Submit" onClick={handleCreateDocument} />
                  <Button label="Cancel" onClick={() => { setShowDocumentForm(false); setDocumentForm(emptyDocumentForm); }} />
                </div>
              </div>
            </Card>
          )}
          {documents.tag === 'Loading' ? (
            <LoadingSpinner />
          ) : documents.tag === 'Failure' ? (
            <div style={{ color: colors.error }}>{documents.error}</div>
          ) : documentsData ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
              {documentsData.map((d) => (
                <Card key={d.id} style={{ cursor: 'pointer' }} onClick={() => navigate(`/documents/${d.id}`)}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.25rem' }}>
                    <DocTypeBadge docType={d.document_type} />
                    <span style={{ fontWeight: 500 }}>{d.title}</span>
                  </div>
                  <div style={{ fontSize: '0.8125rem', color: colors.textSecondary }}>{truncateText(100, d.content)}</div>
                  <Timestamp ts={d.updated_at} />
                </Card>
              ))}
            </div>
          ) : null}
        </>
      )}

      {activeTab === 'activity' && (
        <>
          {activity.tag === 'Loading' ? (
            <LoadingSpinner />
          ) : activity.tag === 'Failure' ? (
            <div style={{ color: colors.error }}>{activity.error}</div>
          ) : activityData ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
              {activityData.map((a) => (
                <Card key={a.id}>
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem', alignItems: 'center' }}>
                    <PillBadge bgColor={colors.borderLight} textColor={colors.textSecondary} label={a.action} />
                    <span style={{ fontFamily: fonts.mono, fontSize: '0.6875rem', color: colors.textMuted }}>{a.actor}</span>
                    <span style={{ fontSize: '0.875rem', color: colors.textPrimary }}>{a.details}</span>
                    <Timestamp ts={a.created_at} />
                  </div>
                </Card>
              ))}
            </div>
          ) : null}
        </>
      )}
    </div>
  );
}
