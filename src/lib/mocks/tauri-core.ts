// Browser-mode mock for @tauri-apps/api/core
// Active only when running `pnpm dev` (no Tauri backend). Controlled via vite.config.js alias.

const _runningStart = Date.now() - 27 * 60 * 1000
const _todayStart = new Date(new Date().setHours(0, 0, 0, 0)).getTime()

type TimerState =
  | { state: 'Empty' }
  | { state: 'Running'; start_ms: number }
  | { state: 'Paused'; last_duration_ms: number }

let _timerState: TimerState = { state: 'Running', start_ms: _runningStart }

const _dispatch: Record<string, () => unknown> = {
  get_timer_state: () => _timerState,
  begin_interval: () => {
    const now = Date.now()
    _timerState = { state: 'Running', start_ms: now }
    return now
  },
  end_interval: () => {
    _timerState = { state: 'Empty' }
    return null
  },
  end_interval_at: () => {
    _timerState = { state: 'Empty' }
    return null
  },
  get_intervals: () => [
    { id: 1, start_ms: _todayStart + 9 * 3600000, end_ms: _todayStart + 10 * 3600000 },
    { id: 2, start_ms: _todayStart + 10.5 * 3600000, end_ms: _todayStart + 11.5 * 3600000 },
    { id: 3, start_ms: _todayStart + 13 * 3600000, end_ms: null },
  ],
  get_range_total: () => ({
    total_ms: 2 * 3600000,
    most_recent: { id: 3, start_ms: _todayStart + 13 * 3600000, end_ms: null },
  }),
  get_current_goal: () => ({ day: new Date().toISOString().slice(0, 10), goal_ms: 8 * 3600000 }),
  set_daily_goal: () => null,
  update_interval: () => null,
  delete_interval: () => null,
  show_system_popup: () => null,
  get_daily_totals: () => [
    { day: '2026-06-21', total_ms: 0 },
    { day: '2026-06-22', total_ms: 7 * 3600000 },
    { day: '2026-06-23', total_ms: 8 * 3600000 },
    { day: '2026-06-24', total_ms: 6 * 3600000 },
    { day: '2026-06-25', total_ms: 9 * 3600000 },
    { day: '2026-06-26', total_ms: 7.25 * 3600000 },
    { day: '2026-06-27', total_ms: 2 * 3600000 },
  ],
  get_daily_goals_for_range: () => [
    { day: '2026-06-22', goal_ms: 8 * 3600000 },
    { day: '2026-06-23', goal_ms: 8 * 3600000 },
    { day: '2026-06-24', goal_ms: 8 * 3600000 },
    { day: '2026-06-25', goal_ms: 8 * 3600000 },
    { day: '2026-06-26', goal_ms: 8 * 3600000 },
    { day: '2026-06-27', goal_ms: 8 * 3600000 },
  ],
  get_streak: () => 5,
}

export async function invoke(command: string, _args?: unknown): Promise<unknown> {
  const handler = _dispatch[command]
  if (!handler) {
    console.warn(`[browser-mock] Unhandled command: ${command}`)
    return null
  }
  return handler()
}

export class Channel<T = unknown> {
  onmessage: ((response: T) => void) | null = null
}
