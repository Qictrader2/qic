import React from 'react';
import { colors, fonts } from '../theme';

type PageItem = { type: 'page'; page: number } | { type: 'gap' };

function clampPage(totalPages: number, page: number): number {
  const last = Math.max(0, totalPages - 1);
  if (page < 0) return 0;
  if (page > last) return last;
  return page;
}

function pageItems(totalPages: number, currentPage: number, radius: number): PageItem[] {
  const total = Math.max(1, totalPages);
  const current = clampPage(total, currentPage);
  const maxWithoutGaps = 2 * radius + 5;
  const start = Math.max(1, current - radius);
  const finish = Math.min(total - 2, current + radius);

  const leftExtras: PageItem[] =
    start <= 2
      ? Array.from({ length: start - 1 }, (_, i) => ({ type: 'page' as const, page: i + 1 }))
      : [{ type: 'gap' }];

  const rightExtras: PageItem[] =
    finish >= total - 3
      ? Array.from({ length: total - 2 - finish }, (_, i) => ({
          type: 'page' as const,
          page: finish + i + 1,
        }))
      : [{ type: 'gap' }];

  const middle: PageItem[] = Array.from({ length: finish - start + 1 }, (_, i) => ({
    type: 'page' as const,
    page: start + i,
  }));

  if (total <= maxWithoutGaps) {
    return Array.from({ length: total }, (_, i) => ({ type: 'page' as const, page: i }));
  }
  return [
    { type: 'page', page: 0 },
    ...leftExtras,
    ...middle,
    ...rightExtras,
    { type: 'page', page: total - 1 },
  ];
}

export function Pagination({
  page,
  totalPages,
  onPageChange,
}: {
  page: number;
  totalPages: number;
  onPageChange: (p: number) => void;
}) {
  if (totalPages <= 1) {
    return null;
  }

  const items = pageItems(totalPages, page, 2);

  const navButton = (
    label: string,
    glyph: string,
    targetPage: number,
    isDisabled: boolean
  ) => (
    <button
      type="button"
      title={label}
      disabled={isDisabled}
      aria-label={label}
      onClick={() => !isDisabled && onPageChange(clampPage(totalPages, targetPage))}
      style={{
        padding: '0.35rem 0.55rem',
        backgroundColor: isDisabled ? colors.bgSurface : colors.bgTertiary,
        border: `1px solid ${isDisabled ? colors.border : 'rgba(0, 212, 170, 0.18)'}`,
        borderRadius: '6px',
        color: isDisabled ? colors.textMuted : colors.textSecondary,
        fontFamily: fonts.mono,
        fontWeight: 700,
        fontSize: '0.8125rem',
        lineHeight: 1,
        letterSpacing: '0.04em',
        minWidth: '1.75rem',
        cursor: isDisabled ? 'not-allowed' : 'pointer',
        opacity: isDisabled ? 0.55 : 1,
      }}
    >
      {glyph}
    </button>
  );

  const pageButton = (pageIndex: number) => {
    const isActive = pageIndex === page;
    return (
      <button
        type="button"
        key={pageIndex}
        title={`Page ${pageIndex + 1}`}
        onClick={() => onPageChange(pageIndex)}
        aria-current={isActive ? 'page' : undefined}
        style={{
          padding: '0.35rem 0.55rem',
          background: isActive
            ? `linear-gradient(135deg, ${colors.accent} 0%, #00a884 100%)`
            : colors.bgSurface,
          border: `1px solid ${isActive ? 'rgba(0, 212, 170, 0.8)' : colors.border}`,
          borderRadius: '6px',
          color: isActive ? colors.bgPrimary : colors.textSecondary,
          fontFamily: fonts.mono,
          fontWeight: isActive ? 800 : 600,
          fontSize: '0.75rem',
          letterSpacing: '0.04em',
          minWidth: '1.75rem',
          textAlign: 'center',
          boxShadow: isActive
            ? `0 0 0 1px rgba(0, 212, 170, 0.25), 0 0 18px ${colors.accentGlow}`
            : 'none',
          cursor: 'pointer',
        }}
      >
        {pageIndex + 1}
      </button>
    );
  };

  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        gap: '0.375rem',
        marginTop: '2rem',
        padding: '1rem',
      }}
    >
      <div
        style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: '0.25rem',
          padding: '0.25rem',
          maxWidth: '100%',
          overflow: 'visible',
          background: `linear-gradient(180deg, ${colors.bgTertiary} 0%, ${colors.bgSecondary} 100%)`,
          border: `1px solid ${colors.border}`,
          borderRadius: '8px',
          boxShadow: '0 0 0 1px rgba(0, 212, 170, 0.06), 0 10px 30px rgba(0,0,0,0.35)',
        }}
      >
        {navButton('FIRST', '«', 0, page === 0)}
        {navButton('PREV', '‹', page - 1, page === 0)}
        {items.map((item, i) =>
          item.type === 'gap' ? (
            <span
              key={`gap-${i}`}
              style={{
                padding: '0 0.375rem',
                color: colors.textMuted,
                fontFamily: fonts.mono,
                fontSize: '0.75rem',
                letterSpacing: '0.06em',
              }}
            >
              …
            </span>
          ) : (
            <React.Fragment key={item.page}>{pageButton(item.page)}</React.Fragment>
          )
        )}
        {navButton('NEXT', '›', page + 1, page >= totalPages - 1)}
        {navButton('LAST', '»', totalPages - 1, page >= totalPages - 1)}
      </div>
    </div>
  );
}
