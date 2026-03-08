/**
 * API Contract Verification Tests
 *
 * These tests verify that every function in api.ts calls the correct URL
 * with the correct HTTP method and request body field names, matching
 * the Rust backend's router.rs exactly.
 *
 * They intercept global fetch to assert on the request, then return
 * a minimal valid response so the function resolves without error.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

type FetchCall = {
  url: string;
  method: string;
  body: unknown;
  credentials: string | undefined;
};

let fetchCalls: FetchCall[] = [];
let mockResponseBody: unknown = {};

function setMockResponse(body: unknown) {
  mockResponseBody = body;
}

const originalFetch = globalThis.fetch;

beforeEach(() => {
  fetchCalls = [];
  mockResponseBody = {};
  globalThis.fetch = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = typeof input === 'string' ? input : input.toString();
    const method = (init?.method ?? 'GET').toUpperCase();
    let body: unknown = undefined;
    if (init?.body && typeof init.body === 'string') {
      try { body = JSON.parse(init.body); } catch { body = init.body; }
    }
    fetchCalls.push({
      url,
      method,
      body,
      credentials: init?.credentials,
    });
    return new Response(JSON.stringify(mockResponseBody), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  }) as typeof fetch;
});

afterEach(() => {
  globalThis.fetch = originalFetch;
});

function lastCall(): FetchCall {
  const call = fetchCalls[fetchCalls.length - 1];
  if (!call) throw new Error('No fetch calls recorded');
  return call;
}

// ═══════════════════════════════════════════════════════════════════════════
// Import all API functions
// ═══════════════════════════════════════════════════════════════════════════
import * as api from './api';

// ═══════════════════════════════════════════════════════════════════════════
// CORE ENDPOINTS — matching router.rs lines 173-185
// ═══════════════════════════════════════════════════════════════════════════

describe('Core API endpoints', () => {
  it('GET /api/status', async () => {
    setMockResponse({ status: 'ok', version: '1.0' });
    await api.getStatus();
    expect(lastCall().url).toBe('/api/status');
    expect(lastCall().method).toBe('GET');
    expect(lastCall().credentials).toBe('include');
  });

  it('GET /api/feed', async () => {
    setMockResponse({ pending: [], pending_count: 0, running: null, recent_completed: [], completed_count: 0 });
    const res = await api.getFeed();
    expect(lastCall().url).toBe('/api/feed');
    expect(lastCall().method).toBe('GET');
    expect(res).toHaveProperty('pending_count');
  });

  it('GET /api/responses', async () => {
    setMockResponse({ pending: [], pending_count: 0, recent_sent: [], sent_count: 0, recent_failed: [], failed_count: 0 });
    const res = await api.getResponses();
    expect(lastCall().url).toBe('/api/responses');
    expect(lastCall().method).toBe('GET');
    expect(res).toHaveProperty('sent_count');
  });

  it('GET /api/chats — unwraps { chats: [...] }', async () => {
    setMockResponse({ chats: [{ chat_id: '123', message_count: 5 }] });
    const res = await api.getChats();
    expect(lastCall().url).toBe('/api/chats');
    expect(res).toHaveLength(1);
    expect(res[0]?.chat_id).toBe('123');
  });

  it('GET /api/messages/:chatId with query params', async () => {
    setMockResponse({ messages: [], total: 0, page: 0, page_size: 50, total_pages: 0 });
    await api.getMessages('chat-42', 2, 50, 'hello', 'topic-1');
    expect(lastCall().url).toBe('/api/messages/chat-42?page=2&page_size=50&search=hello&topic_id=topic-1');
    expect(lastCall().method).toBe('GET');
  });

  it('GET /api/messages/:chatId — omits empty search', async () => {
    setMockResponse({ messages: [], total: 0, page: 0, page_size: 50, total_pages: 0 });
    await api.getMessages('chat-42', 0, 50);
    expect(lastCall().url).toBe('/api/messages/chat-42?page=0&page_size=50');
  });

  it('GET /api/logs with query params', async () => {
    setMockResponse({ entries: [], total: 0, page: 0, page_size: 100, total_pages: 0 });
    await api.getLogs(1, 100, 'error');
    expect(lastCall().url).toBe('/api/logs?page=1&page_size=100&search=error');
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// SETTINGS — matching router.rs line 180-183
// ═══════════════════════════════════════════════════════════════════════════

describe('Settings endpoints', () => {
  it('GET /api/settings', async () => {
    setMockResponse({ show_tool_messages: false, show_thinking_messages: false, show_tool_results: false, omp_num_threads: 2, allowed_username: null, chat_harness: 'claude', claude_model: 'claude-opus-4-6', dev_role_prompt: '', harden_role_prompt: '', pm_role_prompt: '' });
    await api.getSettings();
    expect(lastCall().url).toBe('/api/settings');
    expect(lastCall().method).toBe('GET');
  });

  it('PUT /api/settings — sends snake_case field names', async () => {
    setMockResponse({});
    const settings = {
      show_tool_messages: true,
      show_thinking_messages: false,
      show_tool_results: true,
      omp_num_threads: 4,
      allowed_username: 'testuser',
      chat_harness: 'codex',
      claude_model: 'claude-opus-4-6',
      dev_role_prompt: 'dev prompt',
      harden_role_prompt: 'harden prompt',
      pm_role_prompt: 'pm prompt',
    };
    await api.putSettings(settings);
    expect(lastCall().url).toBe('/api/settings');
    expect(lastCall().method).toBe('PUT');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.show_tool_messages).toBe(true);
    expect(body.show_tool_results).toBe(true);
    expect(body.omp_num_threads).toBe(4);
    expect(body.allowed_username).toBe('testuser');
    expect(body.chat_harness).toBe('codex');
    expect(body.claude_model).toBe('claude-opus-4-6');
    expect(body.dev_role_prompt).toBe('dev prompt');
    expect(body.harden_role_prompt).toBe('harden prompt');
    expect(body.pm_role_prompt).toBe('pm prompt');
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// CRON — matching router.rs lines 158-170
// ═══════════════════════════════════════════════════════════════════════════

describe('Cron endpoints', () => {
  it('GET /api/cron/jobs — unwraps { jobs: [...] }', async () => {
    setMockResponse({ jobs: [{ id: 'j1', schedule: '* * * * *', status: 'active' }] });
    const res = await api.getCronJobs();
    expect(lastCall().url).toBe('/api/cron/jobs');
    expect(res).toHaveLength(1);
  });

  it('GET /api/cron/status', async () => {
    setMockResponse({ active_jobs: 2, paused_jobs: 1, waiting_executions: 0 });
    const res = await api.getCronStatus();
    expect(lastCall().url).toBe('/api/cron/status');
    expect(res.active_jobs).toBe(2);
  });

  it('POST /api/cron/jobs/:id/pause', async () => {
    setMockResponse({ id: 'j1', status: 'paused' });
    await api.pauseCronJob('j1');
    expect(lastCall().url).toBe('/api/cron/jobs/j1/pause');
    expect(lastCall().method).toBe('POST');
  });

  it('POST /api/cron/jobs/:id/resume', async () => {
    setMockResponse({ id: 'j1', status: 'active' });
    await api.resumeCronJob('j1');
    expect(lastCall().url).toBe('/api/cron/jobs/j1/resume');
    expect(lastCall().method).toBe('POST');
  });

  it('DELETE /api/cron/jobs/:id', async () => {
    setMockResponse({ id: 'j1', status: 'cancelled' });
    await api.cancelCronJob('j1');
    expect(lastCall().url).toBe('/api/cron/jobs/j1');
    expect(lastCall().method).toBe('DELETE');
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// SETUP — matching router.rs lines 193-209
// ═══════════════════════════════════════════════════════════════════════════

describe('Setup endpoints', () => {
  it('GET /api/setup/status', async () => {
    setMockResponse({ data_dir: '/tmp', has_telegram_token: false, has_gemini_key: false, has_claude_cli: false, has_allowed_username: false, has_threading_enabled: false, is_complete: false, platform: 'macos' });
    await api.getSetupStatus();
    expect(lastCall().url).toBe('/api/setup/status');
    expect(lastCall().method).toBe('GET');
  });

  it('POST /api/setup/telegram — sends { token }', async () => {
    setMockResponse({ success: true });
    await api.postTelegramToken('abc123');
    expect(lastCall().url).toBe('/api/setup/telegram');
    expect(lastCall().method).toBe('POST');
    expect((lastCall().body as Record<string, unknown>).token).toBe('abc123');
  });

  it('POST /api/setup/gemini — sends { key }', async () => {
    setMockResponse({ success: true });
    await api.postGeminiKey('key123');
    expect(lastCall().url).toBe('/api/setup/gemini');
    expect((lastCall().body as Record<string, unknown>).key).toBe('key123');
  });

  it('POST /api/setup/install-claude', async () => {
    setMockResponse({ success: true });
    await api.postInstallClaude();
    expect(lastCall().url).toBe('/api/setup/install-claude');
    expect(lastCall().method).toBe('POST');
  });

  it('GET /api/setup/claude-auth', async () => {
    setMockResponse({ installed: true, authenticated: false, needs_update: false });
    await api.getClaudeAuth();
    expect(lastCall().url).toBe('/api/setup/claude-auth');
    expect(lastCall().method).toBe('GET');
  });

  it('POST /api/setup/update-claude', async () => {
    setMockResponse({ success: true });
    await api.postUpdateClaude();
    expect(lastCall().url).toBe('/api/setup/update-claude');
  });

  it('POST /api/setup/test-claude', async () => {
    setMockResponse({ success: true });
    await api.postTestClaude();
    expect(lastCall().url).toBe('/api/setup/test-claude');
  });

  it('POST /api/setup/check-threading', async () => {
    setMockResponse({ success: true, enabled: false });
    await api.checkThreading();
    expect(lastCall().url).toBe('/api/setup/check-threading');
  });

  it('GET /api/setup/api-keys', async () => {
    setMockResponse({ has_telegram_token: false, has_gemini_key: false });
    await api.getApiKeys();
    expect(lastCall().url).toBe('/api/setup/api-keys');
    expect(lastCall().method).toBe('GET');
  });

  it('PUT /api/setup/api-keys — sends snake_case fields', async () => {
    setMockResponse({ success: true, telegram_updated: true, gemini_updated: false });
    await api.putApiKeys('tok123', 'gem456');
    expect(lastCall().url).toBe('/api/setup/api-keys');
    expect(lastCall().method).toBe('PUT');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.telegram_token).toBe('tok123');
    expect(body.gemini_key).toBe('gem456');
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// SEMANTIC — matching router.rs lines 226-231
// ═══════════════════════════════════════════════════════════════════════════

describe('Semantic endpoints', () => {
  it('GET /api/semantic/status', async () => {
    setMockResponse({ enabled: true, memory: {}, conversations: {} });
    await api.getSemanticStatus();
    expect(lastCall().url).toBe('/api/semantic/status');
  });

  it('POST /api/semantic/toggle — sends { enabled }', async () => {
    setMockResponse({ enabled: false });
    await api.toggleSemantic(false);
    expect(lastCall().url).toBe('/api/semantic/toggle');
    expect((lastCall().body as Record<string, unknown>).enabled).toBe(false);
  });

  it('POST /api/semantic/reindex', async () => {
    setMockResponse({});
    await api.triggerSemanticReindex();
    expect(lastCall().url).toBe('/api/semantic/reindex');
    expect(lastCall().method).toBe('POST');
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// TUNNEL — matching router.rs lines 343-348
// ═══════════════════════════════════════════════════════════════════════════

describe('Tunnel endpoint', () => {
  it('GET /api/tunnel/status', async () => {
    setMockResponse({ active: false, url: null, qr_svg: null });
    await api.getTunnelStatus();
    expect(lastCall().url).toBe('/api/tunnel/status');
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// CHAT — matching router.rs lines 301-321
// ═══════════════════════════════════════════════════════════════════════════

describe('Chat endpoints', () => {
  it('GET /api/chat/conversations — unwraps { conversations: [...] }', async () => {
    setMockResponse({ conversations: [{ id: 'c1', name: 'Test', updated_at: '2024-01-01' }] });
    const res = await api.getConversations();
    expect(lastCall().url).toBe('/api/chat/conversations');
    expect(res).toHaveLength(1);
  });

  it('POST /api/chat/conversations (create)', async () => {
    setMockResponse({ conversation_id: 'new-id' });
    const res = await api.createConversation();
    expect(lastCall().url).toBe('/api/chat/conversations');
    expect(lastCall().method).toBe('POST');
    expect(res.conversation_id).toBe('new-id');
  });

  it('DELETE /api/chat/conversations/:id', async () => {
    setMockResponse({});
    await api.deleteConversation('conv-123');
    expect(lastCall().url).toBe('/api/chat/conversations/conv-123');
    expect(lastCall().method).toBe('DELETE');
  });

  it('PUT /api/chat/conversations/:id/name — sends { name }', async () => {
    setMockResponse({});
    await api.renameConversation('conv-123', 'New Name');
    expect(lastCall().url).toBe('/api/chat/conversations/conv-123/name');
    expect(lastCall().method).toBe('PUT');
    expect((lastCall().body as Record<string, unknown>).name).toBe('New Name');
  });

  it('POST /api/chat/send — sends { conversation_id, content }', async () => {
    setMockResponse({ message_id: 'msg1', status: 'sent' });
    await api.sendChatMessage('conv-123', 'Hello');
    expect(lastCall().url).toBe('/api/chat/send');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.conversation_id).toBe('conv-123');
    expect(body.content).toBe('Hello');
  });

  it('POST /api/chat/upload — sends { conversation_id, data, filename, mime_type }', async () => {
    setMockResponse({ success: true });
    await api.uploadChatMedia('conv-123', 'base64data', 'audio.webm', 'audio/webm');
    expect(lastCall().url).toBe('/api/chat/upload');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.conversation_id).toBe('conv-123');
    expect(body.data).toBe('base64data');
    expect(body.filename).toBe('audio.webm');
    expect(body.mime_type).toBe('audio/webm');
  });

  it('GET /api/chat/messages/:conversation_id — parses direction', async () => {
    setMockResponse({ messages: [
      { id: 'm1', direction: 'outbound', content: 'hi', timestamp: '2024-01-01T00:00:00Z' },
      { id: 'm2', direction: 'inbound', content: 'hello', timestamp: '2024-01-01T00:00:01Z' },
    ]});
    const res = await api.getChatMessages('conv-123');
    expect(lastCall().url).toBe('/api/chat/messages/conv-123');
    expect(res[0]?.direction).toBe('outbound');
    expect(res[1]?.direction).toBe('inbound');
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// WORK API — matching router.rs lines 237-284
// All POST to /api/work/...
// ═══════════════════════════════════════════════════════════════════════════

describe('Work API — Projects', () => {
  it('POST /api/work/projects/list — sends { active_only, limit }', async () => {
    setMockResponse({ data: [] });
    await api.listProjects();
    expect(lastCall().url).toBe('/api/work/projects/list');
    expect(lastCall().method).toBe('POST');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.active_only).toBe(true);
    expect(body.limit).toBe(100);
  });

  it('POST /api/work/projects/get — sends { project_id }', async () => {
    setMockResponse({ data: { id: 1, name: 'Test' } });
    await api.getProject(42);
    expect(lastCall().url).toBe('/api/work/projects/get');
    expect((lastCall().body as Record<string, unknown>).project_id).toBe(42);
  });

  it('POST /api/work/projects/create — sends correct fields', async () => {
    setMockResponse({ data: { id: 1 } });
    await api.createProject('Proj', 'Desc', ['tag1'], 'https://git.example.com');
    expect(lastCall().url).toBe('/api/work/projects/create');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.name).toBe('Proj');
    expect(body.description).toBe('Desc');
    expect(body.tags).toEqual(['tag1']);
    expect(body.git_remote_url).toBe('https://git.example.com');
  });
});

describe('Work API — Tasks', () => {
  it('POST /api/work/tasks/list — sends { project_id, status, limit }', async () => {
    setMockResponse({ data: [] });
    await api.listTasks(5, ['todo', 'in_progress']);
    expect(lastCall().url).toBe('/api/work/tasks/list');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.project_id).toBe(5);
    expect(body.status).toEqual(['todo', 'in_progress']);
    expect(body.limit).toBe(200);
  });

  it('POST /api/work/tasks/get — sends { task_id }', async () => {
    setMockResponse({ data: { id: 10 } });
    await api.getTask(10);
    expect((lastCall().body as Record<string, unknown>).task_id).toBe(10);
  });

  it('POST /api/work/tasks/create — sends all required fields', async () => {
    setMockResponse({ data: { id: 11 } });
    await api.createTask(1, 'Title', 'Desc', 'high', ['frontend']);
    expect(lastCall().url).toBe('/api/work/tasks/create');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.project_id).toBe(1);
    expect(body.title).toBe('Title');
    expect(body.description).toBe('Desc');
    expect(body.priority).toBe('high');
    expect(body.tags).toEqual(['frontend']);
  });

  it('POST /api/work/tasks/update — sends { task_id, ...optional fields }', async () => {
    setMockResponse({ data: { id: 11 } });
    await api.updateTask(11, { status: 'done', comment: 'Finished' });
    expect(lastCall().url).toBe('/api/work/tasks/update');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.task_id).toBe(11);
    expect(body.status).toBe('done');
    expect(body.comment).toBe('Finished');
  });

  it('POST /api/work/tasks/analytics', async () => {
    setMockResponse({ data: { status_counts: [], throughput_30d: [] } });
    await api.getTaskAnalytics(5);
    expect(lastCall().url).toBe('/api/work/tasks/analytics');
    expect((lastCall().body as Record<string, unknown>).project_id).toBe(5);
  });

  it('POST /api/work/tasks/take-next — sends { project_id, force }', async () => {
    setMockResponse({});
    await api.takeNextTask(3, true);
    expect(lastCall().url).toBe('/api/work/tasks/take-next');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.project_id).toBe(3);
    expect(body.force).toBe(true);
  });

  it('POST /api/work/tasks/take-next-review', async () => {
    setMockResponse({});
    await api.takeNextReviewTask(3, false);
    expect(lastCall().url).toBe('/api/work/tasks/take-next-review');
  });

  it('POST /api/work/tasks/move — sends { task_id, position }', async () => {
    setMockResponse({ data: { id: 11 } });
    await api.moveTaskToTopOrBottom(11, 'top');
    expect(lastCall().url).toBe('/api/work/tasks/move');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.task_id).toBe(11);
    expect(body.position).toBe('top');
  });

  it('POST /api/work/tasks/reject-review — sends { task_id, reviewer_comment }', async () => {
    setMockResponse({ data: { id: 11 } });
    await api.rejectReview(11, 'Needs changes');
    expect(lastCall().url).toBe('/api/work/tasks/reject-review');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.task_id).toBe(11);
    expect(body.reviewer_comment).toBe('Needs changes');
  });
});

describe('Work API — Documents', () => {
  it('POST /api/work/documents/search — sends { query, project_id, limit }', async () => {
    setMockResponse({ data: [] });
    await api.searchDocuments('test', 5);
    expect(lastCall().url).toBe('/api/work/documents/search');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.query).toBe('test');
    expect(body.project_id).toBe(5);
    expect(body.limit).toBe(50);
  });

  it('POST /api/work/documents/get — sends { document_id }', async () => {
    setMockResponse({ data: { id: 20 } });
    await api.getDocument(20);
    expect((lastCall().body as Record<string, unknown>).document_id).toBe(20);
  });

  it('POST /api/work/documents/create — sends { project_id, title, content, type }', async () => {
    setMockResponse({ data: { id: 21 } });
    await api.createDocument(1, 'Doc Title', 'Content here', 'plan');
    expect(lastCall().url).toBe('/api/work/documents/create');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.project_id).toBe(1);
    expect(body.title).toBe('Doc Title');
    expect(body.content).toBe('Content here');
    expect(body.type).toBe('plan');
  });
});

describe('Work API — Comments', () => {
  it('POST /api/work/comments/list for task — sends { task_id, limit }', async () => {
    setMockResponse({ data: [] });
    await api.listComments(10);
    expect(lastCall().url).toBe('/api/work/comments/list');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.task_id).toBe(10);
    expect(body.limit).toBe(100);
  });

  it('POST /api/work/comments/list for document — sends { document_id, limit }', async () => {
    setMockResponse({ data: [] });
    await api.listCommentsForDocument(20);
    const body = lastCall().body as Record<string, unknown>;
    expect(body.document_id).toBe(20);
    expect(body.limit).toBe(100);
  });

  it('POST /api/work/comments/upsert — sends correct fields', async () => {
    setMockResponse({ data: { id: 30 } });
    await api.upsertComment({ task_id: 10, content: 'A comment', parent_comment_id: 5 });
    expect(lastCall().url).toBe('/api/work/comments/upsert');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.task_id).toBe(10);
    expect(body.content).toBe('A comment');
    expect(body.parent_comment_id).toBe(5);
  });
});

describe('Work API — Activity', () => {
  it('POST /api/work/activity/recent — sends { limit }', async () => {
    setMockResponse({ data: [] });
    await api.getRecentActivity(20);
    expect(lastCall().url).toBe('/api/work/activity/recent');
    expect((lastCall().body as Record<string, unknown>).limit).toBe(20);
  });
});

describe('Work API — Live Board', () => {
  it('POST /api/work/live-board/get — sends { backlog_limit }', async () => {
    setMockResponse({ data: { backlog: [], selected: [], stats: {} } });
    await api.getLiveBoard();
    expect(lastCall().url).toBe('/api/work/live-board/get');
    expect((lastCall().body as Record<string, unknown>).backlog_limit).toBe(50);
  });

  it('POST /api/work/live-board/select — sends { task_ids }', async () => {
    setMockResponse({ data: [] });
    await api.selectTasks([1, 2, 3]);
    expect(lastCall().url).toBe('/api/work/live-board/select');
    expect((lastCall().body as Record<string, unknown>).task_ids).toEqual([1, 2, 3]);
  });

  it('POST /api/work/live-board/deselect — sends { task_id }', async () => {
    setMockResponse({});
    await api.deselectTask(5);
    expect(lastCall().url).toBe('/api/work/live-board/deselect');
    expect((lastCall().body as Record<string, unknown>).task_id).toBe(5);
  });

  it('POST /api/work/live-board/clear-completed', async () => {
    setMockResponse({ data: { cleared: 3 } });
    const cleared = await api.clearCompleted();
    expect(lastCall().url).toBe('/api/work/live-board/clear-completed');
    expect(cleared).toBe(3);
  });

  it('POST /api/work/live-board/move — sends { task_id, position }', async () => {
    setMockResponse({});
    await api.moveSelection(5, 'up');
    expect(lastCall().url).toBe('/api/work/live-board/move');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.task_id).toBe(5);
    expect(body.position).toBe('up');
  });

  it('POST /api/work/live-board/agent/start', async () => {
    setMockResponse({ message: 'started' });
    const res = await api.startAgentLoop();
    expect(lastCall().url).toBe('/api/work/live-board/agent/start');
    expect(res).toBe('started');
  });

  it('POST /api/work/live-board/agent/stop', async () => {
    setMockResponse({ message: 'stopped' });
    await api.stopAgentLoop();
    expect(lastCall().url).toBe('/api/work/live-board/agent/stop');
  });

  it('POST /api/work/live-board/agent/ensure — sends { auto_select_from_todo }', async () => {
    setMockResponse({ message: 'ensured' });
    await api.ensureAgentLoop();
    expect(lastCall().url).toBe('/api/work/live-board/agent/ensure');
    expect((lastCall().body as Record<string, unknown>).auto_select_from_todo).toBe(true);
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// VOICE — matching router.rs lines 291-297
// ═══════════════════════════════════════════════════════════════════════════

describe('Voice endpoints', () => {
  it('POST /api/voice/transcribe — sends { audio_data, mime_type }', async () => {
    setMockResponse({ success: true, transcription: 'hello world' });
    await api.transcribeAudio('base64audio', 'audio/webm');
    expect(lastCall().url).toBe('/api/voice/transcribe');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.audio_data).toBe('base64audio');
    expect(body.mime_type).toBe('audio/webm');
  });

  it('POST /api/voice/format — sends { transcription, mode, existing_content? }', async () => {
    setMockResponse({ success: true, formatted: '## Hello' });
    await api.formatTranscription('hello world', 'ticket');
    expect(lastCall().url).toBe('/api/voice/format');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.transcription).toBe('hello world');
    expect(body.mode).toBe('ticket');
    expect(body).not.toHaveProperty('existing_content');
  });

  it('POST /api/voice/format with existing_content', async () => {
    setMockResponse({ success: true });
    await api.formatTranscription('addition', 'edit', 'existing text');
    const body = lastCall().body as Record<string, unknown>;
    expect(body.existing_content).toBe('existing text');
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// CREDENTIALS CHECK — every call must include credentials: 'include'
// ═══════════════════════════════════════════════════════════════════════════

describe('Auth: credentials included on every request', () => {
  it('all fetch calls use credentials: include', async () => {
    setMockResponse({ status: 'ok', version: '1.0' });
    await api.getStatus();
    setMockResponse({ pending: [], pending_count: 0, running: null, recent_completed: [], completed_count: 0 });
    await api.getFeed();
    setMockResponse({ data: [] });
    await api.listProjects();

    for (const call of fetchCalls) {
      expect(call.credentials).toBe('include');
    }
  });
});

// ═══════════════════════════════════════════════════════════════════════════
// ROUTE COMPLETENESS — verify every route from the Elm frontend has a
// matching API function
// ═══════════════════════════════════════════════════════════════════════════

describe('Route completeness: all Elm Api.elm endpoints have React equivalents', () => {
  const expectedEndpoints = [
    'getStatus', 'getFeed', 'getResponses', 'getChats', 'getMessages', 'getLogs',
    'getSemanticStatus', 'toggleSemantic', 'triggerSemanticReindex',
    'getTunnelStatus',
    'getSetupStatus', 'postTelegramToken', 'postGeminiKey', 'postInstallClaude',
    'getClaudeAuth', 'postUpdateClaude', 'postTestClaude', 'checkThreading',
    'getApiKeys', 'putApiKeys',
    'getSettings', 'putSettings',
    'getCronJobs', 'getCronStatus', 'pauseCronJob', 'resumeCronJob', 'cancelCronJob',
    'getConversations', 'getChatMessages', 'createConversation', 'sendChatMessage',
    'renameConversation', 'deleteConversation', 'uploadChatMedia',
    'listProjects', 'getProject', 'createProject',
    'listTasks', 'getTask', 'createTask', 'updateTask', 'getTaskAnalytics',
    'takeNextTask', 'takeNextReviewTask', 'moveTaskToTopOrBottom', 'rejectReview',
    'searchDocuments', 'getDocument', 'createDocument',
    'listComments', 'listCommentsForDocument', 'upsertComment',
    'getRecentActivity',
    'getLiveBoard', 'selectTasks', 'deselectTask', 'clearCompleted', 'moveSelection',
    'startAgentLoop', 'stopAgentLoop', 'ensureAgentLoop',
    'transcribeAudio', 'formatTranscription',
  ];

  for (const name of expectedEndpoints) {
    it(`api.${name} exists and is a function`, () => {
      expect(typeof (api as Record<string, unknown>)[name]).toBe('function');
    });
  }
});
