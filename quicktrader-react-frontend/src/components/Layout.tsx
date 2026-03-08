import React from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { colors, fonts } from '../theme';
import { Container } from './UI';

interface LayoutProps {
  backendOnline: boolean;
  children: React.ReactNode;
}

const navItems: { label: string; path: string; matchPrefixes: string[] }[] = [
  { label: 'Dashboard', path: '/', matchPrefixes: ['/', '/dashboard'] },
  { label: 'Chat', path: '/chat', matchPrefixes: ['/chat'] },
  { label: 'Messages', path: '/messages', matchPrefixes: ['/messages'] },
  { label: 'Logs', path: '/logs', matchPrefixes: ['/logs'] },
  { label: 'Jobs', path: '/jobs', matchPrefixes: ['/jobs'] },
  { label: '|', path: '', matchPrefixes: [] },
  { label: 'Projects', path: '/projects', matchPrefixes: ['/projects'] },
  { label: 'Live', path: '/live-board', matchPrefixes: ['/live-board'] },
  { label: '|', path: '', matchPrefixes: [] },
  { label: 'Settings', path: '/settings', matchPrefixes: ['/settings'] },
];

function isActive(item: typeof navItems[number], pathname: string): boolean {
  if (item.path === '/' || item.path === '/dashboard') {
    return pathname === '/' || pathname === '/dashboard';
  }
  return item.matchPrefixes.some(p => pathname === p || pathname.startsWith(p + '/'));
}

function StatusIndicator({ online }: { online: boolean }) {
  const indicatorColor = online ? colors.success : colors.textMuted;
  const label = online ? 'ONLINE' : 'OFFLINE';
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', padding: '0.375rem 0.75rem', backgroundColor: colors.bgSurface, borderRadius: '2px' }}>
      <div style={{
        width: '6px', height: '6px', backgroundColor: indicatorColor, borderRadius: '50%',
        boxShadow: `0 0 8px ${indicatorColor}`,
        animation: online ? 'statusPulse 2s infinite' : 'none',
      }} />
      <span style={{ fontFamily: fonts.mono, fontSize: '0.5625rem', fontWeight: 600, letterSpacing: '0.1em', color: indicatorColor }}>
        {label}
      </span>
    </div>
  );
}

function NavDivider() {
  return (
    <div style={{ width: '1px', backgroundColor: colors.border, margin: '0.5rem 0.25rem', alignSelf: 'stretch' }} />
  );
}

function NavLink({ label, path, active, onClick }: { label: string; path: string; active: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      style={{
        background: 'transparent',
        color: active ? colors.accent : colors.textSecondary,
        border: 'none',
        borderBottom: active ? `2px solid ${colors.accent}` : '2px solid transparent',
        padding: '0 clamp(0.75rem, 4vw, 1.5rem)',
        cursor: 'pointer',
        fontFamily: fonts.body,
        fontSize: 'clamp(0.6875rem, 2.8vw, 0.8125rem)',
        fontWeight: 500,
        letterSpacing: '0.02em',
        textTransform: 'uppercase' as const,
        transition: 'none',
        position: 'relative' as const,
        whiteSpace: 'nowrap' as const,
      }}
    >
      {label}
      {active && (
        <div style={{
          position: 'absolute', bottom: '-1px', left: 0, right: 0, height: '1px',
          background: colors.accent, boxShadow: `0 0 12px ${colors.accent}`,
        }} />
      )}
    </button>
  );
}

function MobileTab({ label, active, onClick }: { label: string; active: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      style={{
        backgroundColor: active ? colors.accentDim : colors.bgSurface,
        color: active ? colors.accent : colors.textSecondary,
        border: `1px solid ${active ? 'rgba(0, 212, 170, 0.45)' : colors.border}`,
        borderRadius: '999px',
        padding: '0.5rem 0.875rem',
        fontFamily: fonts.mono,
        fontSize: '0.6875rem',
        fontWeight: 700,
        letterSpacing: '0.08em',
        textTransform: 'uppercase' as const,
        whiteSpace: 'nowrap' as const,
        boxShadow: active ? '0 0 18px rgba(0, 212, 170, 0.18)' : 'none',
      }}
    >
      {label}
    </button>
  );
}

export function Layout({ backendOnline, children }: LayoutProps) {
  const navigate = useNavigate();
  const { pathname } = useLocation();

  return (
    <div style={{
      minHeight: '100vh',
      backgroundColor: colors.bgPrimary,
      backgroundImage: `linear-gradient(to bottom, ${colors.bgPrimary}, #060810), repeating-linear-gradient(0deg, transparent, transparent 100px, ${colors.gridLine} 100px, ${colors.gridLine} 101px)`,
      color: colors.textPrimary,
      fontFamily: fonts.body,
      lineHeight: 1.6,
      fontSize: '14px',
      WebkitFontSmoothing: 'antialiased',
    }}>
      <header style={{
        backgroundColor: colors.bgSecondary,
        borderBottom: `1px solid ${colors.border}`,
        padding: 0,
        position: 'sticky',
        top: 0,
        zIndex: 100,
        backdropFilter: 'blur(12px)',
      }}>
        <Container>
          <div style={{
            display: 'flex', justifyContent: 'space-between', alignItems: 'stretch',
            flexWrap: 'wrap', gap: '0.75rem', minHeight: '64px',
          }}>
            {/* Brand */}
            <div className="tb-brand-shell" style={{
              display: 'flex', alignItems: 'center', gap: 'clamp(0.75rem, 2vw, 1.25rem)',
              paddingRight: 'clamp(0.75rem, 3vw, 2rem)', borderRight: `1px solid ${colors.border}`,
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                <div style={{
                  width: '28px', height: '28px',
                  background: `linear-gradient(135deg, ${colors.accent} 0%, #00a884 100%)`,
                  clipPath: 'polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)',
                }} />
                <h1 style={{
                  fontFamily: fonts.display, fontSize: 'clamp(1.0rem, 4vw, 1.25rem)',
                  fontWeight: 600, letterSpacing: '0.05em', textTransform: 'uppercase',
                  color: colors.textPrimary, margin: 0,
                }}>TWOLEBOT</h1>
              </div>
              <StatusIndicator online={backendOnline} />
            </div>

            {/* Desktop nav */}
            <nav className="tb-desktop-nav" style={{
              display: 'flex', alignItems: 'stretch', flexWrap: 'wrap',
              justifyContent: 'flex-end', columnGap: '0.25rem', rowGap: '0.25rem', marginLeft: 'auto',
            }}>
              {navItems.map((item, i) =>
                item.label === '|'
                  ? <NavDivider key={`div-${i}`} />
                  : <NavLink
                      key={item.path}
                      label={item.label}
                      path={item.path}
                      active={isActive(item, pathname)}
                      onClick={() => navigate(item.path)}
                    />
              )}
            </nav>
          </div>
        </Container>

        {/* Mobile tabs */}
        <div className="tb-mobile-tabs" style={{ borderTop: `1px solid ${colors.border}`, backgroundColor: colors.bgSecondary }}>
          <Container>
            <div style={{ position: 'relative', padding: '0.5rem 0' }}>
              <div className="tb-mobile-tabs__inner" style={{
                display: 'flex', gap: '0.5rem', overflowX: 'auto', overflowY: 'hidden',
                WebkitOverflowScrolling: 'touch', scrollbarWidth: 'none' as never,
                padding: '0.25rem 0.75rem', margin: '0 -0.75rem',
              }}>
                {navItems.filter(n => n.label !== '|').map(item => (
                  <MobileTab
                    key={item.path}
                    label={item.label}
                    active={isActive(item, pathname)}
                    onClick={() => navigate(item.path)}
                  />
                ))}
              </div>
              <div style={{ pointerEvents: 'none', position: 'absolute', left: 0, top: 0, bottom: 0, width: '24px', background: `linear-gradient(90deg, ${colors.bgSecondary}, transparent)` }} />
              <div style={{ pointerEvents: 'none', position: 'absolute', right: 0, top: 0, bottom: 0, width: '24px', background: `linear-gradient(-90deg, ${colors.bgSecondary}, transparent)` }} />
            </div>
          </Container>
        </div>
      </header>

      <Container>{children}</Container>
    </div>
  );
}
