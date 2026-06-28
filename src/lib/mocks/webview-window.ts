// Browser-mode mock for @tauri-apps/api/webviewWindow

export class WebviewWindow {
  constructor(_label: string, _options?: unknown) {}
  async listen(_event: string, _handler: unknown) { return () => {} }
  async once(_event: string, _handler: unknown) { return () => {} }
  async emit(_event: string, _payload?: unknown) {}
  async hide() {}
  async show() {}
  async close() {}
}

export function getCurrentWebviewWindow(): WebviewWindow {
  return new WebviewWindow('mock')
}
