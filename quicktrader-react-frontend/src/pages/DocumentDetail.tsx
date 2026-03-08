import React, { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  PageHeader,
  BackButton,
  Card,
  SectionHeader,
  PrimaryButton,
  DocTypeBadge,
  AccentedItem,
  TextareaField,
  LoadingSpinner,
  Timestamp,
} from '../components/UI';
import { MarkdownView } from '../components/Markdown';
import { getDocument, listCommentsForDocument, upsertComment } from '../api';
import type { WorkDocument, WorkComment, RemoteData } from '../types';
import { NotAsked, Loading, Success, Failure } from '../types';
import { colors, fonts } from '../theme';

type CommentNode = { comment: WorkComment; children: CommentNode[] };

function buildCommentTree(items: WorkComment[]): CommentNode[] {
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
}

export function DocumentDetail(): React.ReactElement {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const documentId = id ? parseInt(id, 10) : NaN;

  const [document, setDocument] = useState<RemoteData<WorkDocument>>(NotAsked);
  const [comments, setComments] = useState<RemoteData<WorkComment[]>>(NotAsked);
  const [commentForm, setCommentForm] = useState('');

  const loadDocument = useCallback(async () => {
    if (isNaN(documentId)) return;
    setDocument(Loading);
    try {
      const data = await getDocument(documentId);
      setDocument(Success(data));
    } catch (e) {
      setDocument(Failure(e instanceof Error ? e.message : 'Failed to load document'));
    }
  }, [documentId]);

  const loadComments = useCallback(async () => {
    if (isNaN(documentId)) return;
    setComments(Loading);
    try {
      const data = await listCommentsForDocument(documentId);
      setComments(Success(data));
    } catch (e) {
      setComments(Failure(e instanceof Error ? e.message : 'Failed to load comments'));
    }
  }, [documentId]);

  useEffect(() => {
    loadDocument();
    loadComments();
  }, [loadDocument, loadComments]);

  const handleAddComment = useCallback(async () => {
    if (isNaN(documentId) || !commentForm.trim()) return;
    try {
      await upsertComment({ document_id: documentId, content: commentForm.trim() });
      setCommentForm('');
      loadComments();
    } catch (e) {
      console.error(e);
    }
  }, [documentId, commentForm, loadComments]);

  const documentData = document.tag === 'Success' ? document.data : null;
  const commentsData = comments.tag === 'Success' ? comments.data : null;
  const projectId = documentData?.project_id;

  if (isNaN(documentId)) {
    return (
      <div>
        <BackButton onClick={() => navigate(projectId ? `/projects/${projectId}` : '/projects')} />
        <div style={{ color: colors.error, marginTop: '1rem' }}>Invalid document ID</div>
      </div>
    );
  }

  if (document.tag === 'Failure') {
    return (
      <div>
        <BackButton onClick={() => navigate(projectId ? `/projects/${projectId}` : '/projects')} />
        <div style={{ color: colors.error, marginTop: '1rem' }}>{document.error}</div>
      </div>
    );
  }

  if (!documentData) {
    return (
      <div>
        <BackButton onClick={() => navigate('/projects')} />
        <LoadingSpinner />
      </div>
    );
  }

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
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
          <h2
            style={{
              fontFamily: fonts.display,
              fontSize: 'clamp(1.25rem, 5vw, 1.75rem)',
              fontWeight: 600,
              letterSpacing: '0.02em',
              textTransform: 'uppercase',
              margin: 0,
              color: colors.textPrimary,
            }}
          >
            {documentData.title}
          </h2>
          <DocTypeBadge docType={documentData.document_type} />
        </div>
      </div>

      <Card>
        <MarkdownView content={documentData.content || '_No content_'} />
      </Card>

      <SectionHeader title="Comments" />
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1rem' }}>
        {comments.tag === 'Loading' ? (
          <LoadingSpinner />
        ) : (
          renderTree(commentTree, 0)
        )}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
        <TextareaField value={commentForm} onChange={setCommentForm} placeholder="Add a comment..." />
        <PrimaryButton label="Add Comment" onClick={handleAddComment} disabled={!commentForm.trim()} />
      </div>
    </div>
  );
}
