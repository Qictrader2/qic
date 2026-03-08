import type {
  FeedData, ResponseFeedData, ChatSummary, MessagesPage, LogsPage,
  SemanticStatus, TunnelStatus, SetupStatusResponse, TelegramSetupResponse,
  GeminiSetupResponse, ClaudeInstallResponse, ClaudeAuthCheckResponse,
  ClaudeTestResponse, ThreadingCheckResponse, Settings, ApiKeysData,
  UpdateApiKeysResponse, CronJob, CronStatus, Conversation, ChatMessage,
  CreateConversationResponse, SendMessageResponse, UploadMediaResponse,
  WorkProject, WorkTask, WorkDocument, WorkComment, ActivityLog,
  LiveBoard, LiveBoardSelection, TaskAnalytics, TranscribeResult, FormatResult,
} from './types';
import { mediaAttachmentFromServerFields } from './types';

// ═══════════════════════════════════════════════════════════════════════════
// HTTP HELPERS
// ═══════════════════════════════════════════════════════════════════════════

class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ApiError';
  }
}

async function get<T>(url: string): Promise<T> {
  const res = await fetch(url, { credentials: 'include' });
  if (!res.ok) throw new ApiError(res.status, `GET ${url}: ${res.statusText}`);
  return res.json() as Promise<T>;
}

async function post<T>(url: string, body?: unknown): Promise<T> {
  const res = await fetch(url, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: body !== undefined ? JSON.stringify(body) : '{}',
  });
  if (!res.ok) throw new ApiError(res.status, `POST ${url}: ${res.statusText}`);
  return res.json() as Promise<T>;
}

async function put<T>(url: string, body: unknown): Promise<T> {
  const res = await fetch(url, {
    method: 'PUT',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new ApiError(res.status, `PUT ${url}: ${res.statusText}`);
  return res.json() as Promise<T>;
}

async function del<T>(url: string): Promise<T> {
  const res = await fetch(url, {
    method: 'DELETE',
    credentials: 'include',
  });
  if (!res.ok) throw new ApiError(res.status, `DELETE ${url}: ${res.statusText}`);
  return res.json() as Promise<T>;
}

async function postExpectVoid(url: string, body?: unknown): Promise<void> {
  const res = await fetch(url, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: body !== undefined ? JSON.stringify(body) : '{}',
  });
  if (!res.ok) throw new ApiError(res.status, `POST ${url}: ${res.statusText}`);
}

async function putExpectVoid(url: string, body: unknown): Promise<void> {
  const res = await fetch(url, {
    method: 'PUT',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new ApiError(res.status, `PUT ${url}: ${res.statusText}`);
}

async function delExpectVoid(url: string): Promise<void> {
  const res = await fetch(url, {
    method: 'DELETE',
    credentials: 'include',
  });
  if (!res.ok) throw new ApiError(res.status, `DELETE ${url}: ${res.statusText}`);
}

function qs(params: Record<string, string | number | undefined | null>): string {
  const entries = Object.entries(params).filter(
    (e): e is [string, string | number] => e[1] != null && e[1] !== '',
  );
  if (entries.length === 0) return '';
  return '?' + entries.map(([k, v]) => `${k}=${encodeURIComponent(v)}`).join('&');
}

// Work API helper: POST to /api/work/... and unwrap { data: T }
async function workPost<T>(path: string, body: Record<string, unknown>): Promise<T> {
  const res = await post<{ data: T }>(`/api/work${path}`, body);
  return res.data;
}

// ═══════════════════════════════════════════════════════════════════════════
// STATUS
// ═══════════════════════════════════════════════════════════════════════════

export async function getStatus(): Promise<void> {
  await get<{ status: string }>('/api/status');
}

// ═══════════════════════════════════════════════════════════════════════════
// FEED
// ═══════════════════════════════════════════════════════════════════════════

export async function getFeed(): Promise<FeedData> {
  return get<FeedData>('/api/feed');
}

// ═══════════════════════════════════════════════════════════════════════════
// RESPONSES
// ═══════════════════════════════════════════════════════════════════════════

export async function getResponses(): Promise<ResponseFeedData> {
  return get<ResponseFeedData>('/api/responses');
}

// ═══════════════════════════════════════════════════════════════════════════
// CHATS (Telegram)
// ═══════════════════════════════════════════════════════════════════════════

export async function getChats(): Promise<ChatSummary[]> {
  const res = await get<{ chats: ChatSummary[] }>('/api/chats');
  return res.chats;
}

// ═══════════════════════════════════════════════════════════════════════════
// MESSAGES
// ═══════════════════════════════════════════════════════════════════════════

export async function getMessages(
  chatId: string,
  page: number,
  pageSize: number,
  search?: string,
  topicId?: string,
): Promise<MessagesPage> {
  const q = qs({ page, page_size: pageSize, search, topic_id: topicId });
  return get<MessagesPage>(`/api/messages/${chatId}${q}`);
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGS
// ═══════════════════════════════════════════════════════════════════════════

export async function getLogs(
  page: number,
  pageSize: number,
  search?: string,
): Promise<LogsPage> {
  const q = qs({ page, page_size: pageSize, search });
  return get<LogsPage>(`/api/logs${q}`);
}

// ═══════════════════════════════════════════════════════════════════════════
// SEMANTIC INDEXER
// ═══════════════════════════════════════════════════════════════════════════

export async function getSemanticStatus(): Promise<SemanticStatus> {
  return get<SemanticStatus>('/api/semantic/status');
}

export async function toggleSemantic(enabled: boolean): Promise<SemanticStatus> {
  return post<SemanticStatus>('/api/semantic/toggle', { enabled });
}

export async function triggerSemanticReindex(): Promise<void> {
  await postExpectVoid('/api/semantic/reindex');
}

// ═══════════════════════════════════════════════════════════════════════════
// TUNNEL
// ═══════════════════════════════════════════════════════════════════════════

export async function getTunnelStatus(): Promise<TunnelStatus> {
  return get<TunnelStatus>('/api/tunnel/status');
}

// ═══════════════════════════════════════════════════════════════════════════
// SETUP
// ═══════════════════════════════════════════════════════════════════════════

export async function getSetupStatus(): Promise<SetupStatusResponse> {
  return get<SetupStatusResponse>('/api/setup/status');
}

export async function postTelegramToken(token: string): Promise<TelegramSetupResponse> {
  return post<TelegramSetupResponse>('/api/setup/telegram', { token });
}

export async function postGeminiKey(key: string): Promise<GeminiSetupResponse> {
  return post<GeminiSetupResponse>('/api/setup/gemini', { key });
}

export async function postInstallClaude(): Promise<ClaudeInstallResponse> {
  return post<ClaudeInstallResponse>('/api/setup/install-claude');
}

export async function getClaudeAuth(): Promise<ClaudeAuthCheckResponse> {
  return get<ClaudeAuthCheckResponse>('/api/setup/claude-auth');
}

export async function postUpdateClaude(): Promise<ClaudeInstallResponse> {
  return post<ClaudeInstallResponse>('/api/setup/update-claude');
}

export async function postTestClaude(): Promise<ClaudeTestResponse> {
  return post<ClaudeTestResponse>('/api/setup/test-claude');
}

export async function checkThreading(): Promise<ThreadingCheckResponse> {
  return post<ThreadingCheckResponse>('/api/setup/check-threading');
}

export async function getApiKeys(): Promise<ApiKeysData> {
  return get<ApiKeysData>('/api/setup/api-keys');
}

export async function putApiKeys(
  telegramToken?: string,
  geminiKey?: string,
): Promise<UpdateApiKeysResponse> {
  const body: Record<string, string> = {};
  if (telegramToken !== undefined) body.telegram_token = telegramToken;
  if (geminiKey !== undefined) body.gemini_key = geminiKey;
  return put<UpdateApiKeysResponse>('/api/setup/api-keys', body);
}

// ═══════════════════════════════════════════════════════════════════════════
// SETTINGS
// ═══════════════════════════════════════════════════════════════════════════

export async function getSettings(): Promise<Settings> {
  return get<Settings>('/api/settings');
}

export async function putSettings(settings: Settings): Promise<Settings> {
  return put<Settings>('/api/settings', {
    show_tool_messages: settings.show_tool_messages,
    show_thinking_messages: settings.show_thinking_messages,
    show_tool_results: settings.show_tool_results,
    omp_num_threads: settings.omp_num_threads,
    allowed_username: settings.allowed_username,
    chat_harness: settings.chat_harness,
    claude_model: settings.claude_model,
    dev_role_prompt: settings.dev_role_prompt,
    harden_role_prompt: settings.harden_role_prompt,
    pm_role_prompt: settings.pm_role_prompt,
  });
}

// ═══════════════════════════════════════════════════════════════════════════
// CRON JOBS
// ═══════════════════════════════════════════════════════════════════════════

export async function getCronJobs(): Promise<CronJob[]> {
  const res = await get<{ jobs: CronJob[] }>('/api/cron/jobs');
  return res.jobs;
}

export async function getCronStatus(): Promise<CronStatus> {
  return get<CronStatus>('/api/cron/status');
}

export async function pauseCronJob(jobId: string): Promise<CronJob> {
  return post<CronJob>(`/api/cron/jobs/${jobId}/pause`);
}

export async function resumeCronJob(jobId: string): Promise<CronJob> {
  return post<CronJob>(`/api/cron/jobs/${jobId}/resume`);
}

export async function cancelCronJob(jobId: string): Promise<CronJob> {
  return del<CronJob>(`/api/cron/jobs/${jobId}`);
}

// ═══════════════════════════════════════════════════════════════════════════
// CHAT API
// ═══════════════════════════════════════════════════════════════════════════

export async function getConversations(): Promise<Conversation[]> {
  const res = await get<{ conversations: Conversation[] }>('/api/chat/conversations');
  return res.conversations;
}

interface RawChatMessage {
  id: string;
  direction: string;
  content: string;
  timestamp: string;
  media_type?: string | null;
  media_path?: string | null;
}

export async function getChatMessages(conversationId: string): Promise<ChatMessage[]> {
  const res = await get<{ messages: RawChatMessage[] }>(`/api/chat/messages/${conversationId}`);
  return res.messages.map(m => ({
    id: m.id,
    direction: m.direction === 'outbound' ? 'outbound' as const : 'inbound' as const,
    content: m.content,
    timestamp: m.timestamp,
    attachments: mediaAttachmentFromServerFields(m.media_type, m.media_path),
  }));
}

export async function createConversation(): Promise<CreateConversationResponse> {
  return post<CreateConversationResponse>('/api/chat/conversations');
}

export async function sendChatMessage(
  conversationId: string,
  content: string,
): Promise<SendMessageResponse> {
  return post<SendMessageResponse>('/api/chat/send', { conversation_id: conversationId, content });
}

export async function renameConversation(conversationId: string, name: string): Promise<void> {
  await putExpectVoid(`/api/chat/conversations/${conversationId}/name`, { name });
}

export async function deleteConversation(conversationId: string): Promise<void> {
  await delExpectVoid(`/api/chat/conversations/${conversationId}`);
}

export async function uploadChatMedia(
  conversationId: string,
  base64Data: string,
  filename: string,
  mimeType: string,
): Promise<UploadMediaResponse> {
  return post<UploadMediaResponse>('/api/chat/upload', {
    conversation_id: conversationId,
    data: base64Data,
    filename,
    mime_type: mimeType,
  });
}

// ═══════════════════════════════════════════════════════════════════════════
// WORK API — Projects
// ═══════════════════════════════════════════════════════════════════════════

export async function listProjects(): Promise<WorkProject[]> {
  return workPost<WorkProject[]>('/projects/list', { active_only: true, limit: 100 });
}

export async function getProject(projectId: number): Promise<WorkProject> {
  return workPost<WorkProject>('/projects/get', { project_id: projectId });
}

export async function createProject(
  name: string,
  description: string,
  tags: string[],
  gitRemoteUrl?: string,
): Promise<WorkProject> {
  const body: Record<string, unknown> = { name, description, tags };
  if (gitRemoteUrl) body.git_remote_url = gitRemoteUrl;
  return workPost<WorkProject>('/projects/create', body);
}

// ═══════════════════════════════════════════════════════════════════════════
// WORK API — Tasks
// ═══════════════════════════════════════════════════════════════════════════

export async function listTasks(
  projectId?: number,
  statusFilter?: string[],
): Promise<WorkTask[]> {
  const body: Record<string, unknown> = { limit: 200 };
  if (projectId != null) body.project_id = projectId;
  if (statusFilter && statusFilter.length > 0) body.status = statusFilter;
  return workPost<WorkTask[]>('/tasks/list', body);
}

export async function getTask(taskId: number): Promise<WorkTask> {
  return workPost<WorkTask>('/tasks/get', { task_id: taskId });
}

export async function createTask(
  projectId: number,
  title: string,
  description: string,
  priority: string,
  tags: string[],
): Promise<WorkTask> {
  return workPost<WorkTask>('/tasks/create', {
    project_id: projectId, title, description, priority, tags,
  });
}

export async function updateTask(
  taskId: number,
  fields: {
    title?: string;
    description?: string;
    status?: string;
    priority?: string;
    tags?: string[];
    comment?: string;
  },
): Promise<WorkTask> {
  return workPost<WorkTask>('/tasks/update', { task_id: taskId, ...fields });
}

export async function getTaskAnalytics(projectId?: number): Promise<TaskAnalytics> {
  const body: Record<string, unknown> = {};
  if (projectId != null) body.project_id = projectId;
  return workPost<TaskAnalytics>('/tasks/analytics', body);
}

export async function takeNextTask(projectId: number, force: boolean): Promise<void> {
  await postExpectVoid('/api/work/tasks/take-next', { project_id: projectId, force });
}

export async function takeNextReviewTask(projectId: number, force: boolean): Promise<void> {
  await postExpectVoid('/api/work/tasks/take-next-review', { project_id: projectId, force });
}

export async function moveTaskToTopOrBottom(taskId: number, position: string): Promise<WorkTask> {
  return workPost<WorkTask>('/tasks/move', { task_id: taskId, position });
}

export async function rejectReview(taskId: number, reviewerComment: string): Promise<WorkTask> {
  return workPost<WorkTask>('/tasks/reject-review', { task_id: taskId, reviewer_comment: reviewerComment });
}

// ═══════════════════════════════════════════════════════════════════════════
// WORK API — Documents
// ═══════════════════════════════════════════════════════════════════════════

export async function searchDocuments(
  query: string,
  projectId?: number,
): Promise<WorkDocument[]> {
  const body: Record<string, unknown> = { query, limit: 50 };
  if (projectId != null) body.project_id = projectId;
  return workPost<WorkDocument[]>('/documents/search', body);
}

export async function getDocument(documentId: number): Promise<WorkDocument> {
  return workPost<WorkDocument>('/documents/get', { document_id: documentId });
}

export async function createDocument(
  projectId: number,
  title: string,
  content: string,
  docType: string,
): Promise<WorkDocument> {
  return workPost<WorkDocument>('/documents/create', {
    project_id: projectId, title, content, type: docType,
  });
}

// ═══════════════════════════════════════════════════════════════════════════
// WORK API — Comments
// ═══════════════════════════════════════════════════════════════════════════

export async function listComments(taskId: number): Promise<WorkComment[]> {
  return workPost<WorkComment[]>('/comments/list', { task_id: taskId, limit: 100 });
}

export async function listCommentsForDocument(documentId: number): Promise<WorkComment[]> {
  return workPost<WorkComment[]>('/comments/list', { document_id: documentId, limit: 100 });
}

export async function upsertComment(fields: {
  comment_id?: number;
  task_id?: number;
  document_id?: number;
  content: string;
  parent_comment_id?: number;
}): Promise<WorkComment> {
  return workPost<WorkComment>('/comments/upsert', fields);
}

// ═══════════════════════════════════════════════════════════════════════════
// WORK API — Activity
// ═══════════════════════════════════════════════════════════════════════════

export async function getRecentActivity(limit: number, projectId?: number): Promise<ActivityLog[]> {
  const body: Record<string, unknown> = { limit };
  if (projectId != null) body.project_id = projectId;
  return workPost<ActivityLog[]>('/activity/recent', body);
}

// ═══════════════════════════════════════════════════════════════════════════
// WORK API — Live Board
// ═══════════════════════════════════════════════════════════════════════════

export async function getLiveBoard(): Promise<LiveBoard> {
  return workPost<LiveBoard>('/live-board/get', { backlog_limit: 50 });
}

export async function selectTasks(taskIds: number[]): Promise<LiveBoardSelection[]> {
  return workPost<LiveBoardSelection[]>('/live-board/select', { task_ids: taskIds });
}

export async function deselectTask(taskId: number): Promise<void> {
  await postExpectVoid('/api/work/live-board/deselect', { task_id: taskId });
}

export async function clearCompleted(): Promise<number> {
  const res = await post<{ data: { cleared: number } }>('/api/work/live-board/clear-completed', {});
  return res.data.cleared;
}

export async function moveSelection(taskId: number, position: string): Promise<void> {
  await postExpectVoid('/api/work/live-board/move', { task_id: taskId, position });
}

export async function startAgentLoop(): Promise<string> {
  const res = await post<{ message: string }>('/api/work/live-board/agent/start', {});
  return res.message;
}

export async function stopAgentLoop(): Promise<string> {
  const res = await post<{ message: string }>('/api/work/live-board/agent/stop', {});
  return res.message;
}

export async function ensureAgentLoop(): Promise<string> {
  const res = await post<{ message: string }>('/api/work/live-board/agent/ensure', { auto_select_from_todo: true });
  return res.message;
}

// ═══════════════════════════════════════════════════════════════════════════
// VOICE API
// ═══════════════════════════════════════════════════════════════════════════

export async function transcribeAudio(audioData: string, mimeType: string): Promise<TranscribeResult> {
  return post<TranscribeResult>('/api/voice/transcribe', { audio_data: audioData, mime_type: mimeType });
}

export async function formatTranscription(
  transcription: string,
  mode: string,
  existingContent?: string,
): Promise<FormatResult> {
  const body: Record<string, string> = { transcription, mode };
  if (existingContent) body.existing_content = existingContent;
  return post<FormatResult>('/api/voice/format', body);
}

// ═══════════════════════════════════════════════════════════════════════════
// EXPORT ERROR CLASS
// ═══════════════════════════════════════════════════════════════════════════

export { ApiError };
