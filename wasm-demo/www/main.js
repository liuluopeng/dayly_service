import init, { init_api, login, search_wubi, set_token } from './pkg/wasm_demo.js';

const $ = s => document.querySelector(s);
const $$ = s => document.querySelectorAll(s);

async function main() {
  await init();

  // 登录
  $('#login-btn').onclick = async () => {
    const user = $('#login-user').value.trim();
    const pass = $('#login-pass').value.trim();
    if (!user || !pass) { $('#login-msg').textContent = '请输入用户名和密码'; return; }
    $('#login-btn').disabled = true;
    $('#login-msg').textContent = '登录中...';
    try {
      const res = await login(user, pass);
      set_token(res.token);
      $('#login-msg').textContent = '';
      $('#page-login').classList.add('hidden');
      $('#page-app').classList.remove('hidden');
    } catch (e) {
      $('#login-msg').textContent = '登录失败: ' + e;
    } finally {
      $('#login-btn').disabled = false;
    }
  };

  // 退出
  $('#logout-btn').onclick = () => {
    set_token('');
    $('#page-app').classList.add('hidden');
    $('#page-login').classList.remove('hidden');
    $('#login-pass').value = '';
  };

  // 五笔查询
  $('#wubi-btn').onclick = async () => {
    const code = $('#wubi-input').value.trim();
    if (!code) return;
    $('#wubi-btn').disabled = true;
    $('#wubi-result').textContent = '查询中...';
    try {
      const res = await search_wubi(code);
      let html = `<div class="code">${res.char || '-'} → ${res.code86 || '-'}</div>`;
      if (res.has_diagram) {
        html += '<div class="detail">含字根图</div>';
      }
      $('#wubi-result').innerHTML = html;
    } catch (e) {
      $('#wubi-result').textContent = '查询失败: ' + e;
    } finally {
      $('#wubi-btn').disabled = false;
    }
  };

  // 回车键触发
  $('#login-pass').onkeydown = e => { if (e.key === 'Enter') $('#login-btn').click(); };
  $('#wubi-input').onkeydown = e => { if (e.key === 'Enter') $('#wubi-btn').click(); };
}

main();
