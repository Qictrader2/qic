import React from 'react';
import { colors, fonts, formatDateTime } from '../theme';

// ─── Layout Shell ───────────────────────────────────────────────────────────

export function AppShell({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        minHeight: '100vh',
        backgroundColor: colors.bgPrimary,
        backgroundImage: `linear-gradient(to bottom, ${colors.bgPrimary}, #060810), repeating-linear-gradient(0deg, transparent, transparent 100px, ${colors.gridLine} 100px, ${colors.gridLine} 101px)`,
        color: colors.textPrimary,
        fontFamily: fonts.body,
        lineHeight: 1.6,
        fontSize: '14px',
        WebkitFontSmoothing: 'antialiased',
      }}
    >
      {children}
    </div>
  );
}

export function Container({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        maxWidth: '1400px',
        margin: '0 auto',
        padding: '0 clamp(1rem, 4vw, 2rem)',
      }}
    >
      {children}
    </div>
  );
}

// ─── Page Layout ─────────────────────────────────────────────────────────────

export function PageHeader({
  title,
  actions,
}: {
  title: string;
  actions?: React.ReactNode;
}) {
  return (
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
      <div
        style={{
          display: 'flex',
          alignItems: 'baseline',
          gap: '1rem',
        }}
      >
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
          {title}
        </h2>
        <div
          style={{
            width: '32px',
            height: '2px',
            background: `linear-gradient(90deg, ${colors.accent}, transparent)`,
          }}
        />
      </div>
      {actions && (
        <div
          style={{
            display: 'flex',
            gap: '0.75rem',
            flexWrap: 'wrap',
          }}
        >
          {actions}
        </div>
      )}
    </div>
  );
}

export function SectionHeader({ title }: { title: string }) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '0.75rem',
        marginBottom: '1rem',
      }}
    >
      <div
        style={{
          width: '3px',
          height: '16px',
          backgroundColor: colors.accent,
        }}
      />
      <h3
        style={{
          fontFamily: fonts.mono,
          fontSize: '0.6875rem',
          fontWeight: 600,
          letterSpacing: '0.12em',
          textTransform: 'uppercase',
          color: colors.textMuted,
          margin: 0,
        }}
      >
        {title}
      </h3>
    </div>
  );
}

// ─── Cards / Panels ─────────────────────────────────────────────────────────

export function Card({
  children,
  ...props
}: { children: React.ReactNode } & React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      {...props}
      style={{
        backgroundColor: colors.bgTertiary,
        border: `1px solid ${colors.border}`,
        borderRadius: '4px',
        padding: '1.5rem',
        position: 'relative',
        overflow: 'hidden',
        ...props.style,
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          height: '1px',
          background: `linear-gradient(90deg, ${colors.accent}, transparent 60%)`,
          opacity: 0.5,
        }}
      />
      {children}
    </div>
  );
}

export function CardWithHeader({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <Card>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.625rem',
          marginBottom: '1.25rem',
          paddingBottom: '0.75rem',
          borderBottom: `1px solid ${colors.border}`,
        }}
      >
        <div
          style={{
            width: '4px',
            height: '4px',
            backgroundColor: colors.accent,
            boxShadow: `0 0 6px ${colors.accent}`,
          }}
        />
        <h3
          style={{
            fontFamily: fonts.mono,
            fontSize: '0.6875rem',
            fontWeight: 600,
            letterSpacing: '0.12em',
            textTransform: 'uppercase',
            color: colors.textSecondary,
            margin: 0,
          }}
        >
          {title}
        </h3>
      </div>
      {children}
    </Card>
  );
}

// ─── Stats ───────────────────────────────────────────────────────────────────

export function StatCard({
  label,
  value,
  accent,
}: {
  label: string;
  value: string;
  accent: string;
}) {
  return (
    <div
      style={{
        backgroundColor: colors.bgTertiary,
        border: `1px solid ${colors.border}`,
        borderRadius: '4px',
        padding: '1.25rem',
        position: 'relative',
        overflow: 'hidden',
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          width: '40px',
          height: '40px',
          background: `linear-gradient(135deg, ${accent} 0%, transparent 70%)`,
          opacity: 0.15,
        }}
      />
      <div style={{ position: 'relative' }}>
        <div
          style={{
            fontFamily: fonts.mono,
            fontSize: '0.625rem',
            fontWeight: 600,
            letterSpacing: '0.12em',
            textTransform: 'uppercase',
            color: colors.textMuted,
            marginBottom: '0.625rem',
          }}
        >
          {label}
        </div>
        <div
          style={{
            fontFamily: fonts.display,
            fontSize: '2.5rem',
            fontWeight: 600,
            color: accent,
            lineHeight: 1,
            letterSpacing: '-0.02em',
          }}
        >
          {value}
        </div>
      </div>
    </div>
  );
}

export function MiniStat({
  label,
  count,
  color,
}: {
  label: string;
  count: number;
  color: string;
}) {
  return (
    <div
      style={{
        padding: '1rem',
        backgroundColor: colors.bgSurface,
        borderRadius: '4px',
        textAlign: 'center',
      }}
    >
      <div
        style={{
          fontFamily: fonts.display,
          fontSize: '1.75rem',
          fontWeight: 600,
          color,
          lineHeight: 1,
        }}
      >
        {count}
      </div>
      <MonoLabel>{label}</MonoLabel>
    </div>
  );
}

// ─── Empty States ────────────────────────────────────────────────────────────

export function EmptyState({
  message,
  icon = '—',
}: {
  message: string;
  icon?: string;
}) {
  return (
    <div
      style={{
        textAlign: 'center',
        padding: '3rem 2rem',
      }}
    >
      <div
        style={{
          fontSize: '2rem',
          marginBottom: '1rem',
          opacity: 0.3,
          color: colors.textMuted,
        }}
      >
        {icon}
      </div>
      <div
        style={{
          fontFamily: fonts.mono,
          fontSize: '0.75rem',
          letterSpacing: '0.05em',
          color: colors.textMuted,
        }}
      >
        {message}
      </div>
    </div>
  );
}

// ─── Loading ─────────────────────────────────────────────────────────────────

export function LoadingSpinner() {
  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        padding: '3rem',
      }}
    >
      <div
        style={{
          width: '32px',
          height: '32px',
          border: `2px solid ${colors.border}`,
          borderTopColor: colors.accent,
          borderRadius: '50%',
          animation: 'spin 0.8s linear infinite',
        }}
      />
    </div>
  );
}

export function LoadingText({ message }: { message: string }) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        padding: '3rem',
        gap: '1rem',
      }}
    >
      <div
        style={{
          width: '32px',
          height: '32px',
          border: `2px solid ${colors.border}`,
          borderTopColor: colors.accent,
          borderRadius: '50%',
          animation: 'spin 0.8s linear infinite',
        }}
      />
      <span
        style={{
          fontFamily: fonts.mono,
          fontSize: '0.75rem',
          letterSpacing: '0.05em',
          color: colors.textMuted,
        }}
      >
        {message}
      </span>
    </div>
  );
}

// ─── Badges ─────────────────────────────────────────────────────────────────

function getStatusBadgeColors(
  status: string
): { bgColor: string; textColor: string; label: string } {
  switch (status) {
    case 'pending':
      return { bgColor: colors.borderLight, textColor: colors.textMuted, label: 'PENDING' };
    case 'running':
      return { bgColor: colors.warningDim, textColor: colors.warning, label: 'RUNNING' };
    case 'completed':
      return { bgColor: colors.successDim, textColor: colors.success, label: 'DONE' };
    case 'failed':
      return { bgColor: colors.errorDim, textColor: colors.error, label: 'FAILED' };
    case 'sent':
      return { bgColor: colors.successDim, textColor: colors.success, label: 'SENT' };
    default:
      return { bgColor: colors.borderLight, textColor: colors.textMuted, label: status.toUpperCase() };
  }
}

export function StatusBadge({ status }: { status: string }) {
  const { bgColor, textColor, label } = getStatusBadgeColors(status);
  return <PillBadge bgColor={bgColor} textColor={textColor} label={label} />;
}

export function PillBadge({
  bgColor,
  textColor,
  label,
}: {
  bgColor: string;
  textColor: string;
  label: string;
}) {
  return (
    <span
      style={{
        backgroundColor: bgColor,
        color: textColor,
        fontFamily: fonts.mono,
        fontSize: '0.625rem',
        fontWeight: 600,
        padding: '0.25rem 0.625rem',
        borderRadius: '2px',
        letterSpacing: '0.05em',
      }}
    >
      {label}
    </span>
  );
}

export function Badge({ color, label }: { color: string; label: string }) {
  return (
    <span
      style={{
        backgroundColor: color,
        color: colors.bgPrimary,
        fontFamily: fonts.mono,
        fontSize: '0.5625rem',
        fontWeight: 700,
        padding: '0.25rem 0.5rem',
        borderRadius: '1px',
        textTransform: 'uppercase',
        letterSpacing: '0.08em',
      }}
    >
      {label}
    </span>
  );
}

export function TagChip({ tag }: { tag: string }) {
  return (
    <span
      style={{
        fontFamily: fonts.mono,
        fontSize: '0.5625rem',
        color: colors.textSecondary,
        padding: '0.1875rem 0.5rem',
        backgroundColor: colors.borderLight,
        borderRadius: '2px',
        letterSpacing: '0.03em',
      }}
    >
      {tag}
    </span>
  );
}

function getDocTypeBadgeColors(
  docType: string
): { bgColor: string; textColor: string; label: string } {
  switch (docType) {
    case 'plan':
      return { bgColor: 'rgba(96, 165, 250, 0.12)', textColor: '#60a5fa', label: 'PLAN' };
    case 'specification':
      return { bgColor: 'rgba(167, 139, 250, 0.12)', textColor: '#a78bfa', label: 'SPEC' };
    case 'notes':
      return { bgColor: colors.borderLight, textColor: colors.textSecondary, label: 'NOTES' };
    case 'code':
      return { bgColor: 'rgba(0, 212, 170, 0.12)', textColor: colors.accent, label: 'CODE' };
    default:
      return { bgColor: colors.borderLight, textColor: colors.textMuted, label: docType.toUpperCase() };
  }
}

export function DocTypeBadge({ docType }: { docType: string }) {
  const { bgColor, textColor, label } = getDocTypeBadgeColors(docType);
  return <PillBadge bgColor={bgColor} textColor={textColor} label={label} />;
}

// ─── Utility Components ───────────────────────────────────────────────────────

export function MonoLabel({ children }: { children: React.ReactNode }) {
  return (
    <span
      style={{
        fontFamily: fonts.mono,
        fontSize: '0.5625rem',
        color: colors.textMuted,
        textTransform: 'uppercase',
        letterSpacing: '0.1em',
      }}
    >
      {children}
    </span>
  );
}

export function Timestamp({ ts }: { ts: string }) {
  return (
    <span
      style={{
        fontFamily: fonts.mono,
        color: colors.textMuted,
        fontSize: '0.6875rem',
        letterSpacing: '0.02em',
      }}
    >
      {formatDateTime(ts)}
    </span>
  );
}

export function BackButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      style={{
        backgroundColor: 'transparent',
        color: colors.textSecondary,
        border: `1px solid ${colors.border}`,
        padding: '0.5rem 1.25rem',
        borderRadius: '2px',
        cursor: 'pointer',
        fontFamily: fonts.mono,
        fontSize: '0.75rem',
        fontWeight: 500,
        letterSpacing: '0.05em',
        textTransform: 'uppercase',
        display: 'inline-flex',
        alignItems: 'center',
        gap: '0.5rem',
      }}
    >
      <span style={{ fontSize: '0.875rem' }}>←</span>
      Back
    </button>
  );
}

export function RoleBadge({ label, color }: { label: string; color: string }) {
  const bgColor =
    color === colors.accent
      ? colors.accentDim
      : color === colors.success
        ? colors.successDim
        : colors.borderLight;
  return (
    <span
      style={{
        fontFamily: fonts.mono,
        fontWeight: 700,
        fontSize: '0.625rem',
        letterSpacing: '0.1em',
        color,
        padding: '0.25rem 0.5rem',
        backgroundColor: bgColor,
        borderRadius: '2px',
      }}
    >
      {label}
    </span>
  );
}

export function MediaTypeBadge({
  icon,
  label,
}: {
  icon: string;
  label: string;
}) {
  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '0.375rem',
        padding: '0.25rem 0.5rem',
        backgroundColor: colors.bgSurface,
        border: `1px solid ${colors.border}`,
        borderRadius: '2px',
        fontFamily: fonts.mono,
        fontSize: '0.625rem',
        color: colors.textSecondary,
        letterSpacing: '0.05em',
      }}
    >
      <span style={{ color: colors.accent }}>{icon}</span>
      {label.toUpperCase()}
    </span>
  );
}

export function AccentedItem({
  borderColor,
  children,
  ...props
}: {
  borderColor: string;
  children: React.ReactNode;
} & React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      {...props}
      style={{
        backgroundColor: colors.bgSurface,
        borderRadius: '4px',
        padding: '1rem 1.25rem',
        borderLeft: `3px solid ${borderColor}`,
        ...props.style,
      }}
    >
      {children}
    </div>
  );
}

// ─── Layout Helpers ──────────────────────────────────────────────────────────

export function Row({
  gap,
  children,
}: {
  gap: string;
  children: React.ReactNode;
}) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap,
      }}
    >
      {children}
    </div>
  );
}

export function RowBetween({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
      }}
    >
      {children}
    </div>
  );
}

export function Col({
  gap,
  children,
}: {
  gap: string;
  children: React.ReactNode;
}) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap,
      }}
    >
      {children}
    </div>
  );
}

export function StatusDot({ color, pulse }: { color: string; pulse?: boolean }) {
  return (
    <div
      style={{
        width: '8px',
        height: '8px',
        backgroundColor: color,
        borderRadius: '50%',
        boxShadow: `0 0 12px ${color}`,
        animation: pulse ? 'statusPulse 2s infinite' : 'none',
      }}
    />
  );
}

export function TableHeader({ label }: { label: string }) {
  return (
    <span
      style={{
        fontFamily: fonts.mono,
        fontSize: '0.5625rem',
        fontWeight: 700,
        textTransform: 'uppercase',
        letterSpacing: '0.12em',
        color: colors.textMuted,
      }}
    >
      {label}
    </span>
  );
}

export function GridTwo({ children }: { children: React.ReactNode }) {
  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
        gap: '1.5rem',
      }}
    >
      {children}
    </div>
  );
}

// ─── Buttons ─────────────────────────────────────────────────────────────────

export function Button({
  label,
  ...props
}: { label: string } & React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      type="button"
      {...props}
      style={{
        backgroundColor: colors.bgSurface,
        color: colors.textSecondary,
        border: `1px solid ${colors.border}`,
        padding: '0.5rem 1.25rem',
        borderRadius: '2px',
        cursor: 'pointer',
        fontFamily: fonts.mono,
        fontSize: '0.75rem',
        fontWeight: 500,
        letterSpacing: '0.05em',
        textTransform: 'uppercase',
        transition: 'all 0.15s ease',
        ...props.style,
      }}
    >
      {label}
    </button>
  );
}

export function PrimaryButton({
  label,
  ...props
}: { label: string } & React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      type="button"
      {...props}
      style={{
        backgroundColor: colors.accent,
        color: colors.bgPrimary,
        border: 'none',
        padding: '0.5rem 1.25rem',
        borderRadius: '2px',
        cursor: 'pointer',
        fontFamily: fonts.mono,
        fontSize: '0.75rem',
        fontWeight: 600,
        letterSpacing: '0.05em',
        textTransform: 'uppercase',
        transition: 'all 0.15s ease',
        boxShadow: `0 0 20px ${colors.accentGlow}`,
        ...props.style,
      }}
    >
      {label}
    </button>
  );
}

export function IconButton({
  icon,
  ...props
}: { icon: string } & React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      type="button"
      {...props}
      style={{
        backgroundColor: 'transparent',
        color: colors.textMuted,
        border: 'none',
        padding: '0.5rem',
        borderRadius: '2px',
        cursor: 'pointer',
        fontSize: '1.125rem',
        lineHeight: 1,
        transition: 'all 0.15s ease',
        ...props.style,
      }}
    >
      {icon}
    </button>
  );
}

// ─── Form Helpers ─────────────────────────────────────────────────────────────

export function ToggleSwitch({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
}) {
  return (
    <label
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '0.75rem',
        cursor: 'pointer',
      }}
    >
      <div
        role="switch"
        aria-checked={checked}
        tabIndex={0}
        onClick={() => onChange(!checked)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            onChange(!checked);
          }
        }}
        style={{
          width: '44px',
          height: '24px',
          borderRadius: '12px',
          backgroundColor: checked ? colors.accent : colors.bgSurface,
          border: `1px solid ${checked ? colors.accent : colors.border}`,
          position: 'relative',
          transition: 'background-color 0.2s, border-color 0.2s',
        }}
      >
        <div
          style={{
            position: 'absolute',
            top: '2px',
            left: checked ? '22px' : '2px',
            width: '18px',
            height: '18px',
            borderRadius: '50%',
            backgroundColor: colors.bgPrimary,
            border: `1px solid ${colors.border}`,
            transition: 'left 0.2s',
          }}
        />
      </div>
      <span style={{ fontFamily: fonts.body, fontSize: '0.875rem', color: colors.textPrimary }}>
        {label}
      </span>
    </label>
  );
}

export function FormField({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div>
      <div
        style={{
          fontFamily: fonts.mono,
          fontSize: '0.625rem',
          fontWeight: 600,
          letterSpacing: '0.1em',
          textTransform: 'uppercase',
          color: colors.textMuted,
          marginBottom: '0.5rem',
        }}
      >
        {label}
      </div>
      {children}
    </div>
  );
}

export function InputField({
  value,
  onChange,
  placeholder,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
}) {
  return (
    <input
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      style={{
        width: '100%',
        backgroundColor: colors.bgPrimary,
        color: colors.textPrimary,
        border: `1px solid ${colors.border}`,
        borderRadius: '2px',
        padding: '0.5rem 0.75rem',
        fontFamily: fonts.body,
        fontSize: '0.875rem',
        boxSizing: 'border-box',
      }}
    />
  );
}

export function TextareaField({
  value,
  onChange,
  placeholder,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
}) {
  return (
    <textarea
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      style={{
        width: '100%',
        minHeight: '80px',
        backgroundColor: colors.bgPrimary,
        color: colors.textPrimary,
        border: `1px solid ${colors.border}`,
        borderRadius: '2px',
        padding: '0.5rem 0.75rem',
        fontFamily: fonts.body,
        fontSize: '0.875rem',
        resize: 'vertical',
        boxSizing: 'border-box',
      }}
    />
  );
}

export function SelectField({
  value,
  onChange,
  options,
}: {
  value: string;
  onChange: (v: string) => void;
  options: [string, string][];
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      style={{
        width: '100%',
        backgroundColor: colors.bgPrimary,
        color: colors.textPrimary,
        border: `1px solid ${colors.border}`,
        borderRadius: '2px',
        padding: '0.5rem 0.75rem',
        fontFamily: fonts.body,
        fontSize: '0.875rem',
        boxSizing: 'border-box',
      }}
    >
      {options.map(([v, label]) => (
        <option key={v} value={v}>
          {label}
        </option>
      ))}
    </select>
  );
}

// ─── Page Info Bar ───────────────────────────────────────────────────────────

export function PageInfo({
  page,
  pageSize,
  total,
  totalPages,
}: {
  page: number;
  pageSize: number;
  total: number;
  totalPages: number;
}) {
  const startNum = page * pageSize + 1;
  const endNum = Math.min((page + 1) * pageSize, total);
  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '0.75rem',
        marginBottom: '1rem',
        padding: '0.75rem 1rem',
        backgroundColor: colors.bgTertiary,
        border: `1px solid ${colors.border}`,
        borderRadius: '4px',
      }}
    >
      <MonoLabel>
        SHOWING {startNum}–{endNum} OF {total}
      </MonoLabel>
      <span
        style={{
          fontFamily: fonts.mono,
          fontSize: '0.6875rem',
          color: colors.textSecondary,
          letterSpacing: '0.05em',
        }}
      >
        PAGE {page + 1} / {totalPages}
      </span>
    </div>
  );
}
