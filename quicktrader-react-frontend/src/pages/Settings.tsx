import React, { useEffect, useState, useCallback } from 'react';
import {
  PageHeader,
  Card,
  CardWithHeader,
  SectionHeader,
  FormField,
  InputField,
  TextareaField,
  SelectField,
  ToggleSwitch,
  Button,
  LoadingSpinner,
} from '../components/UI';
import { getSettings, getApiKeys, putSettings, putApiKeys } from '../api';
import type { Settings, ApiKeysData } from '../types';
import type { RemoteData } from '../types';
import { NotAsked, Loading, Success, Failure, isLoading } from '../types';
import { defaultSettings } from '../types';
import { colors, fonts } from '../theme';

export function Settings(): React.ReactElement {
  const [settings, setSettings] = useState<RemoteData<Settings>>(NotAsked);
  const [apiKeys, setApiKeys] = useState<RemoteData<ApiKeysData>>(NotAsked);

  const [localSettings, setLocalSettings] = useState<Settings>(defaultSettings);
  const [telegramToken, setTelegramToken] = useState('');
  const [geminiKey, setGeminiKey] = useState('');

  const [savingField, setSavingField] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);

  const loadSettings = useCallback(async () => {
    setSettings(Loading);
    try {
      const data = await getSettings();
      setSettings(Success(data));
      setLocalSettings(data);
    } catch (e) {
      setSettings(Failure(e instanceof Error ? e.message : 'Failed to load settings'));
    }
  }, []);

  const loadApiKeys = useCallback(async () => {
    setApiKeys(Loading);
    try {
      const data = await getApiKeys();
      setApiKeys(Success(data));
    } catch (e) {
      setApiKeys(Failure(e instanceof Error ? e.message : 'Failed to load API keys'));
    }
  }, []);

  useEffect(() => {
    loadSettings();
    loadApiKeys();
  }, [loadSettings, loadApiKeys]);

  const saveSettings = useCallback(
    async (updated: Settings) => {
      setSavingField('settings');
      setSaveError(null);
      try {
        const saved = await putSettings(updated);
        setSettings(Success(saved));
        setLocalSettings(saved);
      } catch (e) {
        setSaveError(e instanceof Error ? e.message : 'Failed to save');
      } finally {
        setSavingField(null);
      }
    },
    []
  );

  const saveApiKeys = useCallback(
    async (telegram?: string, gemini?: string) => {
      setSavingField('api-keys');
      setSaveError(null);
      try {
        await putApiKeys(telegram, gemini);
        setTelegramToken('');
        setGeminiKey('');
        loadApiKeys();
      } catch (e) {
        setSaveError(e instanceof Error ? e.message : 'Failed to save');
      } finally {
        setSavingField(null);
      }
    },
    [loadApiKeys]
  );

  const settingsData = settings.tag === 'Success' ? settings.data : null;
  const apiKeysData = apiKeys.tag === 'Success' ? apiKeys.data : null;

  if (settings.tag === 'Loading' || settings.tag === 'NotAsked') {
    return <LoadingSpinner />;
  }

  return (
    <div>
      <PageHeader title="Settings" />
      {saveError && (
        <div
          style={{
            marginBottom: '1rem',
            padding: '0.75rem',
            backgroundColor: colors.errorDim,
            border: `1px solid ${colors.error}`,
            borderRadius: '4px',
            color: colors.error,
            fontFamily: fonts.mono,
            fontSize: '0.75rem',
          }}
        >
          {saveError}
        </div>
      )}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
        <CardWithHeader title="Display Preferences">
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <ToggleSwitch
              checked={localSettings.show_tool_messages}
              onChange={(v) => {
                const next = { ...localSettings, show_tool_messages: v };
                setLocalSettings(next);
                saveSettings(next);
              }}
              label="Show tool messages"
            />
            <ToggleSwitch
              checked={localSettings.show_thinking_messages}
              onChange={(v) => {
                const next = { ...localSettings, show_thinking_messages: v };
                setLocalSettings(next);
                saveSettings(next);
              }}
              label="Show thinking messages"
            />
            <ToggleSwitch
              checked={localSettings.show_tool_results}
              onChange={(v) => {
                const next = { ...localSettings, show_tool_results: v };
                setLocalSettings(next);
                saveSettings(next);
              }}
              label="Show tool results"
            />
          </div>
        </CardWithHeader>

        <CardWithHeader title="Chat Configuration">
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <FormField label="Chat Harness">
              <SelectField
                value={localSettings.chat_harness}
                onChange={(v) => setLocalSettings((s) => ({ ...s, chat_harness: v }))}
                options={[
                  ['claude', 'Claude'],
                  ['codex', 'Codex'],
                ]}
              />
            </FormField>
            <FormField label="Claude Model">
              <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <InputField
                  value={localSettings.claude_model}
                  onChange={(v) => setLocalSettings((s) => ({ ...s, claude_model: v }))}
                  placeholder="claude-opus-4-6"
                />
                <Button
                  label="Save"
                  onClick={() => saveSettings(localSettings)}
                  disabled={savingField !== null}
                />
              </div>
            </FormField>
          </div>
        </CardWithHeader>

        <CardWithHeader title="Role Prompts">
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <FormField label="Dev Role Prompt">
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                <TextareaField
                  value={localSettings.dev_role_prompt}
                  onChange={(v) => setLocalSettings((s) => ({ ...s, dev_role_prompt: v }))}
                  placeholder="Dev role prompt..."
                />
                <Button
                  label="Save"
                  onClick={() => saveSettings(localSettings)}
                  disabled={savingField !== null}
                />
              </div>
            </FormField>
            <FormField label="Harden Role Prompt">
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                <TextareaField
                  value={localSettings.harden_role_prompt}
                  onChange={(v) => setLocalSettings((s) => ({ ...s, harden_role_prompt: v }))}
                  placeholder="Harden role prompt..."
                />
                <Button
                  label="Save"
                  onClick={() => saveSettings(localSettings)}
                  disabled={savingField !== null}
                />
              </div>
            </FormField>
            <FormField label="PM Role Prompt">
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                <TextareaField
                  value={localSettings.pm_role_prompt}
                  onChange={(v) => setLocalSettings((s) => ({ ...s, pm_role_prompt: v }))}
                  placeholder="PM role prompt..."
                />
                <Button
                  label="Save"
                  onClick={() => saveSettings(localSettings)}
                  disabled={savingField !== null}
                />
              </div>
            </FormField>
          </div>
        </CardWithHeader>

        <CardWithHeader title="API Keys">
          {apiKeysData ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
              <FormField label="Telegram">
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                  {apiKeysData.telegram_token_masked && (
                    <span
                      style={{
                        fontFamily: fonts.mono,
                        fontSize: '0.75rem',
                        color: colors.textMuted,
                      }}
                    >
                      Current: {apiKeysData.telegram_token_masked}
                    </span>
                  )}
                  <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                    <InputField
                      value={telegramToken}
                      onChange={setTelegramToken}
                      placeholder="New token..."
                    />
                    <Button
                      label="Save"
                      onClick={() => saveApiKeys(telegramToken || undefined)}
                      disabled={savingField !== null}
                    />
                  </div>
                </div>
              </FormField>
              <FormField label="Gemini">
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                  {apiKeysData.gemini_key_masked && (
                    <span
                      style={{
                        fontFamily: fonts.mono,
                        fontSize: '0.75rem',
                        color: colors.textMuted,
                      }}
                    >
                      Current: {apiKeysData.gemini_key_masked}
                    </span>
                  )}
                  <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                    <InputField
                      value={geminiKey}
                      onChange={setGeminiKey}
                      placeholder="New key..."
                    />
                    <Button
                      label="Save"
                      onClick={() => saveApiKeys(undefined, geminiKey || undefined)}
                      disabled={savingField !== null}
                    />
                  </div>
                </div>
              </FormField>
              {apiKeysData.claude_code_status && (
                <FormField label="Claude Code">
                  <div
                    style={{
                      fontFamily: fonts.mono,
                      fontSize: '0.75rem',
                      color: colors.textSecondary,
                    }}
                  >
                    {apiKeysData.claude_code_status.auth_mode} —{' '}
                    {apiKeysData.claude_code_status.account_email ??
                      apiKeysData.claude_code_status.account_name ??
                      'configured'}
                  </div>
                </FormField>
              )}
            </div>
          ) : apiKeys.tag === 'Failure' ? (
            <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
              {apiKeys.error}
            </div>
          ) : null}
        </CardWithHeader>

        <CardWithHeader title="Access Control">
          <FormField label="Allowed Username">
            <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
              <InputField
                value={localSettings.allowed_username ?? ''}
                onChange={(v) =>
                  setLocalSettings((s) => ({
                    ...s,
                    allowed_username: v.trim() || null,
                  }))
                }
                placeholder="Username (e.g. @user)"
              />
              <Button
                label="Save"
                onClick={() => saveSettings(localSettings)}
                disabled={savingField !== null}
              />
              <Button
                label="Clear"
                onClick={() => {
                  const next = { ...localSettings, allowed_username: null };
                  setLocalSettings(next);
                  saveSettings(next);
                }}
                disabled={savingField !== null}
              />
            </div>
          </FormField>
        </CardWithHeader>
      </div>
    </div>
  );
}
