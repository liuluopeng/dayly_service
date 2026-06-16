import { watch } from 'vue';
import { useAuth } from './useAuth';
import { init_api, set_api_port } from '../types/wasm-typed';
import { getApiUrl, getPortFromUrl } from '../utils/apiUrl';

export function useApiReinit() {
  const { token } = useAuth();

  // 监听token变化，重新初始化以设置新token
  watch(token, (newToken) => {
    if (newToken) {
      const apiUrl = getApiUrl();
      const port = getPortFromUrl(apiUrl);
      init_api(newToken, apiUrl, port);
    }
  });

  return {
    reinitApi: () => {
      if (token.value) {
        const apiUrl = getApiUrl();
        const port = getPortFromUrl(apiUrl);
        init_api(token.value, apiUrl, port);
      }
    },

    setApiUrl: (apiUrl: string) => {
      localStorage.setItem('apiUrl', apiUrl);
      const port = getPortFromUrl(apiUrl);
      set_api_port(port);
    },

    getApiUrl
  };
}
