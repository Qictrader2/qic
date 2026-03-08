import React from 'react';
import { useNavigate } from 'react-router-dom';
import { PrimaryButton, Button } from '../components/UI';
import { colors, fonts } from '../theme';

export function Welcome(): React.ReactElement {
  const navigate = useNavigate();

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: '60vh',
        padding: '2rem',
        maxWidth: '600px',
        margin: '0 auto',
      }}
    >
      <h1
        style={{
          fontFamily: fonts.display,
          fontSize: '2.5rem',
          fontWeight: 700,
          color: colors.accent,
          margin: 0,
          marginBottom: '0.5rem',
          letterSpacing: '0.02em',
        }}
      >
        TWOLEBOT
      </h1>
      <p
        style={{
          fontFamily: fonts.mono,
          fontSize: '0.875rem',
          color: colors.textSecondary,
          margin: 0,
          marginBottom: '1.5rem',
          letterSpacing: '0.05em',
        }}
      >
        Personal AI Command Console
      </p>
      <p
        style={{
          fontFamily: fonts.body,
          fontSize: '0.9375rem',
          color: colors.textPrimary,
          lineHeight: 1.7,
          margin: 0,
          marginBottom: '2rem',
          textAlign: 'center',
        }}
      >
        A personal AI assistant with Telegram integration, semantic search, work management,
        and more. Get started by running through the setup wizard.
      </p>
      <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', justifyContent: 'center' }}>
        <PrimaryButton label="Get Started" onClick={() => navigate('/setup')} />
        <Button label="Go to Dashboard" onClick={() => navigate('/')} />
      </div>
    </div>
  );
}
