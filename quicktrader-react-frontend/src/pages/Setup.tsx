import React, { useEffect, useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  PageHeader,
  Card,
  FormField,
  InputField,
  Button,
  PrimaryButton,
  LoadingSpinner,
} from '../components/UI';
import {
  getSetupStatus,
  postTelegramToken,
  postGeminiKey,
  postInstallClaude,
  getClaudeAuth,
  postUpdateClaude,
  postTestClaude,
  checkThreading,
  getSettings,
  putSettings,
} from '../api';
import type {
  SetupStatusResponse,
  TelegramSetupResponse,
  GeminiSetupResponse,
  ClaudeInstallResponse,
  ClaudeAuthCheckResponse,
  ClaudeTestResponse,
  ThreadingCheckResponse,
} from '../types';
import type { RemoteData } from '../types';
import { NotAsked, Loading, Success, Failure } from '../types';
import { colors, fonts } from '../theme';

export function Setup(): React.ReactElement {
  const navigate = useNavigate();
  const [status, setStatus] = useState<RemoteData<SetupStatusResponse>>(NotAsked);

  const [telegramToken, setTelegramToken] = useState('');
  const [telegramLoading, setTelegramLoading] = useState(false);
  const [telegramError, setTelegramError] = useState<string | null>(null);
  const [telegramSuccess, setTelegramSuccess] = useState<string | null>(null);

  const [geminiKey, setGeminiKey] = useState('');
  const [geminiLoading, setGeminiLoading] = useState(false);
  const [geminiError, setGeminiError] = useState<string | null>(null);
  const [geminiSuccess, setGeminiSuccess] = useState<string | null>(null);

  const [claudeAuth, setClaudeAuth] = useState<RemoteData<ClaudeAuthCheckResponse>>(NotAsked);
  const [claudeInstallLoading, setClaudeInstallLoading] = useState(false);
  const [claudeInstallError, setClaudeInstallError] = useState<string | null>(null);
  const [claudeUpdateLoading, setClaudeUpdateLoading] = useState(false);
  const [claudeTestLoading, setClaudeTestLoading] = useState(false);
  const [claudeTestOutput, setClaudeTestOutput] = useState<string | null>(null);
  const [claudeTestError, setClaudeTestError] = useState<string | null>(null);

  const [allowedUsername, setAllowedUsername] = useState('');
  const [allowedUsernameLoading, setAllowedUsernameLoading] = useState(false);
  const [allowedUsernameError, setAllowedUsernameError] = useState<string | null>(null);

  const [threadingResult, setThreadingResult] = useState<RemoteData<ThreadingCheckResponse>>(NotAsked);
  const [threadingLoading, setThreadingLoading] = useState(false);

  const loadStatus = useCallback(async () => {
    setStatus(Loading);
    try {
      const data = await getSetupStatus();
      setStatus(Success(data));
      setAllowedUsername(data.allowed_username_value ?? '');
    } catch (e) {
      setStatus(Failure(e instanceof Error ? e.message : 'Failed to load setup status'));
    }
  }, []);

  const loadClaudeAuth = useCallback(async () => {
    setClaudeAuth(Loading);
    try {
      const data = await getClaudeAuth();
      setClaudeAuth(Success(data));
    } catch (e) {
      setClaudeAuth(Failure(e instanceof Error ? e.message : 'Failed to check Claude auth'));
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  useEffect(() => {
    loadClaudeAuth();
  }, [loadClaudeAuth]);

  const statusData = status.tag === 'Success' ? status.data : null;

  const handleTelegramSubmit = async () => {
    setTelegramError(null);
    setTelegramSuccess(null);
    setTelegramLoading(true);
    try {
      const res: TelegramSetupResponse = await postTelegramToken(telegramToken);
      if (res.success && res.bot_name) {
        setTelegramSuccess(`Connected as @${res.bot_name}`);
        setTelegramToken('');
        loadStatus();
      } else {
        setTelegramError(res.error ?? 'Failed');
      }
    } catch (e) {
      setTelegramError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setTelegramLoading(false);
    }
  };

  const handleGeminiSubmit = async () => {
    setGeminiError(null);
    setGeminiSuccess(null);
    setGeminiLoading(true);
    try {
      const res: GeminiSetupResponse = await postGeminiKey(geminiKey);
      if (res.success) {
        setGeminiSuccess('Key saved');
        setGeminiKey('');
        loadStatus();
      } else {
        setGeminiError(res.error ?? 'Failed');
      }
    } catch (e) {
      setGeminiError(e instanceof Error ? e.message : 'Failed');
    } finally {
      setGeminiLoading(false);
    }
  };

  const handleInstallClaude = async () => {
    setClaudeInstallError(null);
    setClaudeInstallLoading(true);
    try {
      const res: ClaudeInstallResponse = await postInstallClaude();
      if (res.success) {
        loadClaudeAuth();
        loadStatus();
      } else {
        setClaudeInstallError(res.error ?? 'Install failed');
      }
    } catch (e) {
      setClaudeInstallError(e instanceof Error ? e.message : 'Install failed');
    } finally {
      setClaudeInstallLoading(false);
    }
  };

  const handleUpdateClaude = async () => {
    setClaudeUpdateLoading(true);
    try {
      const res: ClaudeInstallResponse = await postUpdateClaude();
      if (res.success) {
        loadClaudeAuth();
        loadStatus();
      }
    } catch {
      // ignore
    } finally {
      setClaudeUpdateLoading(false);
    }
  };

  const handleTestClaude = async () => {
    setClaudeTestOutput(null);
    setClaudeTestError(null);
    setClaudeTestLoading(true);
    try {
      const res: ClaudeTestResponse = await postTestClaude();
      if (res.success) {
        setClaudeTestOutput(res.output ?? 'OK');
      } else {
        setClaudeTestError(res.error ?? 'Test failed');
      }
    } catch (e) {
      setClaudeTestError(e instanceof Error ? e.message : 'Test failed');
    } finally {
      setClaudeTestLoading(false);
    }
  };

  const handleCheckThreading = async () => {
    setThreadingLoading(true);
    setThreadingResult(Loading);
    try {
      const res = await checkThreading();
      setThreadingResult(Success(res));
    } catch (e) {
      setThreadingResult(Failure(e instanceof Error ? e.message : 'Check failed'));
    } finally {
      setThreadingLoading(false);
    }
  };

  const handleSaveAllowedUsername = async () => {
    setAllowedUsernameError(null);
    setAllowedUsernameLoading(true);
    try {
      const settings = await getSettings();
      await putSettings({
        ...settings,
        allowed_username: allowedUsername.trim() || null,
      });
      loadStatus();
    } catch (e) {
      setAllowedUsernameError(e instanceof Error ? e.message : 'Save failed');
    } finally {
      setAllowedUsernameLoading(false);
    }
  };

  const authData = claudeAuth.tag === 'Success' ? claudeAuth.data : null;
  const threadingData = threadingResult.tag === 'Success' ? threadingResult.data : null;

  if (status.tag === 'Loading' || status.tag === 'NotAsked') {
    return <LoadingSpinner />;
  }

  return (
    <div>
      <PageHeader title="Setup" />
      <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
        {/* Step 1: Telegram */}
        <Card>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '1rem' }}>
            <StepIndicator done={statusData?.has_telegram_token ?? false} />
            <h3 style={{ fontFamily: fonts.display, fontSize: '1rem', margin: 0, color: colors.textPrimary }}>
              Step 1: Telegram Token
            </h3>
          </div>
          {statusData?.bot_name && (
            <div style={{ marginBottom: '0.75rem', color: colors.success, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
              Bot: @{statusData.bot_name}
            </div>
          )}
          <FormField label="Token">
            <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
              <InputField value={telegramToken} onChange={setTelegramToken} placeholder="Bot token..." />
              <Button label="Submit" onClick={handleTelegramSubmit} disabled={telegramLoading} />
            </div>
          </FormField>
          {telegramError && <div style={{ color: colors.error, fontSize: '0.75rem', marginTop: '0.5rem' }}>{telegramError}</div>}
          {telegramSuccess && <div style={{ color: colors.success, fontSize: '0.75rem', marginTop: '0.5rem' }}>{telegramSuccess}</div>}
        </Card>

        {/* Step 2: Gemini */}
        <Card>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '1rem' }}>
            <StepIndicator done={statusData?.has_gemini_key ?? false} />
            <h3 style={{ fontFamily: fonts.display, fontSize: '1rem', margin: 0, color: colors.textPrimary }}>
              Step 2: Gemini API Key
            </h3>
          </div>
          {statusData?.gemini_key_preview && (
            <div style={{ marginBottom: '0.75rem', color: colors.success, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
              Preview: {statusData.gemini_key_preview}
            </div>
          )}
          <FormField label="Key">
            <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
              <InputField value={geminiKey} onChange={setGeminiKey} placeholder="API key..." />
              <Button label="Submit" onClick={handleGeminiSubmit} disabled={geminiLoading} />
              <Button label="Skip" onClick={() => loadStatus()} />
            </div>
          </FormField>
          {geminiError && <div style={{ color: colors.error, fontSize: '0.75rem', marginTop: '0.5rem' }}>{geminiError}</div>}
          {geminiSuccess && <div style={{ color: colors.success, fontSize: '0.75rem', marginTop: '0.5rem' }}>{geminiSuccess}</div>}
        </Card>

        {/* Step 3: Claude CLI */}
        <Card>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '1rem' }}>
            <StepIndicator done={statusData?.has_claude_cli ?? false} />
            <h3 style={{ fontFamily: fonts.display, fontSize: '1rem', margin: 0, color: colors.textPrimary }}>
              Step 3: Claude CLI
            </h3>
          </div>
          {authData && (
            <div style={{ marginBottom: '1rem', fontFamily: fonts.mono, fontSize: '0.75rem', color: colors.textSecondary }}>
              {authData.installed
                ? `Installed: ${authData.version ?? '?'}${authData.authenticated ? ` — ${authData.auth_mode ?? 'auth'}` : ' — Not authenticated'}
                ${authData.needs_update ? ' (update available)' : ''}`
                : 'Not installed'}
            </div>
          )}
          <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
            {!authData?.installed && (
              <Button label="Install" onClick={handleInstallClaude} disabled={claudeInstallLoading} />
            )}
            {authData?.installed && (
              <>
                <Button label="Check Auth" onClick={loadClaudeAuth} disabled={claudeAuth.tag === 'Loading'} />
                {authData.needs_update && (
                  <Button label="Update" onClick={handleUpdateClaude} disabled={claudeUpdateLoading} />
                )}
                <Button label="Test" onClick={handleTestClaude} disabled={claudeTestLoading} />
              </>
            )}
          </div>
          {claudeInstallError && <div style={{ color: colors.error, fontSize: '0.75rem', marginTop: '0.5rem' }}>{claudeInstallError}</div>}
          {claudeTestOutput && <div style={{ color: colors.success, fontSize: '0.75rem', marginTop: '0.5rem', fontFamily: fonts.mono }}>{claudeTestOutput}</div>}
          {claudeTestError && <div style={{ color: colors.error, fontSize: '0.75rem', marginTop: '0.5rem' }}>{claudeTestError}</div>}
        </Card>

        {/* Step 4: Access Control */}
        <Card>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '1rem' }}>
            <StepIndicator done={statusData?.has_allowed_username ?? false} />
            <h3 style={{ fontFamily: fonts.display, fontSize: '1rem', margin: 0, color: colors.textPrimary }}>
              Step 4: Access Control
            </h3>
          </div>
          <FormField label="Allowed Username">
            <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
              <InputField value={allowedUsername} onChange={setAllowedUsername} placeholder="@username" />
              <Button label="Save" onClick={handleSaveAllowedUsername} disabled={allowedUsernameLoading} />
            </div>
          </FormField>
          {allowedUsernameError && <div style={{ color: colors.error, fontSize: '0.75rem', marginTop: '0.5rem' }}>{allowedUsernameError}</div>}
        </Card>

        {/* Step 5: Threading */}
        <Card>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '1rem' }}>
            <StepIndicator done={statusData?.has_threading_enabled ?? false} />
            <h3 style={{ fontFamily: fonts.display, fontSize: '1rem', margin: 0, color: colors.textPrimary }}>
              Step 5: Threading
            </h3>
          </div>
          <Button label="Check Threading" onClick={handleCheckThreading} disabled={threadingLoading} />
          {threadingData && (
            <div style={{ marginTop: '0.75rem', fontFamily: fonts.mono, fontSize: '0.75rem', color: threadingData.enabled ? colors.success : colors.warning }}>
              {threadingData.enabled ? 'Enabled' : 'Disabled'}
              {threadingData.error && ` — ${threadingData.error}`}
            </div>
          )}
        </Card>

        {/* Completion */}
        {statusData?.is_complete && (
          <Card>
            <div style={{ textAlign: 'center', padding: '1rem' }}>
              <div style={{ fontFamily: fonts.display, fontSize: '1.25rem', color: colors.success, marginBottom: '1rem' }}>
                Setup Complete
              </div>
              <PrimaryButton label="Go to Dashboard" onClick={() => navigate('/')} />
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}

function StepIndicator({ done }: { done: boolean }): React.ReactElement {
  return (
    <div
      style={{
        width: '24px',
        height: '24px',
        borderRadius: '50%',
        backgroundColor: done ? colors.success : colors.bgSurface,
        border: `2px solid ${done ? colors.success : colors.border}`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontFamily: fonts.mono,
        fontSize: '0.75rem',
        color: done ? colors.bgPrimary : colors.textMuted,
      }}
    >
      {done ? '✓' : ''}
    </div>
  );
}
