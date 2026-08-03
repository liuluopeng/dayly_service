function isTauri(): boolean {
  return typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__;
}

export function getDefaultApiUrl(): string {
  // Tauri WebView 不走 HTTP origin，必须用绝对地址
  if (isTauri()) {
    return 'http://localhost:23000';
  }
  // 浏览器（vite dev / docker 同源部署）：返回空串 = 相对路径
  // 生产：前端与 API 同源（axum 服务），/api 自动跟随当前端口；
  // 开发：vite server.proxy 将 /api 转发到 localhost:23001
  return '';
}

export function getApiUrl(): string {
  return localStorage.getItem('apiUrl') || getDefaultApiUrl();
}

export function getPortFromUrl(url: string): string {
  const portMatch = url.match(/:([0-9]+)/);
  return portMatch ? portMatch[1] : '';
}
