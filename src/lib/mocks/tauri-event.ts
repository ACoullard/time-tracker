// Browser-mode mock for @tauri-apps/api/event
// listen/once return a no-op unlisten; emit is a no-op.

export type EventCallback<T> = (event: { payload: T; id: number; event: string }) => void

export async function listen<T>(
  _event: string,
  _handler: EventCallback<T>,
): Promise<() => void> {
  return () => {}
}

export async function once<T>(
  _event: string,
  _handler: EventCallback<T>,
): Promise<() => void> {
  return () => {}
}

export async function emit(_event: string, _payload?: unknown): Promise<void> {}
