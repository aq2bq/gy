/* The search palette (n-5ca6): /, ⌘K, Ctrl+K, or the top bar opens it. It
   asks /api/search and opens the page of the chosen hit. */
(function () {
  const PANEL = document.getElementById('pal');
  const QUERY = document.getElementById('palq');
  const RESULT = document.getElementById('palres');
  const HINT = document.getElementById('palhint');

  let hits = [];
  let selected = 0;

  const t = key => window.GyShell.t(key);
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));

  function open() {
    PANEL.classList.add('on');
    QUERY.value = '';
    QUERY.placeholder = t('palPh');
    HINT.textContent = t('palHint');
    QUERY.focus();
    search('');
  }

  function close() {
    PANEL.classList.remove('on');
  }

  async function search(text) {
    const query = text.trim();
    if (!query) {
      hits = [];
      selected = 0;
      draw();
      return;
    }
    const res = await window.GyShell.read(`/api/search?q=${encodeURIComponent(query)}`);
    const body = await res.json();
    hits = body.hits || [];
    selected = 0;
    draw();
  }

  function draw() {
    if (!hits.length) {
      RESULT.innerHTML = QUERY.value.trim()
        ? `<div class="r"><span class="t" style="color:var(--paper-3)">${t('notFound')}</span></div>`
        : '';
      return;
    }
    RESULT.innerHTML = hits
      .map((hit, index) => `<div class="r ${index === selected ? 'sel' : ''}" data-i="${index}"><span class="dot dot-${hit.kind}"></span><span class="id">${esc(hit.alias || hit.id)}</span><span class="t">${esc(hit.title)}</span><span class="sc">${esc(hit.scope)}</span></div>`)
      .join('');
  }

  function go(hit) {
    if (!hit) return;
    close();
    location.hash = `#/n/${hit.id}`;
  }

  QUERY.addEventListener('input', event => search(event.target.value));
  QUERY.addEventListener('keydown', event => {
    if (event.key === 'ArrowDown') {
      selected = Math.min(hits.length - 1, selected + 1);
      draw();
      event.preventDefault();
    }
    if (event.key === 'ArrowUp') {
      selected = Math.max(0, selected - 1);
      draw();
      event.preventDefault();
    }
    if (event.key === 'Enter') go(hits[selected]);
    if (event.key === 'Escape') close();
  });
  RESULT.addEventListener('click', event => {
    const row = event.target.closest('.r');
    if (row && hits[row.dataset.i]) go(hits[row.dataset.i]);
  });
  PANEL.addEventListener('click', event => {
    if (event.target === PANEL) close();
  });
  document.getElementById('searchbtn').addEventListener('click', open);
  document.addEventListener('keydown', event => {
    const typing = event.target.tagName === 'INPUT';
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      open();
    }
    if (event.key === '/' && !typing) {
      event.preventDefault();
      open();
    }
  });
})();
