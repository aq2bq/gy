/* copy.js (part, n-6afd): the small copy mark beside a name. It shows the text
   it is handed and copies that same text, so what is written is what is seen.
   The click is events.js's; the sign lives in state.copied and the root's
   timer clears it. No clipboard, no mark (a mark that does nothing is worse
   than none). */
(function () {
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* `text` is both what the eye reads and what the clipboard gets. */
  function tag(text, state, ui) {
    const shown = esc(text);
    const done = state.copied === text;
    const mark = ui.canCopy
      ? `<button type="button" class="cpb${done ? ' on' : ''}" data-act="copy" data-arg="${shown}" aria-label="${esc(fill(ui.t('copyText'), { text }))}">${done ? '✓' : '⧉'}</button>`
      : '';
    return `<span class="cp"><span class="id">${shown}</span>${mark}</span>`;
  }

  window.GyCopy = { tag };
})();