export function share_text(text, url) {
  if (navigator.share) { navigator.share({ title: 'Code-Iot', text: text, url: url }).catch(() => {}); return 1; }
  if (navigator.clipboard) { navigator.clipboard.writeText(text + ' ' + url); return 2; }
  return 0;
}