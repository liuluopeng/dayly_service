function isTauri(): boolean {
  return typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;
}

export function getDefaultApiUrl(): string {
  // Tauri WebView 不走 HTTP origin，必须用绝对地址
  if (isTauri()) {
    return 'http://localhost:23000';
  }
  // 浏览器：返回同源绝对地址（页面 origin）
  // - 生产（docker）：页面 http://host:23000 → API http://host:23000，端口自动跟随
  // - 开发（vite 1420）：请求发到 1420，由 server.proxy 转发到 23001
  // 注意：wasm reqwest 不接受相对 URL（builder error），必须绝对地址
  return `${window.location.protocol}//${window.location.hostname}:${window.location.port}`;
}

export function getApiUrl(): string {
  return localStorage.getItem('apiUrl') || getDefaultApiUrl();
}

export function getPortFromUrl(url: string): string {
  const portMatch = url.match(/:([0-9]+)/);
  return portMatch ? portMatch[1] : '';
}
