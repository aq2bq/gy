/* The top bar region (d-03ca, 第 2 段): the breadcrumb only. The node page
   writes its own crumb until 第 3 段, so this leaves the node route alone. */
(function () {
  const EYE_WORD = { wait: 'eyeWait', next: 'readyHead', resume: 'resumeHead' };

  function render(state, el, ui) {
    if (!el) return;
    const route = state.route;
    const now = `<a href="#/">${ui.t('now')}</a><span>›</span>`;
    if (route.name === 'now' || route.name === 'missing') el.innerHTML = `<span>${ui.t('now')}</span>`;
    else if (route.name === 'list') el.innerHTML = `${now}<span>${ui.plural(route.arg)}</span>`;
    else if (route.name === 'graph') el.innerHTML = `${now}<span>${ui.t('graph')}</span>`;
    else if (route.name === 'eye') el.innerHTML = `${now}<span>${ui.t(EYE_WORD[route.arg])}</span>`;
    else if (route.name === 'history') el.innerHTML = `${now}<span>${ui.t('history')}</span>`;
    /* node: the page writes it (unchanged until 第 3 段). */
  }

  window.GyTopbar = { render };
})();