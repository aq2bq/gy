/* The rail region (d-03ca, 第 2 段; was n-b963): the ten newest writes, drawn
   inside #railBody from the state root read. The search beside it belongs to
   the palette. */
(function () {
  const FLASH = 1400;

  /* The newest sequence drawn while at the head; null until the first draw. */
  let head = null;

  function render(state, el, ui) {
    if (!el) return;
    /* Drawn only where app.css gives the column; a hidden rail holds no rows. */
    if (!state.wide) {
      el.innerHTML = '';
      return;
    }
    const data = state.rail;
    if (!data) {
      el.innerHTML = '';
      return;
    }
    const rows = data.rows || [];
    const labels = data.labels || {};
    const order = [];
    /* A write is a surprise only while the head is in view (n-b963). */
    const fresh = state.at === null && head !== null;
    let flashed = false;
    const body = rows
      .map(entry => {
        const mark = fresh && entry.seq > head;
        flashed = flashed || mark;
        return `<div class="ri${mark ? ' new' : ''}">${window.GyWrites.row(entry, labels, order, ui, state.lang)}</div>`;
      })
      .join('');
    el.innerHTML =
      `<div class="rail-h"><span class="live${state.live ? '' : ' off'}"></span>${ui.t('railTitle')}</div>` +
      (rows.length ? `<div class="hist">${body}</div>` : `<div class="empty">${ui.t('histEmpty')}</div>`) +
      `<a class="rail-all" href="#/history">${ui.t('railAll')}</a>`;
    if (state.at === null && rows.length) head = rows[0].seq;
    if (flashed) {
      setTimeout(() => el.querySelectorAll('.ri.new').forEach(row => row.classList.remove('new')), FLASH);
    }
  }

  window.GyRail = { render };
})();