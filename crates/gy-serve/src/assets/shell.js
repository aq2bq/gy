/* The sidebar: it draws what /api/shell answers (n-04ea). No framework, only
   fetch and the DOM. The words come from /assets/i18n.json. */
const LANG_KEY = 'gy-lang';
const SCOPE_KEY = 'gy-scope';
const KINDS = ['Need', 'Question', 'Decision', 'Requirement', 'Criterion'];
const PLURAL = { Need: 'Needs', Question: 'Questions', Decision: 'Decisions', Requirement: 'Requirements', Criterion: 'Criteria' };
const ENTRIES = [['#/', 'now', '◉'], ['#/graph', 'graph', '✦'], ['#/history', 'history', '≡']];

let words = null;
let shell = { seq: 0, at: '', scopes: [], kinds: [] };
let scope = sessionStorage.getItem(SCOPE_KEY) || 'all';

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

function drawNav() {
  const hash = location.hash || '#/';
  const entry = ([href, key, mark]) => `<a href="${href}" class="${hash === href ? 'on' : ''}">${mark} ${word(key)}</a>`;
  const kind = name => `<button data-kind="${name}"><span class="dot dot-${name}"></span>${word(PLURAL[name])}${counter(name)}</button>`;
  const scopeRow = item =>
    `<button data-s="${esc(item.name)}" class="${scope === item.name ? 'on' : ''}">${esc(item.name)}<span class="cnt">${item.count}</span></button>`;
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
  if (hash === '' || hash === '#/') {
    crumb.innerHTML = `<span>${word('now')}</span>`;
    if (window.GyNow) window.GyNow.draw();
  } else if (list) {
    crumb.innerHTML = `<a href="#/">${word('now')}</a><span>›</span><span>${plural(list[1])}</span>`;
    if (window.GyList) window.GyList.draw(list[1]);
  } else if (node) {
    if (window.GyNode) window.GyNode.draw(decodeURIComponent(node[1]));
  } else {
    crumb.innerHTML = `<span>${word('now')}</span>`;
    document.getElementById('main').textContent = word('notYet');
  }
}

function draw() {
  document.getElementById('crumb').textContent = word('now');
  document.getElementById('searchlbl').textContent = word('searchLbl');
  document.getElementById('clock').innerHTML =
    `<span class="live"></span>${word('canon')} <b>${shell.seq}</b><br>${stamp(shell.at)}`;
  document.querySelectorAll('#lang button').forEach(button => {
    button.classList.toggle('on', button.dataset.l === lang);
  });
  drawNav();
  route();
}

async function refresh() {
  const res = await fetch(`/api/shell?scope=${encodeURIComponent(scope)}`);
  shell = await res.json();
  draw();
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

/* What a page script (now.js, list.js, node.js, palette.js) needs. */
window.GyShell = {
  t: word,
  lang: () => lang,
  scope: () => scope,
  plural: kind => word(PLURAL[kind] || kind),
};

async function start() {
  const res = await fetch('/assets/i18n.json');
  words = await res.json();
  document.documentElement.lang = lang;
  wire();
  await refresh();
}

start();
