#!/usr/bin/env node
'use strict';

const fs = require('fs');
const path = require('path');

const DEPS_JSON = path.join(__dirname, '..', 'ticket-dependencies.json');
const DONE_FILE = path.join(__dirname, '..', '.tickets-done');

if (!fs.existsSync(DONE_FILE)) fs.writeFileSync(DONE_FILE, '');

function loadDeps() {
  if (!fs.existsSync(DEPS_JSON)) {
    console.error(`Error: ${DEPS_JSON} not found. Run aiteam/sync-trello.js first.`);
    process.exit(1);
  }
  try {
    return JSON.parse(fs.readFileSync(DEPS_JSON, 'utf8'));
  } catch (e) {
    console.error(`Error: Could not parse ${DEPS_JSON}: ${e.message}`);
    process.exit(1);
  }
}

function doneSet() {
  const content = fs.readFileSync(DONE_FILE, 'utf8');
  return new Set(
    content.split('\n')
      .map(l => l.trim())
      .filter(l => l && !l.startsWith('#'))
  );
}

function markDone(ids) {
  const done = doneSet();
  const toAdd = [];
  for (const id of ids) {
    if (done.has(id)) {
      console.log(`  already done: ${id}`);
    } else {
      toAdd.push(id);
      console.log(`  + ${id} marked done`);
    }
  }
  if (toAdd.length > 0) {
    fs.appendFileSync(DONE_FILE, toAdd.join('\n') + '\n');
  }
}

function unmark(ids) {
  let lines = fs.readFileSync(DONE_FILE, 'utf8').split('\n');
  for (const id of ids) {
    lines = lines.filter(l => l.trim() !== id);
    console.log(`  - ${id} unmarked`);
  }
  fs.writeFileSync(DONE_FILE, lines.join('\n'));
}

function resetAll() {
  fs.writeFileSync(DONE_FILE, '');
  console.log('All tickets reset.');
  process.exit(0);
}

function listDone() {
  const deps = loadDeps();
  const done = doneSet();
  if (done.size === 0) {
    console.log('No tickets marked done yet.');
    process.exit(0);
  }
  console.log(`Done (${done.size}):`);
  const sorted = [...done].sort();
  for (const t of sorted) {
    const ticket = (deps.tickets || {})[t] || {};
    const name = ticket.name || '?';
    const domain = ticket.domain || '?';
    const wave = ticket.wave ?? '?';
    console.log(`  ${t.padEnd(14)}  W${wave}  [${String(domain).padEnd(10)}]  ${name}`);
  }
  process.exit(0);
}

function summary() {
  const deps = loadDeps();
  const done = doneSet();
  const tickets = deps.tickets || {};
  const waves = deps.waves || {};

  const totalDone = done.size;
  const totalTickets = Object.keys(tickets).length;
  console.log(`Progress: ${totalDone} / ${totalTickets} tickets done\n`);

  const waveKeys = Object.keys(waves).sort((a, b) => Number(a) - Number(b));
  if (waveKeys.length === 0) {
    console.log('No wave data in ticket-dependencies.json.');
    process.exit(0);
  }

  console.log(`  ${'Wave'.padEnd(8)} ${'Name'.padEnd(28)} ${'Done'.padStart(6)} ${'Total'.padStart(6)} ${'%'.padStart(5)}`);
  console.log(`  ${'----'.padEnd(8)} ${'----'.padEnd(28)} ${'----'.padStart(6)} ${'-----'.padStart(6)} ${'--'.padStart(5)}`);

  for (const w of waveKeys) {
    const waveInfo = waves[w];
    const wname = waveInfo.name || `Wave ${w}`;
    const wtotal = waveInfo.ticketCount || (waveInfo.tickets || []).length;
    const wtickets = waveInfo.tickets || [];
    const wdone = wtickets.filter(t => done.has(t)).length;
    const pct = wtotal > 0 ? Math.round((wdone / wtotal) * 100) : 0;
    console.log(`  W${String(w).padEnd(7)} ${wname.padEnd(28)} ${String(wdone).padStart(6)} ${String(wtotal).padStart(6)}  ${String(pct).padStart(3)}%`);
  }
  console.log('');
  process.exit(0);
}

function usage() {
  console.log(`
Usage: node next-tickets.js [OPTIONS]

Show tickets that are unblocked (all dependencies done) and not yet done.

Options:
  -n NUM    Max tickets to show (default: all available)
  -c COL    Filter by Trello column/list name (case-insensitive, substring match)
  -d DOMAIN Filter by domain (case-insensitive)
  -w        Show wave number per ticket
  -l        List all tickets marked done
  -m ID     Mark ticket(s) done (comma-separated). Call after confirmed Ship Complete.
  -u ID     Unmark ticket(s) (comma-separated, for rework/kickback)
  -r        Reset - clear all done tickets
  -s        Summary: progress per wave
  -h        This help

Examples:
  node next-tickets.js              # all unblocked tickets
  node next-tickets.js -n 10 -w    # top 10 with wave info
  node next-tickets.js -c "to do"  # only tickets on the To Do column
  node next-tickets.js -m ES-006   # mark ES-006 done
  node next-tickets.js -m ES-006,AUTH-001  # mark several done at once
  node next-tickets.js -s          # progress summary
`);
  process.exit(0);
}

// --- Arg parsing ---
const args = process.argv.slice(2);
let maxTickets = null;
let domainFilter = null;
let columnFilter = null;
let showWave = false;

for (let i = 0; i < args.length; i++) {
  const arg = args[i];
  switch (arg) {
    case '-h': usage(); break;
    case '-r': resetAll(); break;
    case '-l': listDone(); break;
    case '-s': summary(); break;
    case '-w': showWave = true; break;
    case '-n': maxTickets = parseInt(args[++i], 10); break;
    case '-c': columnFilter = args[++i]; break;
    case '-d': domainFilter = args[++i]; break;
    case '-m': {
      const ids = args[++i].split(',').map(s => s.trim()).filter(Boolean);
      markDone(ids);
      process.exit(0);
      break;
    }
    case '-u': {
      const ids = args[++i].split(',').map(s => s.trim()).filter(Boolean);
      unmark(ids);
      process.exit(0);
      break;
    }
    default:
      console.error(`Unknown flag: ${arg}`);
      usage();
  }
}

// --- Find unblocked tickets ---
const deps = loadDeps();
const done = doneSet();
const tickets = deps.tickets || {};

let available = Object.entries(tickets)
  .filter(([id]) => !done.has(id))
  .filter(([, t]) => {
    const required = t.dependsOn || [];
    return required.length === 0 || required.every(dep => done.has(dep));
  })
  .map(([id, t]) => ({
    id,
    name: t.name || '?',
    domain: t.domain || '?',
    wave: t.wave ?? 99,
    list: t.list || '',
  }))
  .sort((a, b) => a.wave - b.wave || a.domain.localeCompare(b.domain) || a.id.localeCompare(b.id));

if (columnFilter) {
  available = available.filter(t => t.list.toLowerCase().includes(columnFilter.toLowerCase()));
}

if (domainFilter) {
  available = available.filter(t => t.domain.toLowerCase() === domainFilter.toLowerCase());
}

const totalCount = available.length;

if (totalCount === 0) {
  const msg = domainFilter
    ? `No unblocked tickets in domain '${domainFilter}'.`
    : 'No unblocked tickets (all done or dependency graph is empty).';
  console.log(msg);
  process.exit(0);
}

if (maxTickets) {
  available = available.slice(0, maxTickets);
  console.log(`Next ${maxTickets} unblocked tickets (of ${totalCount} available):\n`);
} else {
  console.log(`All ${totalCount} unblocked tickets:\n`);
}

if (showWave) {
  console.log(`  ${'TICKET'.padEnd(14)}  ${'WAVE'.padEnd(4)}  ${'DOMAIN'.padEnd(12)}  NAME`);
  console.log(`  ${'------'.padEnd(14)}  ${'----'.padEnd(4)}  ${'------'.padEnd(12)}  ----`);
  for (const t of available) {
    console.log(`  ${t.id.padEnd(14)}  W${String(t.wave).padEnd(3)}  [${t.domain.padEnd(10)}]  ${t.name}`);
  }
} else {
  console.log(`  ${'TICKET'.padEnd(14)}  ${'DOMAIN'.padEnd(12)}  NAME`);
  console.log(`  ${'------'.padEnd(14)}  ${'------'.padEnd(12)}  ----`);
  for (const t of available) {
    console.log(`  ${t.id.padEnd(14)}  [${t.domain.padEnd(10)}]  ${t.name}`);
  }
}
