
export function in_app() {
  return !!(window.__TAURI__ || window.__TAURI_INTERNALS__);
}
export function open_external(url) {
  try {
    if (window.__TAURI__ && window.__TAURI__.opener) { window.__TAURI__.opener.openUrl(url); return true; }
    if (window.__TAURI_INTERNALS__) { window.__TAURI_INTERNALS__.invoke('plugin:opener|open_url', { url: url }); return true; }
  } catch (e) {}
  window.open(url, '_blank');
  return false;
}
export function native_google(clientId) {
  var inv = (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke)
    || (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke);
  if (!inv) { return Promise.reject('unavailable'); }
  return inv('plugin:google-signin|sign_in', { clientId: clientId }).then(function (r) { return r.idToken; });
}
export function go_to(url) { window.location.href = url; }
export function sleep(ms) { return new Promise(function (r) { setTimeout(r, ms); }); }
export function random_id() {
  var a = new Uint8Array(16); crypto.getRandomValues(a);
  return Array.from(a, function (b) { return ('0' + b.toString(16)).slice(-2); }).join('');
}
