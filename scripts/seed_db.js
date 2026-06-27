#!/usr/bin/env node
/**
 * Seed the time-tracker dev database with test data.
 *
 * Usage:
 *   node scripts/seed_db.js empty   — wipe everything (first-launch state)
 *   node scripts/seed_db.js medium  — ~2 weeks, couple of goal changes, some off days
 *   node scripts/seed_db.js heavy   — ~11 months, massive streak
 *
 * Requires the sqlite3 CLI (same dependency as scripts/db_console.ps1).
 * Run the app at least once first so the database file exists.
 */

import { execSync } from 'child_process'
import { readFileSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, '..')

// ---------------------------------------------------------------------------
// DB path — mirrors db_console.ps1
// ---------------------------------------------------------------------------

function getDbPath() {
  const conf = JSON.parse(readFileSync(join(root, 'src-tauri', 'tauri.conf.json'), 'utf8'))
  const id = conf.identifier
  if (process.platform === 'win32') {
    return join(process.env.APPDATA, id, 'time-tracker.db')
  }
  if (process.platform === 'darwin') {
    return join(process.env.HOME, 'Library', 'Application Support', id, 'time-tracker.db')
  }
  const xdg = process.env.XDG_DATA_HOME ?? join(process.env.HOME, '.local', 'share')
  return join(xdg, id, 'time-tracker.db')
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Midnight of `d` (a Date) in local time, as Unix milliseconds. */
function midnightMs(d) {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate(), 0, 0, 0, 0).getTime()
}

/** Subtract `n` days from today, returning a Date at local midnight. */
function daysAgo(n) {
  const d = new Date()
  d.setHours(0, 0, 0, 0)
  d.setDate(d.getDate() - n)
  return d
}

/** ISO date string (YYYY-MM-DD) for a Date. */
function isoDate(d) {
  return d.toISOString().slice(0, 10)
}

/**
 * Seeded LCG — gives reproducible "random" numbers without Math.random().
 * Returns a function rng(min, max) → integer in [min, max].
 */
function makeRng(seed) {
  let s = seed >>> 0
  return function (min, max) {
    s = (Math.imul(1664525, s) + 1013904223) >>> 0
    return min + (s % (max - min + 1))
  }
}

/**
 * Build INSERT statements for work sessions on `day` (a Date) totalling
 * approximately `targetMs` of tracked time, with short gaps between sessions.
 * Sessions start between 08:00–10:00 and won't push past 21:00.
 */
function buildSessionSql(rng, day, targetMs) {
  const midnight = midnightMs(day)
  let offsetMs = rng(8 * 3600, 10 * 3600) * 1000
  let remaining = targetMs
  const rows = []

  while (remaining > 0) {
    const sessionMs = Math.min(remaining, rng(20, 90) * 60 * 1000)
    const startMs = midnight + offsetMs
    const endMs = startMs + sessionMs

    if (endMs > midnight + 21 * 3600 * 1000) break

    rows.push(`INSERT INTO intervals (start_ms, end_ms) VALUES (${startMs}, ${endMs});`)
    remaining -= sessionMs
    offsetMs += sessionMs + rng(10, 45) * 60 * 1000
  }
  return rows
}

// ---------------------------------------------------------------------------
// Scenario 1: Empty
// ---------------------------------------------------------------------------

function buildEmpty() {
  return [
    'DELETE FROM intervals;',
    'DELETE FROM daily_goals;',
  ]
}

// ---------------------------------------------------------------------------
// Scenario 2: Medium  (~2 weeks of data)
// ---------------------------------------------------------------------------

function buildMedium() {
  const rng = makeRng(42)
  const today = new Date()
  today.setHours(0, 0, 0, 0)

  const sql = [
    'DELETE FROM intervals;',
    'DELETE FROM daily_goals;',
    // Goal: 2 h from 14 days ago, bumped to 3 h one week ago
    `INSERT INTO daily_goals (day, goal_ms) VALUES ('${isoDate(daysAgo(14))}', ${2 * 3600_000});`,
    `INSERT INTO daily_goals (day, goal_ms) VALUES ('${isoDate(daysAgo(7))}',  ${3 * 3600_000});`,
  ]

  // Mark weekends as off days; also pick one random weekday to skip
  const offOffsets = new Set()
  for (let i = 0; i <= 14; i++) {
    const d = daysAgo(i)
    if (d.getDay() === 0 || d.getDay() === 6) offOffsets.add(i)
  }
  const weekdayOffsets = []
  for (let i = 1; i <= 13; i++) {
    const d = daysAgo(i)
    if (d.getDay() > 0 && d.getDay() < 6) weekdayOffsets.push(i)
  }
  offOffsets.add(weekdayOffsets[rng(0, weekdayOffsets.length - 1)])

  // Historical days: yesterday → 14 days ago
  for (let i = 1; i <= 14; i++) {
    if (offOffsets.has(i)) continue
    const d = daysAgo(i)
    const goalMs = i <= 7 ? 3 * 3600_000 : 2 * 3600_000
    const targetMs = goalMs + rng(0, 45) * 60_000
    sql.push(...buildSessionSql(rng, d, targetMs))
  }

  // Today: one session already ended — clamp end to now so we never write future timestamps
  const todayMidnight = midnightMs(today)
  const startMs = todayMidnight + 9 * 3600_000
  const endMs = Math.min(startMs + 5_400_000, Date.now())
  if (endMs > startMs) {
    sql.push(`INSERT INTO intervals (start_ms, end_ms) VALUES (${startMs}, ${endMs});`)
  }

  return sql
}

// ---------------------------------------------------------------------------
// Scenario 3: Heavy  (~11 months, massive streak)
// ---------------------------------------------------------------------------

function buildHeavy() {
  const rng = makeRng(99)
  const today = new Date()
  today.setHours(0, 0, 0, 0)

  const sql = [
    'DELETE FROM intervals;',
    'DELETE FROM daily_goals;',
  ]

  // Goal milestones
  const goalChanges = [
    [daysAgo(335), 2    * 3600_000],
    [daysAgo(180), Math.round(2.5 * 3600_000)],
    [daysAgo(90),  3    * 3600_000],
  ]
  for (const [d, ms] of goalChanges) {
    sql.push(`INSERT INTO daily_goals (day, goal_ms) VALUES ('${isoDate(d)}', ${ms});`)
  }

  function goalOn(d) {
    let g = 0
    for (const [gd, ms] of goalChanges) {
      if (gd.getTime() <= d.getTime()) g = ms
    }
    return g
  }

  function forRange(startOffset, endOffset, fn) {
    for (let i = endOffset; i >= startOffset; i--) {
      fn(daysAgo(i), i)
    }
  }

  // Phase 1: sporadic early period (days 335 → 211 ago)
  forRange(211, 335, (d) => {
    if (rng(0, 99) < 45) return          // 45% off days
    const goal = goalOn(d)
    const target = rng(0, 99) < 30       // 30% miss
      ? Math.round(goal * rng(30, 85) / 100)
      : goal + rng(0, 30) * 60_000
    sql.push(...buildSessionSql(rng, d, target))
  })

  // Phase 2: building habits (days 210 → 122 ago)
  forRange(122, 210, (d) => {
    if (rng(0, 99) < 20) return          // 20% off days
    const goal = goalOn(d)
    const target = rng(0, 99) < 15       // 15% miss
      ? Math.round(goal * rng(40, 90) / 100)
      : goal + rng(0, 40) * 60_000
    sql.push(...buildSessionSql(rng, d, target))
  })

  // Phase 3: the massive streak (days 121 → 1 ago), every day meets goal
  forRange(1, 121, (d) => {
    const goal = goalOn(d)
    const target = goal + rng(0, 45) * 60_000
    sql.push(...buildSessionSql(rng, d, target))
  })

  // Today: one session already ended — clamp end to now so we never write future timestamps
  const todayMidnight = midnightMs(today)
  const goal = goalOn(today)
  const target = goal + rng(0, 45) * 60_000
  const startMs = todayMidnight + 9 * 3600_000
  const endMs = Math.min(startMs + target, Date.now())
  if (endMs > startMs) {
    sql.push(`INSERT INTO intervals (start_ms, end_ms) VALUES (${startMs}, ${endMs});`)
  }

  return sql
}

// ---------------------------------------------------------------------------
// Scenario 4: Today  (adds today's intervals without wiping historical data)
// ---------------------------------------------------------------------------

function buildToday() {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const midnight = today.getTime()
  const cap = midnight + 21 * 3600 * 1000  // 9 PM hard stop

  // [startOffsetSec, durationSec] — a realistic workday spread across the day
  const sessions = [
    [ 9 * 3600,            95 * 60],  // 9:00–10:35
    [10 * 3600 + 50 * 60,  70 * 60],  // 10:50–12:00
    [13 * 3600,            85 * 60],  // 1:00–2:25
    [14 * 3600 + 45 * 60,  80 * 60],  // 2:45–4:05
    [17 * 3600 + 30 * 60,  90 * 60],  // 5:30–7:00
    [19 * 3600 + 20 * 60,  70 * 60],  // 7:20–8:30
  ]

  const sql = []
  for (const [offsetSec, durationSec] of sessions) {
    const startMs = midnight + offsetSec * 1000
    const endMs   = startMs + durationSec * 1000
    if (startMs >= cap) break
    sql.push(`INSERT INTO intervals (start_ms, end_ms) VALUES (${startMs}, ${Math.min(endMs, cap)});`)
  }
  return sql
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

const SCENARIOS = { empty: buildEmpty, medium: buildMedium, heavy: buildHeavy, today: buildToday }

const scenario = process.argv[2]
if (!scenario || !SCENARIOS[scenario]) {
  console.error(`Usage: node scripts/seed_db.js [${Object.keys(SCENARIOS).join(' | ')}]`)
  process.exit(1)
}

const dbPath = getDbPath()
try {
  execSync(`sqlite3 "${dbPath}" ".tables"`, { stdio: 'pipe' })
} catch {
  console.error(`Database not found or sqlite3 not available: ${dbPath}`)
  console.error('Run the app at least once to create the database, then re-run this script.')
  process.exit(1)
}

const stmts = SCENARIOS[scenario]()
const sql = `BEGIN;\n${stmts.join('\n')}\nCOMMIT;\n`

execSync(`sqlite3 "${dbPath}"`, { input: sql, stdio: ['pipe', 'inherit', 'inherit'] })

const [count] = execSync(`sqlite3 "${dbPath}" "SELECT COUNT(*) FROM intervals"`, { encoding: 'utf8' }).trim().split('\n')
const [goals] = execSync(`sqlite3 "${dbPath}" "SELECT COUNT(*) FROM daily_goals"`, { encoding: 'utf8' }).trim().split('\n')

console.log(`${scenario} — done.  ${count} intervals, ${goals} goal change(s).`)
