
export function ask(message) { return window.confirm(message); }
export function copy_text(text) { try { navigator.clipboard.writeText(text); } catch (e) {} }
export function download_text(name, text) {
  var blob = new Blob(['﻿' + text], { type: 'text/csv;charset=utf-8' });
  var a = document.createElement('a');
  a.href = URL.createObjectURL(blob); a.download = name;
  document.body.appendChild(a); a.click(); a.remove();
  setTimeout(function () { URL.revokeObjectURL(a.href); }, 2000);
}
