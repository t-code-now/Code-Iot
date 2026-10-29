// Serves a built web/dist and a tiny fake /api for screenshots.
const http = require('http'), fs = require('fs'), path = require('path');
const DIST = process.argv[2], PORT = +process.argv[3] || 4100;
const he = u => /lang=he/.test(u);
const course = (u) => ({ id: 'python', title: 'Python', language: 'python', description: he(u) ? 'שפה ידידותית למתחילים' : 'A friendly first language', color: '#3b82f6', icon: '🐍', lesson_count: 104, completed_count: 3, unlocked: false });
const lesson = (u) => ({ id: 'py-1-2', title: he(u) ? 'משתנים' : 'Variables', course_id: 'python', course_title: 'Python', color: '#3b82f6', next_lesson_id: 'py-1-3', steps: [
  { kind: 'learn', title: he(u) ? 'סדר פעולות' : 'Order of operations', body: he(u) ? 'כפל מתבצע לפני חיבור.' : 'Multiplication happens before addition.', code: 'print(2 + 3 * 4)' },
  { kind: 'multiple_choice', prompt: he(u) ? 'מה זה מציג?' : 'What does this print?', code: 'print(2 + 3 * 4)', options: ['20', '14', '24', '9'] },
  { kind: 'multiple_choice', prompt: he(u) ? 'ומה זה?' : 'And this?', code: 'print((2 + 3) * 4)', options: ['20', '14'] },
]});
const detail = (u) => ({ ...course(u), viewer: 'guest', sections: [1, 2].map(s => ({ id: 'py-' + s, title: (he(u) ? 'נושא ' : 'Topic ') + s, lessons: [1, 2, 3, 4, 5].map(l => ({ id: `py-${s}-${l}`, title: (he(u) ? 'שיעור ' : 'Lesson ') + l, exercise_count: 4, completed: s == 1 && l < 3, best_correct: 4, locked: s == 2 })) })) });
const json = (res, code, body) => { res.writeHead(code, { 'content-type': 'application/json' }); res.end(JSON.stringify(body)); };
http.createServer((req, res) => {
  const u = req.url;
  let body = ''; req.on('data', d => body += d); req.on('end', () => {
    if (u.startsWith('/api/')) {
      if (u.startsWith('/api/courses/')) return json(res, 200, detail(u));
      if (u.startsWith('/api/courses')) return json(res, 200, [course(u)]);
      if (/\/api\/lessons\/[^/]+\/check/.test(u)) { const b = JSON.parse(body || '{}'); const ok = b.step === 1 ? b.answer.value === 1 : b.answer.value === 0;
        return json(res, 200, { correct: ok, expected: ok ? '' : (b.step === 1 ? '14' : '20'), explanation: he(u) ? 'כפל מתבצע לפני חיבור: 3 * 4 = 12, ועוד 2 זה 14.' : '3 * 4 = 12, plus 2 is 14.' }); }
      if (u.startsWith('/api/lessons/')) return json(res, 200, lesson(u));
      if (u.startsWith('/api/auth/providers')) return json(res, 200, { google: true, password: true, google_client_id: 'x' });
      if (u.startsWith('/api/payments/config')) return json(res, 200, { enabled: true, sales_paused: false });
      if (u.startsWith('/api/leaderboard')) return json(res, 200, { scope: 'weekly', entries: [], me: null });
      if (req.headers.authorization && (u.startsWith('/api/me') && !u.startsWith('/api/me/'))) return json(res, 200, { id: 1, username: 'jean walters', xp: 41, weekly_xp: 41, level: 1, level_xp: 41, level_xp_needed: 100, streak: 1, longest_streak: 1, lessons_completed: 3, courses_completed: 0, badges: [], joined_at: 1761700000, avatar: null, email: 'jeanwallters@gmail.com', is_admin: false, plans: [], phone: null, country: null });
      if (req.headers.authorization && u.startsWith('/api/me/')) return json(res, 200, []);
      return json(res, 401, { error: 'not signed in' });
    }
    let f = path.join(DIST, decodeURIComponent(u.split('?')[0]));
    if (!f.startsWith(DIST) || !fs.existsSync(f) || fs.statSync(f).isDirectory()) f = path.join(DIST, 'index.html');
    const ext = path.extname(f);
    const types = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.png': 'image/png', '.svg': 'image/svg+xml', '.webp': 'image/webp', '.jpg': 'image/jpeg', '.woff2': 'font/woff2', '.json': 'application/json' };
    res.writeHead(200, { 'content-type': types[ext] || 'application/octet-stream' }); fs.createReadStream(f).pipe(res);
  });
}).listen(PORT, () => console.log('on', PORT));
