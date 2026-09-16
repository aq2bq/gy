/* The rail (n-b963): the ten newest writes, so a write by an agent shows up
   the moment it lands. It reads /api/history?limit=10 and draws each row with
   GyWrites; the shell redraws it after every reload, so the live poll keeps it
   moving. On a narrow screen app.css hides it, and the search lives in the top
   bar instead (n-ccd2). */
(function () {
  const LIMIT = 10;
  const FLASH = 1400;
  /* The rail is shown only where app.css gives it a column; it draws nowhere
     else, so a hidden rail holds no rows. The 1560px is app.css's breakpoint,
     and must stay equal to it (change both files together). */
  const WIDE = window.matchMedia('(min-width: 1560px)');

  /* The newest sequence drawn while at the head; null until the first draw. */
  let head = null;

  const t = key => window.GyShell.t(key);

  function historyPath() {
    const scope = window.GyShell.scope();
    const only = scope && scope !== 'all' ? `&scope=${encodeURIComponent(scope)}` : '';
    return `/api/history?limit=${LIMIT}${only}`;
  }

  async function labels(ids) {
    const unique = [...new Set(ids.filter(Boolean))];
    if (!unique.length) return {};
    const res = await window.GyShell.read(`/api/labels?ids=${unique.join(',')}`);
    return (await res.json()).labels;
  }

  /* One row, wrapped so the `new` mark sits beside GyWrites' own class. */
  function item(entry, known, order, mark) {
    return `<div class="ri${mark ? ' new' : ''}">${window.GyWrites.row(entry, known, order)}</div>`;
  }

  /* The search is one element: it sits in the rail's own holder at this width,
     and in the top bar otherwise. It is moved, never copied, and the holder is
     never rewritten, so the panel's redraw cannot take it (n-ccd2, n-1d12). */
  function placeSearch(where) {
    const search = document.getElementById('searchbtn');
    if (!search) return;
    const holder = where === 'rail' ? document.getElementById('railSearch') : document.querySelector('.topbar');
    if (!holder || search.parentElement === holder) return;
    holder.appendChild(search);
  }

  async function draw() {
    /* The rail rewrites only its body: the search holder beside it stays. */
    const box = document.getElementById('railBody');
    if (!box) return;
    if (!WIDE.matches) {
      placeSearch('bar');
      box.innerHTML = '';
      return;
    }
    const at = window.GyShell.at;
    const data = await (await window.GyShell.read(historyPath())).json();
    const known = await labels(data.rows.map(entry => entry.node));
    /* A write is a surprise only while the head is in view (n-b963). */
    const fresh = at === null && head !== null;
    const order = [];
    let flashed = false;
    const rows = data.rows
      .map(entry => {
        const mark = fresh && entry.seq > head;
        flashed = flashed || mark;
        return item(entry, known, order, mark);
      })
      .join('');
    const body = data.rows.length
      ? `<div class="hist">${rows}</div>`
      : `<div class="empty">${t('histEmpty')}</div>`;
    box.innerHTML =
      `<div class="rail-h"><span class="live"></span>${t('railTitle')}</div>` +
      body +
      `<a class="rail-all" href="#/history">${t('railAll')}</a>`;
    placeSearch('rail');
    if (at === null && data.rows.length) head = data.rows[0].seq;
    if (flashed) {
      setTimeout(() => box.querySelectorAll('.ri.new').forEach(row => row.classList.remove('new')), FLASH);
    }
  }

  window.GyRail = { draw };
  WIDE.addEventListener('change', () => draw());
})();