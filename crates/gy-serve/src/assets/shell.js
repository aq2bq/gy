/* The sidebar: it draws what /api/shell answers (n-04ea). No framework, only
   fetch and the DOM. The words come from /assets/i18n.json. */
const LANG_KEY = 'gy-lang';
const SCOPE_KEY = 'gy-scope';
const KINDS = ['Need', 'Question', 'Decision', 'Requirement', 'Criterion'];
const PLURAL = { Need: 'Needs', Question: 'Questions', Decision: 'Decisions', Requirement: 'Requirements', Criterion: 'Criteria' };
const ENTRIES = [['#/', 'now', '◉'], ['#/graph', 'graph', '✦'], ['#/history', 'history', '≡']];
/* The three sidebar boxes and the words their pages wear (n-c122). */
const EYE_WORD = { wait: 'eyeWait', next: 'readyHead', resume: 'resumeHead' };

let words = null;
let shell = { seq: 0, at: '', scopes: [], kinds: [] };
let scope = sessionStorage.getItem(SCOPE_KEY) || 'all';
/* The point in the log every read asks for; null is the head (n-32a9). */
let at = null;
/* The three numbers the sidebar keeps from the now view (d-3e8f). */
let eyes = { wait: 0, next: 0, resume: 0 };

/* ?lang= wins, then the stored choice, then what the server sent (ac-b9b1). */
function pickLang() {
  const asked = new URLSearchParams(location.search).get('lang');
  if (asked === 'en' || asked === 'ja') return asked;
  return localStorage.getItem(LANG_KEY) || (document.documentElement.lang === 'ja' ? 'ja' : 'en');
}

let lang = pickLang();

/* A word by key; a dotted key walks nested objects (`st.Need.open`). */
function word(key) {
  if (!words) return key;
  const found = key.split('.').reduce((value, part) => (value ? value[part] : undefined), words[lang]);
  return found ?? key;
}

/* The plural name of a kind, for a list heading. */
const plural = kind => word(PLURAL[kind] || kind);

const esc = text => String(text).replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));

/* The one place a scope name becomes a badge (n-d29f): the hue comes from a
   short hash of the name, so the same scope wears the same colour on every
   page and nothing has to be configured or stored. The look is app.css's .sb,
   a filled pill beside the outline-only kind badge. */
const SCOPE_HUES = [10, 40, 95, 140, 175, 210, 262, 320];
function scopeTag(name) {
  const text = String(name ?? '');
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index++) {
    hash = Math.imul(hash ^ text.charCodeAt(index), 0x01000193);
  }
  const hue = SCOPE_HUES[(hash >>> 0) % SCOPE_HUES.length];
  return `<span class="sb" style="--sb:${hue}">${esc(text)}</span>`;
}

/* An IME's own key, not the app's (n-87ab): the composition flag or keyCode
   229. Some IMEs report the confirming Enter with neither, so a short guard
   after the composition ends is kept too. */
let composedAt = 0;
document.addEventListener('compositionend', () => {
  composedAt = performance.now();
});
const composing = event => event.isComposing || event.keyCode === 229;
const composedRecently = () => performance.now() - composedAt < 50;

/* A time the way the browser writes it in this language. */
function stamp(at) {
  return new Date(at).toLocaleString(lang === 'ja' ? 'ja-JP' : 'en-GB', {
    month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
  });
}

/* One kind's counter: a decision has no open, so a total only. */
function counter(kind) {
  const row = shell.kinds.find(item => item.kind === kind);
  if (!row) return '<span class="cnt"></span>';
  return kind === 'Decision'
    ? `<span class="cnt"><b>${row.total}</b></span>`
    : `<span class="cnt"><b>${row.open}</b> / ${row.total}</span>`;
}

/* The three counts under the wordmark. The numbers come from the now page (or
   one /api/now read); the words from the dictionary. */
function drawEyes() {
  const box = document.getElementById('eyes3');
  if (!box) return;
  const item = (key, n, name) => `<a href="#/eye/${key}" class="${n > 0 && key === 'wait' ? 'hot' : ''}"><b>${n}</b><small>${name}</small></a>`;
  box.innerHTML =
    item('wait', eyes.wait, word('eyeWait')) +
    item('next', eyes.next, word('eyeNext')) +
    item('resume', eyes.resume, word('eyeResume'));
}

function drawNav() {
  const hash = location.hash || '#/';
  const entry = ([href, key, mark]) => `<a href="${href}" class="${hash === href ? 'on' : ''}">${mark} ${word(key)}</a>`;
  const kind = name => `<a href="#/list/${name}" class="${hash.startsWith(`#/list/${name}`) ? 'on' : ''}"><span class="dot dot-${name}"></span>${word(PLURAL[name])}${counter(name)}</a>`;
  const scopeRow = item =>
    `<button data-s="${esc(item.name)}" class="${scope === item.name ? 'on' : ''}">${scopeTag(item.name)}<span class="cnt">${item.count}</span></button>`;
  const nodes = shell.scopes.reduce((sum, item) => sum + item.count, 0);
  document.getElementById('nav').innerHTML =
    ENTRIES.map(entry).join('') +
    `<div class="h">${word('nodes')}</div>` +
    KINDS.map(kind).join('') +
    `<div class="h">${word('scopes')}</div>` +
    `<button data-s="all" class="${scope === 'all' ? 'on' : ''}">${word('all')}<span class="cnt">${nodes}</span></button>` +
    shell.scopes.map(scopeRow).join('');
}

/* The router: the empty hash is the now page, #/list and #/n are the two read
   pages, and every other page waits its own need. */
function route() {
  const hash = location.hash;
  const crumb = document.getElementById('crumb');
  const list = hash.match(/^#\/list\/(\w+)$/);
  const node = hash.match(/^#\/n\/(.+)$/);
  const graph = hash.match(/^#\/graph(?:\/(.+))?$/);
  const eye = hash.match(/^#\/eye\/(wait|next|resume)$/);
  document.getElementById('main').className = '';
  if (hash === '' || hash === '#/') {
    crumb.innerHTML = `<span>${word('now')}</span>`;
    if (window.GyNow) window.GyNow.draw();
  } else if (list) {
    crumb.innerHTML = `<a href="#/">${word('now')}</a><span>›</span><span>${plural(list[1])}</span>`;
    if (window.GyList) window.GyList.draw(list[1]);
  } else if (graph) {
    crumb.innerHTML = `<a href="#/">${word('now')}</a><span>›</span><span>${word('graph')}</span>`;
    if (window.GyGraph) window.GyGraph.draw(graph[1] ? decodeURIComponent(graph[1]) : null);
  } else if (eye) {
    crumb.innerHTML = `<a href="#/">${word('now')}</a><span>›</span><span>${word(EYE_WORD[eye[1]])}</span>`;
    if (window.GyEye) window.GyEye.draw(eye[1]);
  } else if (node) {
    if (window.GyNode) window.GyNode.draw(decodeURIComponent(node[1]));
  } else if (hash === '#/history') {
    crumb.innerHTML = `<a href="#/">${word('now')}</a><span>›</span><span>${word('history')}</span>`;
    if (window.GyHistory) window.GyHistory.draw();
  } else {
    crumb.innerHTML = `<span>${word('now')}</span>`;
    document.getElementById('main').textContent = word('notYet');
  }
}

function draw() {
  document.getElementById('crumb').textContent = word('now');
  document.getElementById('searchlbl').textContent = word('searchLbl');
  /* While a past point is chosen the band owns the clock (n-32a9). */
  if (at === null) {
    /* A ledger with no writes has no time to show: the epoch would read as a
       fault, not as a first run (n-3e6b). */
    const when = shell.seq > 0 ? `<br>${stamp(shell.at)}` : '';
    document.getElementById('clock').innerHTML =
      `<span class="live"></span>${word('canon')} <b>${shell.seq}</b>${when}`;
  }
  document.querySelectorAll('#lang button').forEach(button => {
    button.classList.toggle('on', button.dataset.l === lang);
  });
  drawNav();
  drawEyes();
  if (window.GyTime) window.GyTime.labels();
  route();
  /* The rail follows every redraw: live updates, language, scope, rewind. */
  if (window.GyRail) window.GyRail.draw();
}

/* The one place a read goes out: it carries the point in the log when one is
   chosen, so every page answers for that moment (n-32a9). */
function read(path) {
  if (at === null) return fetch(path);
  const join = path.includes('?') ? '&' : '?';
  return fetch(`${path}${join}at=${at}`);
}

async function refresh() {
  const res = await read(`/api/shell?scope=${encodeURIComponent(scope)}`);
  shell = await res.json();
  draw();
  loadEyes();
}

/* The three counts, from one read of the now view. A failure keeps them. */
function setEyes(wait, next, resume) {
  eyes = { wait, next, resume };
  drawEyes();
}

async function loadEyes() {
  try {
    const only = scope && scope !== 'all' ? `?scope=${encodeURIComponent(scope)}` : '';
    const res = await read(`/api/now${only}`);
    const view = await res.json();
    setEyes(view.waiting.length, view.ready.length, view.resume.in_progress.length);
  } catch {
    /* Nothing to update. */
  }
}

function wire() {
  document.getElementById('lang').addEventListener('click', event => {
    const button = event.target.closest('button');
    if (!button) return;
    lang = button.dataset.l;
    localStorage.setItem(LANG_KEY, lang);
    document.documentElement.lang = lang;
    draw();
  });
  document.getElementById('nav').addEventListener('click', event => {
    const button = event.target.closest('button[data-s]');
    if (!button) return;
    scope = button.dataset.s;
    sessionStorage.setItem(SCOPE_KEY, scope);
    refresh();
  });
  window.addEventListener('hashchange', () => {
    drawNav();
    route();
  });
}

/* What a page script (now.js, list.js, node.js, palette.js, time.js) needs. */
window.GyShell = {
  t: word,
  lang: () => lang,
  scope: () => scope,
  plural: kind => word(PLURAL[kind] || kind),
  /* A scope name as the one badge (n-d29f); every page draws it through this. */
  scopeTag,
  /* The point in the log, or null for the head. */
  get at() {
    return at;
  },
  set at(seq) {
    at = seq;
  },
  /* A read with the point attached; every page fetches through this. */
  read,
  /* Whether a keydown belongs to an IME, and whether one just ended (n-87ab). */
  composing,
  composedRecently,
  /* The three sidebar counts; the now page passes them after it draws. */
  setEyes,
  /* Fetch the shell again and redraw what is on screen. */
  reload: refresh,
};

async function start() {
  const res = await fetch('/assets/i18n.json');
  words = await res.json();
  document.documentElement.lang = lang;
  wire();
  await refresh();
}

start();
