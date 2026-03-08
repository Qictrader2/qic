import React, { useEffect, useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  PageHeader,
  Card,
  PrimaryButton,
  Button,
  FormField,
  InputField,
  TagChip,
  LoadingSpinner,
  Timestamp,
} from '../components/UI';
import { listProjects, createProject } from '../api';
import type { WorkProject, RemoteData } from '../types';
import type { ProjectForm } from '../types';
import { NotAsked, Loading, Success, Failure, isLoading } from '../types';
import { emptyProjectForm } from '../types';
import { colors, fonts, formatDateTime, truncateText } from '../theme';

export function Projects(): React.ReactElement {
  const navigate = useNavigate();
  const [projects, setProjects] = useState<RemoteData<WorkProject[]>>(NotAsked);
  const [showForm, setShowForm] = useState(false);
  const [projectForm, setProjectForm] = useState<ProjectForm>(emptyProjectForm);

  const loadProjects = useCallback(async () => {
    setProjects(Loading);
    try {
      const data = await listProjects();
      setProjects(Success(data));
    } catch (e) {
      setProjects(Failure(e instanceof Error ? e.message : 'Failed to load projects'));
    }
  }, []);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  const handleSubmit = useCallback(async () => {
    const tags = projectForm.tags
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);
    try {
      await createProject(
        projectForm.name,
        projectForm.description,
        tags,
        projectForm.gitRemoteUrl || undefined
      );
      setShowForm(false);
      setProjectForm(emptyProjectForm);
      loadProjects();
    } catch (e) {
      console.error(e);
    }
  }, [projectForm, loadProjects]);

  const handleCancel = useCallback(() => {
    setShowForm(false);
    setProjectForm(emptyProjectForm);
  }, []);

  const projectsData = projects.tag === 'Success' ? projects.data : null;

  return (
    <div>
      <PageHeader
        title="Projects"
        actions={
          <PrimaryButton
            label="New Project"
            onClick={() => setShowForm((s) => !s)}
          />
        }
      />

      {showForm && (
        <Card style={{ marginBottom: '1.5rem' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <FormField label="Name">
              <InputField
                value={projectForm.name}
                onChange={(v) => setProjectForm((f) => ({ ...f, name: v }))}
                placeholder="Project name"
              />
            </FormField>
            <FormField label="Description">
              <InputField
                value={projectForm.description}
                onChange={(v) => setProjectForm((f) => ({ ...f, description: v }))}
                placeholder="Description"
              />
            </FormField>
            <FormField label="Tags (comma-separated)">
              <InputField
                value={projectForm.tags}
                onChange={(v) => setProjectForm((f) => ({ ...f, tags: v }))}
                placeholder="tag1, tag2, tag3"
              />
            </FormField>
            <FormField label="Git Remote URL">
              <InputField
                value={projectForm.gitRemoteUrl}
                onChange={(v) => setProjectForm((f) => ({ ...f, gitRemoteUrl: v }))}
                placeholder="https://github.com/..."
              />
            </FormField>
            <div style={{ display: 'flex', gap: '0.75rem' }}>
              <PrimaryButton label="Submit" onClick={handleSubmit} />
              <Button label="Cancel" onClick={handleCancel} />
            </div>
          </div>
        </Card>
      )}

      {projects.tag === 'Loading' || projects.tag === 'NotAsked' ? (
        <LoadingSpinner />
      ) : projects.tag === 'Failure' ? (
        <div style={{ color: colors.error, fontFamily: fonts.mono, fontSize: '0.75rem' }}>
          {projects.error}
        </div>
      ) : projectsData ? (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
          {projectsData.map((p) => (
            <Card
              key={p.id}
              style={{ cursor: 'pointer' }}
              onClick={() => navigate(`/projects/${p.id}`)}
            >
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '0.5rem' }}>
                  <span
                    style={{
                      fontFamily: fonts.display,
                      fontSize: '1.125rem',
                      fontWeight: 600,
                      color: colors.textPrimary,
                    }}
                  >
                    {p.name}
                  </span>
                  <span
                    style={{
                      fontFamily: fonts.mono,
                      fontSize: '0.625rem',
                      color: colors.textMuted,
                      backgroundColor: colors.bgSurface,
                      padding: '0.25rem 0.5rem',
                      borderRadius: '2px',
                    }}
                  >
                    {p.task_count} tasks
                  </span>
                </div>
                {p.description && (
                  <div style={{ fontSize: '0.875rem', color: colors.textSecondary }}>
                    {truncateText(120, p.description)}
                  </div>
                )}
                {p.tags.length > 0 && (
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.375rem' }}>
                    {p.tags.map((t) => (
                      <TagChip key={t} tag={t} />
                    ))}
                  </div>
                )}
                {p.git_remote_url && (
                  <div
                    style={{
                      fontFamily: fonts.mono,
                      fontSize: '0.6875rem',
                      color: colors.textMuted,
                      wordBreak: 'break-all',
                    }}
                  >
                    {p.git_remote_url}
                  </div>
                )}
                <div style={{ display: 'flex', gap: '1rem', fontSize: '0.6875rem', color: colors.textMuted }}>
                  <span>Created: <Timestamp ts={p.created_at} /></span>
                  <span>Updated: <Timestamp ts={p.updated_at} /></span>
                </div>
              </div>
            </Card>
          ))}
          {projectsData.length === 0 && (
            <div style={{ textAlign: 'center', padding: '2rem', color: colors.textMuted }}>
              No projects yet. Create one above.
            </div>
          )}
        </div>
      ) : null}
    </div>
  );
}
