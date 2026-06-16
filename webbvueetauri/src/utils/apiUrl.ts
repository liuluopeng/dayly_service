export function getDefaultApiUrl(): string {
  if (import.meta.env.DEV) {
    return `http://${window.location.hostname}:23001`;
  }
  return `http://${window.location.hostname}:23000`;
}

export function getApiUrl(): string {
  return localStorage.getItem('apiUrl') || getDefaultApiUrl();
}

export function getPortFromUrl(url: string): string {
  const portMatch = url.match(/:([0-9]+)/);
  return portMatch ? portMatch[1] : '23001';
}
