// ═══════════════════════════════════════════════════════════════════════════
// REMOTE DATA
// ═══════════════════════════════════════════════════════════════════════════

export type RemoteData<T> =
  | { tag: 'NotAsked' }
  | { tag: 'Loading' }
  | { tag: 'Success'; data: T }
  | { tag: 'Failure'; error: string };

export const NotAsked: RemoteData<never> = { tag: 'NotAsked' };
export const Loading: RemoteData<never> = { tag: 'Loading' };
export function Success<T>(data: T): RemoteData<T> { return { tag: 'Success', data }; }
export function Failure(error: string): RemoteData<never> { return { tag: 'Failure', error }; }

export function isLoading<T>(rd: RemoteData<T>): boolean { return rd.tag === 'Loading'; }
export function withDefault<T>(def: T, rd: RemoteData<T>): T {
  return rd.tag === 'Success' ? rd.data : def;
}

// ═══════════════════════════════════════════════════════════════════════════
// FEED / PROMPTS
// ═══════════════════════════════════════════════════════════════════════════

export interface PromptItem {
  id: string;
  source_type: string;
  user_id: number;
  prompt: string;
  media_path: string | null;
  status: string;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  error: string | null;
}

export interface FeedData {
  pending: PromptItem[];
  pending_count: number;
  running: PromptItem | null;
  recent_completed: PromptItem[];
  completed_count: number;
}

// ═══════════════════════════════════════════════════════════════════════════
// RESPONSES
// ═══════════════════════════════════════════════════════════════════════════

export interface ResponseItem {
  id: string;
  prompt_id: string;
  source_type: string;
  user_id: number;
  content: string;
  is_partial: boolean;
  is_final: boolean;
  sequence: number;
  status: string;
  created_at: string;
  sent_at: string | null;
  next_attempt_at: string | null;
  error: string | null;
}

export interface ResponseFeedData {
  pending: ResponseItem[];
  pending_count: number;
  recent_sent: ResponseItem[];
  sent_count: number;
  recent_failed: ResponseItem[];
  failed_count: number;
}

// ═══════════════════════════════════════════════════════════════════════════
// MESSAGES (Telegram)
// ═══════════════════════════════════════════════════════════════════════════

export interface StoredMessage {
  id: string;
  chat_id: string;
  user_id: number | null;
  direction: string;
  content: string;
  media_type: string | null;
  media_path: string | null;
  timestamp: string;
}

export interface ChatSummary {
  chat_id: string;
  topic_id: number | null;
  username: string | null;
  display_name: string | null;
  message_count: number;
}

export interface MessagesPage {
  messages: StoredMessage[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

// ═══════════════════════════════════════════════════════════════════════════
// LOGS
// ═══════════════════════════════════════════════════════════════════════════

export interface LogEntry {
  timestamp: string;
  level: string;
  component: string;
  message: string;
}

export interface LogsPage {
  entries: LogEntry[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

// ═══════════════════════════════════════════════════════════════════════════
// SETTINGS
// ═══════════════════════════════════════════════════════════════════════════

export interface Settings {
  show_tool_messages: boolean;
  show_thinking_messages: boolean;
  show_tool_results: boolean;
  omp_num_threads: number;
  allowed_username: string | null;
  chat_harness: string;
  claude_model: string;
  dev_role_prompt: string;
  harden_role_prompt: string;
  pm_role_prompt: string;
}

export const defaultSettings: Settings = {
  show_tool_messages: false,
  show_thinking_messages: false,
  show_tool_results: false,
  omp_num_threads: 2,
  allowed_username: null,
  chat_harness: 'claude',
  claude_model: 'claude-opus-4-6',
  dev_role_prompt: '',
  harden_role_prompt: '',
  pm_role_prompt: '',
};

// ═══════════════════════════════════════════════════════════════════════════
// API KEYS
// ═══════════════════════════════════════════════════════════════════════════

export interface ApiKeyStatus {
  valid: boolean;
  error: string | null;
  info: string | null;
}

export interface ApiKeysData {
  has_telegram_token: boolean;
  telegram_token_masked: string | null;
  telegram_status: ApiKeyStatus | null;
  has_gemini_key: boolean;
  gemini_key_masked: string | null;
  gemini_status: ApiKeyStatus | null;
  claude_code_status: ClaudeCodeStatus | null;
  has_user_contacted: boolean | null;
}

export interface ClaudeCodeStatus {
  auth_mode: string;
  account_email: string | null;
  account_name: string | null;
  organization: string | null;
}

// ═══════════════════════════════════════════════════════════════════════════
// CRON JOBS
// ═══════════════════════════════════════════════════════════════════════════

export interface CronJob {
  id: string;
  name: string | null;
  schedule: string;
  status: string;
  deferrable: boolean;
  next_run: string | null;
  last_run: string | null;
  created_at: string;
}

export interface CronStatus {
  active_jobs: number;
  paused_jobs: number;
  waiting_executions: number;
}

// ═══════════════════════════════════════════════════════════════════════════
// SEMANTIC INDEXER
// ═══════════════════════════════════════════════════════════════════════════

export interface IndexerTaskStatus {
  activity: string;
  current_file: string | null;
  files_indexed: number;
  files_skipped: number;
  files_total: number | null;
  chunks_processed: number;
  chunks_total: number | null;
}

export interface SemanticStatus {
  enabled: boolean;
  memory: IndexerTaskStatus;
  conversations: IndexerTaskStatus;
  total_memory_chunks: number;
  total_memory_files: number;
  total_conversation_chunks: number;
  total_conversation_sessions: number;
  total_memory_files_available: number;
  total_conversation_files_available: number;
  memory_files_stale: number;
  conversation_files_stale: number;
  last_conversation_poll_at: number | null;
  conversation_poll_interval_secs: number;
}

// ═══════════════════════════════════════════════════════════════════════════
// TUNNEL
// ═══════════════════════════════════════════════════════════════════════════

export interface TunnelStatus {
  active: boolean;
  url: string | null;
  qr_svg: string | null;
}

// ═══════════════════════════════════════════════════════════════════════════
// SETUP
// ═══════════════════════════════════════════════════════════════════════════

export interface SetupStatusResponse {
  data_dir: string;
  has_telegram_token: boolean;
  has_gemini_key: boolean;
  has_claude_cli: boolean;
  claude_cli_version: string | null;
  has_allowed_username: boolean;
  has_threading_enabled: boolean;
  is_complete: boolean;
  platform: string;
  gemini_key_preview: string | null;
  allowed_username_value: string | null;
  bot_name: string | null;
}

export interface TelegramSetupResponse {
  success: boolean;
  bot_name: string | null;
  error: string | null;
}

export interface GeminiSetupResponse {
  success: boolean;
  error: string | null;
}

export interface ClaudeInstallResponse {
  success: boolean;
  version: string | null;
  error: string | null;
}

export interface ClaudeAuthCheckResponse {
  installed: boolean;
  version: string | null;
  authenticated: boolean;
  auth_mode: string | null;
  account_email: string | null;
  account_name: string | null;
  needs_update: boolean;
  latest_version: string | null;
  error: string | null;
}

export interface ClaudeTestResponse {
  success: boolean;
  output: string | null;
  error: string | null;
}

export interface ThreadingCheckResponse {
  success: boolean;
  enabled: boolean;
  error: string | null;
}

export interface UpdateApiKeysResponse {
  success: boolean;
  telegram_updated: boolean;
  gemini_updated: boolean;
  telegram_error: string | null;
  gemini_error: string | null;
}

// ═══════════════════════════════════════════════════════════════════════════
// SETUP STATUS (local UI state, derived from server response)
// ═══════════════════════════════════════════════════════════════════════════

export interface SetupStatus {
  dataDir: string;
  hasTelegramToken: boolean;
  hasGeminiKey: boolean;
  hasClaudeCli: boolean;
  claudeCliVersion: string | null;
  hasAllowedUsername: boolean;
  isComplete: boolean;
  platform: string;
  botName: string | null;
  telegramError: string | null;
  geminiError: string | null;
  claudeInstalling: boolean;
  claudeInstallError: string | null;
  allowedUsernameError: string | null;
  claudeAuthenticated: boolean;
  claudeAuthMode: string | null;
  claudeAccountEmail: string | null;
  claudeAccountName: string | null;
  claudeNeedsUpdate: boolean;
  claudeLatestVersion: string | null;
  claudeUpdating: boolean;
  claudeUpdateError: string | null;
  claudeTesting: boolean;
  claudeTestResult: boolean | null;
  claudeTestOutput: string | null;
  claudeTestError: string | null;
  claudeAuthChecking: boolean;
  hasThreadingEnabled: boolean;
  threadingChecking: boolean;
  threadingError: string | null;
  geminiKeyPreview: string | null;
  allowedUsernameValue: string | null;
}

// ═══════════════════════════════════════════════════════════════════════════
// CHAT TYPES
// ═══════════════════════════════════════════════════════════════════════════

export interface Conversation {
  id: string;
  name: string;
  custom_name: string | null;
  auto_name: string | null;
  display_name: string | null;
  protocol: string | null;
  last_message_preview: string | null;
  updated_at: string;
}

export type MessageDirection = 'inbound' | 'outbound';

export interface MediaAttachment {
  type: 'audio' | 'video' | 'image' | 'file';
  path: string;
  transcription?: string | null;
  description?: string | null;
  name?: string;
  mimeType?: string;
}

export interface ChatMessage {
  id: string;
  direction: MessageDirection;
  content: string;
  timestamp: string;
  attachments: MediaAttachment[];
}

export function mediaAttachmentFromServerFields(
  mediaType: string | null | undefined,
  mediaPath: string | null | undefined,
): MediaAttachment[] {
  if (!mediaType || !mediaPath) return [];
  if (mediaType.startsWith('audio') || mediaType === 'voice') {
    return [{ type: 'audio', path: mediaPath, transcription: null }];
  }
  if (mediaType.startsWith('video') || mediaType === 'video') {
    return [{ type: 'video', path: mediaPath, transcription: null }];
  }
  if (mediaType.startsWith('image') || mediaType === 'photo') {
    return [{ type: 'image', path: mediaPath, description: null }];
  }
  return [{ type: 'file', path: mediaPath, name: mediaPath, mimeType: mediaType }];
}

export interface CreateConversationResponse {
  conversation_id: string;
}

export interface SendMessageResponse {
  message_id: string;
  status: string;
}

export interface UploadMediaResponse {
  success: boolean;
  message_id: string | null;
  transcription: string | null;
  media_type: string | null;
  media_path: string | null;
  error: string | null;
}

// ═══════════════════════════════════════════════════════════════════════════
// CHAT STATE
// ═══════════════════════════════════════════════════════════════════════════

export type ComposingMode = 'text' | 'voice' | 'video';

export type ChatActivity =
  | { tag: 'idle' }
  | { tag: 'composing'; mode: ComposingMode }
  | { tag: 'sending'; content: string; pendingId: string }
  | { tag: 'transcribing' }
  | { tag: 'awaiting'; pendingId: string }
  | { tag: 'streaming'; buffer: string }
  | { tag: 'error'; info: ChatErrorInfo }
  | { tag: 'observing' };

export interface ChatErrorInfo {
  error: string;
  retryable: boolean;
  failedContent: string | null;
}

export type WsConnection =
  | { tag: 'disconnected' }
  | { tag: 'connected'; conversationId: string }
  | { tag: 'reconnecting'; conversationId: string; retries: number };

export type UploadStatus =
  | { tag: 'uploading' }
  | { tag: 'succeeded'; transcription: string | null; mediaPath: string }
  | { tag: 'failed'; error: string };

export interface UploadTask {
  id: string;
  media: MediaUploadPayload;
  status: UploadStatus;
}

export type MediaUploadPayload =
  | { type: 'voice'; data: string; mimeType: string }
  | { type: 'video'; data: string; mimeType: string }
  | { type: 'file'; data: string; name: string; mimeType: string };

export type MessageLoadState =
  | { tag: 'not_loaded' }
  | { tag: 'loading' }
  | { tag: 'loaded'; hasMore: boolean }
  | { tag: 'error'; message: string };

export interface ChatConversationState {
  messages: ChatMessage[];
  pendingOutbound: ChatMessage[];
  uploads: UploadTask[];
  activity: ChatActivity;
  connection: WsConnection;
  inputText: string;
  messageCounter: number;
  uploadCounter: number;
  messagesLoaded: MessageLoadState;
  lastChunkSequence: number;
}

export const emptyChatConversationState: ChatConversationState = {
  messages: [],
  pendingOutbound: [],
  uploads: [],
  activity: { tag: 'idle' },
  connection: { tag: 'disconnected' },
  inputText: '',
  messageCounter: 0,
  uploadCounter: 0,
  messagesLoaded: { tag: 'not_loaded' },
  lastChunkSequence: 0,
};

export interface ChatPageState {
  conversations: RemoteData<Conversation[]>;
  activeChatId: string | null;
  conversationStates: Record<string, ChatConversationState>;
  renamingConversationId: string | null;
  renameText: string;
  confirmingDeleteId: string | null;
}

export const emptyChatPageState: ChatPageState = {
  conversations: { tag: 'NotAsked' },
  activeChatId: null,
  conversationStates: {},
  renamingConversationId: null,
  renameText: '',
  confirmingDeleteId: null,
};

export function getConversationState(convId: string, page: ChatPageState): ChatConversationState {
  return page.conversationStates[convId] ?? emptyChatConversationState;
}

export function isUuidLike(s: string): boolean {
  return s.length > 20 && s.includes('-');
}

export function isTelegramConversation(convId: string, conversations: Conversation[]): boolean {
  return conversations.some(
    c => c.id === convId && (c.protocol === 'telegram' || (c.protocol == null && !isUuidLike(c.id)))
  );
}

// ═══════════════════════════════════════════════════════════════════════════
// WORK TYPES
// ═══════════════════════════════════════════════════════════════════════════

export interface WorkProject {
  id: number;
  name: string;
  description: string;
  git_remote_url: string | null;
  tags: string[];
  is_active: boolean;
  task_count: number;
  created_at: string;
  updated_at: string;
}

export interface WorkTask {
  id: number;
  project_id: number;
  status: string;
  priority: string;
  sort_order: number;
  title: string;
  description: string;
  tags: string[];
  completed_at: string | null;
  created_at: string;
  updated_at: string;
  blocked_by: number[];
  blocks: number[];
}

export interface WorkDocument {
  id: number;
  project_id: number;
  document_type: string;
  title: string;
  content: string;
  version: number;
  created_at: string;
  updated_at: string;
}

export interface WorkComment {
  id: number;
  task_id: number | null;
  document_id: number | null;
  parent_comment_id: number | null;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface ActivityLog {
  id: number;
  project_id: number | null;
  task_id: number | null;
  document_id: number | null;
  action: string;
  actor: string;
  details: string;
  created_at: string;
}

export interface LiveBoard {
  backlog: WorkTask[];
  selected: SelectedTask[];
  stats: LiveBoardStats;
}

export interface SelectedTask {
  selection: LiveBoardSelection;
  task: WorkTask;
  comments: WorkComment[];
}

export interface LiveBoardSelection {
  id: number;
  task_id: number;
  sort_order: number;
  selected_at: string;
  started_at: string | null;
  completed_at: string | null;
  status: string;
}

export interface LiveBoardStats {
  total_backlog: number;
  total_selected: number;
  queued: number;
  completed: number;
  failed: number;
  active: number | null;
  agent_loop_state: string;
}

export interface TaskAnalytics {
  status_counts: StatusCount[];
  avg_completion_hours: number | null;
  throughput_30d: DayCount[];
}

export interface StatusCount {
  status: string;
  count: number;
}

export interface DayCount {
  date: string;
  count: number;
}

// ═══════════════════════════════════════════════════════════════════════════
// VOICE TYPES
// ═══════════════════════════════════════════════════════════════════════════

export type VoiceRecordingState =
  | 'idle'
  | 'recording'
  | 'transcribing'
  | 'formatting'
  | { tag: 'done'; result: string }
  | { tag: 'error'; message: string };

export type VoiceMode = 'ticket' | 'edit' | 'comment';

export interface VoiceState {
  recordingState: VoiceRecordingState;
  mode: VoiceMode;
  transcription: string | null;
  existingContent: string | null;
}

export const emptyVoiceState: VoiceState = {
  recordingState: 'idle',
  mode: 'comment',
  transcription: null,
  existingContent: null,
};

// ═══════════════════════════════════════════════════════════════════════════
// WORK FORM TYPES
// ═══════════════════════════════════════════════════════════════════════════

export interface ProjectForm {
  name: string;
  description: string;
  tags: string;
  gitRemoteUrl: string;
}

export const emptyProjectForm: ProjectForm = { name: '', description: '', tags: '', gitRemoteUrl: '' };

export interface TaskForm {
  title: string;
  description: string;
  priority: string;
  status: string;
}

export const emptyTaskForm: TaskForm = { title: '', description: '', priority: 'medium', status: 'todo' };

export interface DocumentForm {
  title: string;
  content: string;
  documentType: string;
}

export const emptyDocumentForm: DocumentForm = { title: '', content: '', documentType: 'notes' };

export type EditingField =
  | { tag: 'none' }
  | { tag: 'title'; value: string }
  | { tag: 'description'; value: string }
  | { tag: 'priority' }
  | { tag: 'tags'; value: string };

export type TaskViewMode = 'list' | 'board';

export type ProjectTab = 'tasks' | 'documents' | 'activity';

export interface TaskFilters {
  statusFilter: string[];
}

export const emptyTaskFilters: TaskFilters = { statusFilter: [] };

// ═══════════════════════════════════════════════════════════════════════════
// VOICE API TYPES
// ═══════════════════════════════════════════════════════════════════════════

export interface TranscribeResult {
  success: boolean;
  transcription: string | null;
  error: string | null;
}

export interface FormatResult {
  success: boolean;
  formatted: string | null;
  title: string | null;
  error: string | null;
}

// ═══════════════════════════════════════════════════════════════════════════
// WEBSOCKET EVENT TYPES
// ═══════════════════════════════════════════════════════════════════════════

export type ChatWsEvent =
  | { type: 'connected'; conversation_id: string; last_seq: number }
  | { type: 'message_chunk'; conversation_id: string; content: string; sequence: number; is_final: boolean }
  | { type: 'conversation_renamed'; conversation_id: string; name: string }
  | { type: 'typing_indicator'; conversation_id: string; is_typing: boolean }
  | { type: 'transcribing'; conversation_id: string }
  | { type: 'message_updated'; conversation_id: string; message_id: string; content: string }
  | { type: 'file_message'; conversation_id: string; message_id: string; filename: string; media_path: string; mime_type: string; caption?: string }
  | { type: 'connection_state'; state: string };

// ═══════════════════════════════════════════════════════════════════════════
// SSE EVENT TYPES
// ═══════════════════════════════════════════════════════════════════════════

export type WorkEvent =
  | { type: 'selection_updated'; data: LiveBoardStats }
  | { type: 'agent_progress'; data: { task_id: number; message: string } };

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

export function mediaUrl(chatId: string, filename: string): string {
  return `/api/media/${chatId}/${filename}`;
}

export function filenameFromPath(path: string): string | null {
  const parts = path.split('/');
  return parts[parts.length - 1] ?? null;
}
