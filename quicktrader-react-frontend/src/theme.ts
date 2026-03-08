export const colors = {
  bgPrimary: '#0a0e14',
  bgSecondary: '#11151c',
  bgTertiary: '#1a1f2b',
  bgSurface: '#0d1117',
  textPrimary: '#e6edf3',
  textSecondary: '#9ca3af',
  textMuted: '#5c6370',
  accent: '#00d4aa',
  accentGlow: 'rgba(0, 212, 170, 0.25)',
  accentDim: 'rgba(0, 212, 170, 0.08)',
  success: '#4ade80',
  successDim: 'rgba(74, 222, 128, 0.12)',
  warning: '#fbbf24',
  warningDim: 'rgba(251, 191, 36, 0.12)',
  error: '#f87171',
  errorDim: 'rgba(248, 113, 113, 0.12)',
  border: '#21262d',
  borderLight: 'rgba(255, 255, 255, 0.06)',
  gridLine: 'rgba(0, 212, 170, 0.06)',
} as const;

export const fonts = {
  body: "'IBM Plex Sans', 'SF Pro Display', -apple-system, system-ui, sans-serif",
  mono: "'JetBrains Mono', 'IBM Plex Mono', 'SF Mono', Consolas, monospace",
  display: "'IBM Plex Sans Condensed', 'Impact', 'Arial Black', sans-serif",
} as const;

export function taskStatusColor(status: string): string {
  switch (status) {
    case 'todo': return colors.textMuted;
    case 'in_progress': return colors.warning;
    case 'ready_for_review': return colors.accent;
    case 'under_review': return '#a78bfa';
    case 'done': return colors.success;
    case 'blocked': return colors.error;
    case 'abandoned': return colors.textMuted;
    default: return colors.border;
  }
}

export function taskStatusLabel(status: string): string {
  switch (status) {
    case 'todo': return 'Todo';
    case 'in_progress': return 'In Progress';
    case 'ready_for_review': return 'Ready for Review';
    case 'under_review': return 'Under Review';
    case 'done': return 'Done';
    case 'blocked': return 'Blocked';
    case 'abandoned': return 'Abandoned';
    default: return status;
  }
}

export function taskPriorityLabel(priority: string): string {
  switch (priority) {
    case 'low': return 'Low';
    case 'medium': return 'Medium';
    case 'high': return 'High';
    case 'critical': return 'Critical';
    default: return priority;
  }
}

export function formatTime(ts: string): string {
  return ts.slice(11, 19);
}

export function formatDateTime(ts: string): string {
  const date = ts.slice(8, 10);
  const month = ts.slice(5, 7);
  const time = ts.slice(11, 16);
  const monthNames: Record<string, string> = {
    '01': 'Jan', '02': 'Feb', '03': 'Mar', '04': 'Apr',
    '05': 'May', '06': 'Jun', '07': 'Jul', '08': 'Aug',
    '09': 'Sep', '10': 'Oct', '11': 'Nov', '12': 'Dec',
  };
  return `${date} ${monthNames[month] ?? month} ${time}`;
}

export function truncateText(maxLen: number, str: string): string {
  return str.length > maxLen ? str.slice(0, maxLen) + '...' : str;
}

export const globalStyles = `
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    background-color: ${colors.bgPrimary};
    color: ${colors.textPrimary};
    font-family: ${fonts.body};
    line-height: 1.6;
    font-size: 14px;
    -webkit-font-smoothing: antialiased;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  @keyframes statusPulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  button { cursor: pointer; }
  button:focus-visible { outline: 1px solid ${colors.accent}; outline-offset: 2px; }
  .tb-mobile-tabs { display: none; }
  .tb-mobile-tabs__inner::-webkit-scrollbar { display: none; }
  @media (max-width: 768px) {
    .tb-desktop-nav { display: none !important; }
    .tb-mobile-tabs { display: block !important; }
    .tb-brand-shell { border-right: none !important; }
  }
`;
