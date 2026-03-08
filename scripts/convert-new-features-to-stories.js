#!/usr/bin/env node
/**
 * Parse NEW_FEATURES.md and write one story file per story into stories/new stories/
 * Format matches existing stories (e.g. AFFILIATE-004-view-earnings.md)
 */
const fs = require('fs');
const path = require('path');

const srcPath = path.join(__dirname, '..', 'NEW_FEATURES.md');
const outDir = path.join(__dirname, '..', 'stories', 'new stories');
const content = fs.readFileSync(srcPath, 'utf8');

// Split by ### ID: Title blocks (and ---). User story may have commas in goal/benefit.
const blocks = content.split(/(?=### [A-Z]+-\d+:)/).filter((b) => /^### [A-Z]+-\d+:/.test(b));
const stories = [];
for (const block of blocks) {
  const idMatch = block.match(/^### ([A-Z]+-\d+): ([^\n]+)/);
  if (!idMatch) continue;
  const [, id, title] = idMatch;
  let aOrAn, role, goal, benefit;
  // Match: "As [a|an] ROLE, I want GOAL[, | —] so that BENEFIT." or "I want to GOAL." (no so that). Allow "As QicTrader" (no a/an). Try "an" before "a".
  const storyMatchWithSoThat = block.match(/\*\*User Story:\*\* As (an|a)? ?([^,]+), I want (.+?)(?:,| —) so that ([^.]+)\./s);
  if (storyMatchWithSoThat) {
    [, aOrAn, role, goal, benefit] = storyMatchWithSoThat;
    aOrAn = (aOrAn || 'a').toLowerCase();
    goal = goal.trim();
    benefit = benefit.trim();
  } else {
    const storyMatchNoSoThat = block.match(/\*\*User Story:\*\* As (an|a)? ?([^,]+), I want (.+?)\./s);
    if (!storyMatchNoSoThat) continue;
    [, aOrAn, role, goal, benefit] = storyMatchNoSoThat;
    aOrAn = (aOrAn || 'a').toLowerCase();
    goal = goal.trim();
    benefit = 'I can benefit from this capability';
  }
  role = role.trim();
  const criteriaMatch = block.match(/\*\*Acceptance Criteria:\*\*\n([\s\S]*?)(?=\n---|\n### |\n## Epic|$)/);
  const criteriaBlock = criteriaMatch ? criteriaMatch[1] : '';
  const criteria = criteriaBlock
    .trim()
    .split(/\n(?=\d+\.)/)
    .filter(Boolean)
    .map((line) => line.replace(/^\d+\.\s*/, '- '))
    .join('\n');
  // Normalize goal: "noun to verb" -> "have noun verb" (e.g. "the system to generate"); "to verb" -> "verb" to avoid "I want to to verb"
  let goalPhrase = goal;
  if (goalPhrase.startsWith('to ')) {
    goalPhrase = goalPhrase.slice(3);
  } else if (/^[a-z]/.test(goalPhrase) && goalPhrase.includes(' to ')) {
    goalPhrase = 'have ' + goalPhrase.replace(/ to /, ' ', 1);
  }
  stories.push({
    id,
    title,
    aOrAn: aOrAn === 'an' ? 'an' : 'a',
    role,
    goal: goalPhrase,
    benefit: benefit.trim(),
    criteria,
  });
}

// Slug for filename: ID + lowercase hyphenated title
function slug(s) {
  return s
    .replace(/[^a-zA-Z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
    .toLowerCase();
}

const template = (s) => `# ${s.id}: ${s.title}

**As ${s.aOrAn}** ${s.role},
**I want to** ${s.goal},
**so that** ${s.benefit}.

## Acceptance Criteria

${s.criteria}

## Testing Instructions

1. (To be added.)
`;

stories.forEach((s) => {
  const filename = `${s.id}-${slug(s.title)}.md`;
  const filepath = path.join(outDir, filename);
  fs.writeFileSync(filepath, template(s), 'utf8');
  console.log('Wrote', filename);
});
console.log('Total:', stories.length, 'stories');
