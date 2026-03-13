#!/usr/bin/env node
'use strict';

/**
 * sync-trello.js
 *
 * Pulls the QIC Trello board state and rebuilds ticket-dependencies.json
 * and .tickets-done to match reality.
 *
 * Usage:
 *   node aiteam/sync-trello.js             # full sync, writes files
 *   node aiteam/sync-trello.js --dry-run   # show changes, write nothing
 *   node aiteam/sync-trello.js --new-only  # only process new cards
 *   node aiteam/sync-trello.js --no-infer  # skip claude -p inference for new cards
 */

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// --- Load .env from repo root ---
const REPO_ROOT_ENV = path.join(__dirname, '..', '.env');
if (fs.existsSync(REPO_ROOT_ENV)) {
  for (const line of fs.readFileSync(REPO_ROOT_ENV, 'utf8').split('\n')) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const eq = trimmed.indexOf('=');
    if (eq < 0) continue;
    const key = trimmed.slice(0, eq).trim();
    const val = trimmed.slice(eq + 1).trim();
    if (!process.env[key]) process.env[key] = val;
  }
}

// --- Config ---

const TRELLO_KEY   = process.env.TRELLO_API_KEY || process.env.TRELLO_KEY;
const TRELLO_TOKEN = process.env.TRELLO_TOKEN;
const BOARD_ID     = process.env.TRELLO_BOARD_ID || '69a5bb4b56b71b138fb3f2be';

if (!TRELLO_KEY || !TRELLO_TOKEN) {
  console.error('Error: TRELLO_API_KEY and TRELLO_TOKEN environment variables are required.');
  console.error('Set them in ~/git/qic/.env or export them before running.');
  process.exit(1);
}

const REPO_ROOT  = path.join(__dirname, '..');
const DEPS_FILE  = path.join(REPO_ROOT, 'ticket-dependencies.json');
const DONE_FILE  = path.join(REPO_ROOT, '.tickets-done');

// Columns whose cards count as "done" (shipped, under review, or signed off)
const DONE_COLUMNS = new Set([
  'dev complete',
  'for backend testing',
  'human review',
  'human reviewed - working',
  'qic reviewed',
  'finally done',
  'done',
  'closed',
]);

// Columns whose cards are available for work (or being reworked)
const TODO_COLUMNS = new Set(['to do', 'backlog', 'ready', 'in progress', 'todo']);

// Columns whose cards are neither done nor available (ignore for sync purposes)
// 'blocked' stays as-is in .tickets-done — not unmarked, not newly marked

// --- Args ---

const args = process.argv.slice(2);
const DRY_RUN  = args.includes('--dry-run');
const NEW_ONLY = args.includes('--new-only');
const NO_INFER = args.includes('--no-infer');

if (args.includes('-h') || args.includes('--help')) {
  console.log(`
Usage: node aiteam/sync-trello.js [options]

Options:
  --dry-run    Show proposed changes without writing any files
  --new-only   Only process new cards (skip done/undone syncing)
  --no-infer   Skip claude -p dependency inference for new cards
               (new cards added with empty deps; edit ticket-dependencies.json manually)
  -h, --help   Show this help

What it does:
  1. Fetches all cards from the Trello board
  2. Maps columns to done/undone status
  3. Marks newly done cards in .tickets-done
  4. Unmarks kicked-back cards (and their downstream dependents)
  5. For new cards not in ticket-dependencies.json:
       - Calls claude -p to infer dependencies and domain (unless --no-infer)
       - Always previews the result before writing
`);
  process.exit(0);
}

if (DRY_RUN) console.log('[dry-run] No files will be written.\n');

// --- HTTP helper ---

function trelloGet(path) {
  const sep = path.includes('?') ? '&' : '?';
  const url = `https://api.trello.com/1${path}${sep}key=${TRELLO_KEY}&token=${TRELLO_TOKEN}`;
  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        if (res.statusCode !== 200) {
          reject(new Error(`Trello API ${res.statusCode} for ${path}: ${body.slice(0, 200)}`));
          return;
        }
        try { resolve(JSON.parse(body)); }
        catch (e) { reject(new Error(`JSON parse error for ${path}: ${e.message}`)); }
      });
    }).on('error', reject);
  });
}

// --- File helpers ---

function loadDeps() {
  if (!fs.existsSync(DEPS_FILE)) {
    return { tickets: {}, waves: {} };
  }
  return JSON.parse(fs.readFileSync(DEPS_FILE, 'utf8'));
}

function loadDoneSet() {
  if (!fs.existsSync(DONE_FILE)) return new Set();
  return new Set(
    fs.readFileSync(DONE_FILE, 'utf8')
      .split('\n')
      .map(l => l.trim())
      .filter(l => l && !l.startsWith('#'))
  );
}

function writeDoneSet(set) {
  fs.writeFileSync(DONE_FILE, [...set].sort().join('\n') + '\n');
}

// --- Label extraction + name cleaning ---

function extractLabel(cardName) {
  const m = cardName.match(/\b([A-Z][A-Z0-9]+-\d+)\b/);
  return m ? m[1] : null;
}

// Strip leading "LABEL: " prefix(es) from card name for clean storage
// e.g. "KYC-007: KYC-007: Be Prompted..." -> "Be Prompted..."
function cleanName(cardName, label) {
  let name = cardName;
  // Remove all leading "LABEL: " occurrences
  const prefix = new RegExp(`^(${label}:\\s*)+`, 'i');
  name = name.replace(prefix, '').trim();
  return name || cardName;
}

// --- Downstream cascade unmark ---

function downstreamOf(ticketId, allTickets) {
  // Returns all ticket IDs that (directly or transitively) depend on ticketId
  const result = new Set();
  const queue = [ticketId];
  while (queue.length > 0) {
    const current = queue.shift();
    for (const [id, t] of Object.entries(allTickets)) {
      if ((t.dependsOn || []).includes(current) && !result.has(id)) {
        result.add(id);
        queue.push(id);
      }
    }
  }
  return result;
}

// --- Wave recalculation ---

function recalcWave(ticketId, allTickets, memo = {}) {
  if (memo[ticketId] !== undefined) return memo[ticketId];
  const deps = (allTickets[ticketId]?.dependsOn || []);
  if (deps.length === 0) {
    memo[ticketId] = 0;
    return 0;
  }
  const maxDepWave = Math.max(...deps.map(d => recalcWave(d, allTickets, memo)));
  memo[ticketId] = maxDepWave + 1;
  return memo[ticketId];
}

function rebuildWaves(allTickets) {
  const memo = {};
  for (const id of Object.keys(allTickets)) {
    recalcWave(id, allTickets, memo);
  }
  // Assign wave to each ticket
  for (const [id, t] of Object.entries(allTickets)) {
    t.wave = memo[id] ?? 0;
  }
  // Build waves index
  const waveMap = {};
  for (const [id, t] of Object.entries(allTickets)) {
    const w = String(t.wave);
    if (!waveMap[w]) waveMap[w] = { name: `Wave ${w}`, ticketCount: 0, tickets: [] };
    waveMap[w].tickets.push(id);
    waveMap[w].ticketCount++;
  }
  // Sort tickets within each wave
  for (const w of Object.values(waveMap)) {
    w.tickets.sort();
  }
  return waveMap;
}

// --- Claude inference for new cards ---

function inferDepsViaClaude(card, comments, existingTickets) {
  const commentsText = comments
    .map(c => `[${c.date?.slice(0, 10) || ''}] ${c.data?.text || ''}`)
    .join('\n');

  const existingSummary = Object.entries(existingTickets)
    .map(([id, t]) => `${id} (wave ${t.wave ?? '?'}, domain: ${t.domain ?? '?'}): ${t.name}`)
    .join('\n');

  const prompt = `You are analysing a Trello ticket for QIC Trader, a crypto P2P trading platform (Rust backend + Next.js frontend).

Ticket name: ${card.name}
Description:
${card.desc || '(none)'}

Comments (oldest first):
${commentsText || '(none)'}

Existing tickets in the dependency graph:
${existingSummary || '(none yet)'}

Return ONLY valid JSON — no explanation, no markdown, no code fences:
{
  "dependsOn": ["TICKET-ID"],
  "domain": "string",
  "wave": number
}

Rules:
- dependsOn: list ticket IDs this ticket cannot start before (empty array if none)
- domain: one word from this list only: Auth, Trade, Escrow, Wallet, Ledger, KYC, Affiliate, Admin, Marketplace, Notify, Infra, Profile, UI
- wave: 0 if no dependencies, otherwise max(wave of deps) + 1
- Only list dependencies that exist in the graph above
- If uncertain, return empty dependsOn rather than guessing`;

  try {
    const result = execSync(
      `claude -p ${JSON.stringify(prompt)}`,
      { encoding: 'utf8', timeout: 60000, stdio: ['pipe', 'pipe', 'pipe'] }
    );
    // Strip any markdown fences in case Claude adds them
    const cleaned = result.replace(/```json\n?/g, '').replace(/```\n?/g, '').trim();
    return JSON.parse(cleaned);
  } catch (e) {
    console.warn(`  [warn] claude -p failed for ${card.name}: ${e.message}`);
    return null;
  }
}

// --- Main ---

async function main() {
  console.log('Fetching Trello board...');

  // 1. Fetch lists
  const lists = await trelloGet(`/boards/${BOARD_ID}/lists?fields=id,name`);
  const listMap = {};
  for (const l of lists) {
    listMap[l.id] = l.name;
  }
  console.log(`  ${lists.length} lists found: ${lists.map(l => l.name).join(', ')}`);

  // 2. Fetch all cards
  const cards = await trelloGet(
    `/boards/${BOARD_ID}/cards/open?fields=id,name,desc,labels,idList,pos`
  );
  console.log(`  ${cards.length} cards fetched`);

  // 3. Load local state
  const deps = loadDeps();
  const doneSet = loadDoneSet();
  const existingTickets = deps.tickets || {};

  const newDoneSet = new Set(doneSet);
  const changes = { marked: [], unmarked: [], cascadeUnmarked: [], newCards: [] };

  // 4. Process each card
  for (const card of cards) {
    const listName = (listMap[card.idList] || '').toLowerCase();
    const label = extractLabel(card.name);
    if (!label) continue; // Skip cards without a ticket label

    const isDoneColumn = DONE_COLUMNS.has(listName);
    const isTodoColumn = TODO_COLUMNS.has(listName);
    const wasInDone = newDoneSet.has(label);
    const isKnown = Boolean(existingTickets[label]);

    if (!NEW_ONLY) {
      if (isDoneColumn && !wasInDone) {
        // Newly done
        newDoneSet.add(label);
        changes.marked.push(label);
      } else if (isTodoColumn && wasInDone) {
        // Kicked back - unmark it and its downstream dependents
        newDoneSet.delete(label);
        changes.unmarked.push(label);
        const downstream = downstreamOf(label, existingTickets);
        for (const dep of downstream) {
          if (newDoneSet.has(dep)) {
            newDoneSet.delete(dep);
            changes.cascadeUnmarked.push(dep);
          }
        }
      }
    }

    if (!isKnown) {
      changes.newCards.push(card);
    }
  }

  // 5. Report done/undone changes
  if (changes.marked.length > 0) {
    console.log(`\nNewly done (${changes.marked.length}):`);
    for (const id of changes.marked) console.log(`  + ${id}`);
  }
  if (changes.unmarked.length > 0) {
    console.log(`\nKicked back / rework (${changes.unmarked.length}):`);
    for (const id of changes.unmarked) console.log(`  - ${id}`);
  }
  if (changes.cascadeUnmarked.length > 0) {
    console.log(`\nCascade-unmarked downstream dependents (${changes.cascadeUnmarked.length}):`);
    for (const id of changes.cascadeUnmarked) console.log(`  - ${id} (depended on kicked-back ticket)`);
  }

  // 6. Handle new cards
  if (changes.newCards.length > 0) {
    console.log(`\nNew cards not in ticket-dependencies.json (${changes.newCards.length}):`);

    for (const card of changes.newCards) {
      const label = extractLabel(card.name);
      console.log(`\n  ${label}: ${card.name}`);

      let inferred = null;

      if (!NO_INFER) {
        process.stdout.write('  Inferring dependencies via claude -p...');
        // Fetch comments for this card
        let comments = [];
        try {
          comments = await trelloGet(
            `/cards/${card.id}/actions?filter=commentCard&limit=50`
          );
        } catch (e) {
          // Comments are optional - carry on
        }
        inferred = inferDepsViaClaude(card, comments, existingTickets);
        if (inferred) {
          console.log(' done');
          console.log(`    domain:    ${inferred.domain}`);
          console.log(`    wave:      ${inferred.wave}`);
          console.log(`    dependsOn: [${(inferred.dependsOn || []).join(', ') || 'none'}]`);
        } else {
          console.log(' FAILED - adding with empty deps (edit manually)');
        }
      } else {
        console.log('  --no-infer set, adding with empty deps');
      }

      existingTickets[label] = {
        name: cleanName(card.name, label),
        domain: inferred?.domain || 'Unknown',
        wave: inferred?.wave ?? 0,
        dependsOn: inferred?.dependsOn || [],
      };
    }
  }

  // 7. Recalculate all waves from dependency graph
  const newWaves = rebuildWaves(existingTickets);
  deps.tickets = existingTickets;
  deps.waves = newWaves;

  // 8. Summary
  const totalTickets = Object.keys(existingTickets).length;
  const totalDone = newDoneSet.size;
  console.log(`\nSummary: ${totalDone} / ${totalTickets} tickets done`);

  if (
    changes.marked.length === 0 &&
    changes.unmarked.length === 0 &&
    changes.newCards.length === 0
  ) {
    console.log('No changes detected.');
    if (!DRY_RUN) {
      // Still write to pick up any wave recalculation changes
      fs.writeFileSync(DEPS_FILE, JSON.stringify(deps, null, 2) + '\n');
    }
    return;
  }

  // 9. Write files
  if (DRY_RUN) {
    console.log('\n[dry-run] Would write:');
    console.log(`  ${DEPS_FILE}`);
    console.log(`  ${DONE_FILE}`);
    if (changes.newCards.length > 0) {
      console.log('\n[dry-run] Review the inferred dependencies above.');
      console.log('  Re-run without --dry-run to commit these changes.');
    }
  } else {
    fs.writeFileSync(DEPS_FILE, JSON.stringify(deps, null, 2) + '\n');
    writeDoneSet(newDoneSet);
    console.log('\nFiles written:');
    console.log(`  ${DEPS_FILE}`);
    console.log(`  ${DONE_FILE}`);
  }

  if (changes.newCards.length > 0 && !NO_INFER) {
    console.log('\nReview inferred dependencies before the next /goteam run.');
    console.log(`Edit ${DEPS_FILE} if any edges are wrong.`);
  }
}

main().catch(e => {
  console.error(`\nError: ${e.message}`);
  process.exit(1);
});
