import React from 'react';
import { PageHeader, Card, SectionHeader } from '../components/UI';
import { colors, fonts } from '../theme';

const features: { section: string; items: string[] }[] = [
  {
    section: 'Messaging',
    items: [
      'Telegram integration',
      'Multi-chat support',
      'Media handling',
    ],
  },
  {
    section: 'AI',
    items: [
      'Claude integration',
      'Voice transcription',
      'Semantic search',
    ],
  },
  {
    section: 'Work Management',
    items: [
      'Projects',
      'Tasks',
      'Documents',
      'Live board',
      'Agent loops',
    ],
  },
  {
    section: 'System',
    items: [
      'Cron jobs',
      'Logging',
      'Tunnel access',
      'Setup wizard',
    ],
  },
];

export function Capabilities(): React.ReactElement {
  return (
    <div>
      <PageHeader title="Capabilities" />
      <Card>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
          {features.map(({ section, items }) => (
            <div key={section}>
              <SectionHeader title={section} />
              <ul
                style={{
                  listStyle: 'none',
                  margin: 0,
                  padding: 0,
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '0.5rem',
                }}
              >
                {items.map((item) => (
                  <li
                    key={item}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '0.75rem',
                      fontFamily: fonts.body,
                      fontSize: '0.875rem',
                      color: colors.textPrimary,
                    }}
                  >
                    <span
                      style={{
                        width: '6px',
                        height: '6px',
                        borderRadius: '50%',
                        backgroundColor: colors.accent,
                        flexShrink: 0,
                      }}
                    />
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}
