/* The detail screen (d-e239, d-647f): one node in the middle, where it came
   from on the left, and what it works on on the right. The thread comes from
   /api/node/<id>/path; the node's own facts, body, and writes come from
   /api/node/<id>, so a thread that does not answer still leaves those visible
   (ac-0dbb, r-2d02). */
(function () {
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));
  const DAY = 86400;

  function render(state, el, ui) {
    if (!el) return;
    const path = state.path ? window.GyShape.path(state.path) : null;
    const center = path && path.center;
    /* The thread's answer carries the center; the page's answer carries the
       facts, the body, and the writes. A slow page must not hide the thread
       (ac-0dbb): the center stands in until the page comes. */
    const node = state.page || (center ? { ...center, data: {}, edges: [], history: [] } : null);
    if (!node) {
      el.innerHTML = `<div class="sub">${esc(state.route.arg || '')}</div>` +
        `<h1 class="hd">${esc(state.route.arg || '')}</h1>` + status(state, 'page', !!state.page, ui);
      return;
    }
    const moved = el.querySelector('[data-shown]');
    const kind = window.GyShape.kind(node.kind, node.id);
    const mark = window.GyShape.state(node.state || condition(node, kind));
    const scope = node.scope || '';
    const meta = state.page ? pageMeta(node, kind, mark, state, ui) : [`<span>${ui.scopeTag(scope)}</span>`,
      `<span class="stbadge ${window.GyShape.isOpen(mark) ? 'open' : ''}">${esc(ui.t(`st.${kind}.${mark}`))}</span>`].join('');
    el.innerHTML = `<div data-shown="${esc(node.id)}">` +
      `<a class="back" href="#/cross">${esc(ui.t('backCross'))}</a>` +
      `<div class="nhead"><div class="meta"><span class="k k-${kind}">${esc(ui.t(kind))}</span>${state.page ? '' : `<span class="al">${esc(node.id)}</span>`}${meta}</div><h1>${esc(node.title)}</h1></div>` +
      thread(state, node, path, ui) +
      `<div class="ngrid"><div>${state.page ? facts(node, kind, ui, state.lang) + body(node, ui) : `<div class="sec">${status(state, 'page', false, ui)}</div>`}</div>` +
      `<div>${state.page ? crossOut(node, ui) + history(node, ui, state.lang) : ''}</div></div></div>`;
    if (!moved || moved.dataset.shown !== node.id) window.scrollTo(0, 0);
  }

  function pageMeta(node, kind, mark, state, ui) {
    return `<span class="al">${esc((node.aliases || [])[0] || node.id)}</span>` +
      (ui.canCopy ? window.GyCopy.tag(node.id, state, ui) : `<span class="id">${esc(node.id)}</span>`) +
      `<span>${ui.scopeTag(node.scope || '')}</span>` +
      `<span data-testid="created">${esc(fill(ui.t('createdOn'), { d: window.GyTime.date(node.created) }))}</span>` +
      `<span class="stbadge ${window.GyShape.isOpen(mark) ? 'open' : ''}">${esc(ui.t(`st.${kind}.${mark}`))}</span>`;
  }

  /* The state word from the typed data: the page carries no derived state for
     a decision, so a superseding edge names it. */
  function condition(node, kind) {
    const data = node.data || {};
    if (kind === 'Need') return (data.Need && data.Need.closed) ? 'closed' : 'open';
    if (kind === 'Question') return (data.Question && data.Question.closure) ? 'closed' : 'open';
    if (kind === 'Criterion') {
      if ((node.withdrawn_by || []).length) return 'withdrawn';
      return data.Criterion && data.Criterion.satisfied ? 'satisfied' : 'unsatisfied';
    }
    if (kind === 'Requirement') return String((data.Requirement && data.Requirement.state) || 'filed').toLowerCase();
    const superseded = (node.edges || []).some(edge => edge.name === 'superseded-by');
    return superseded ? 'superseded' : 'live';
  }

  /* ---- the thread ---- */
  const BW = 236;
  const BH = 46;
  const GX = 54;
  const GY = 8;
  const TOP = 28;
  const short = text => (String(text || '').length > 22 ? `${String(text).slice(0, 21)}…` : String(text || ''));

  function thread(state, node, path, ui) {
    const head = `<div class="sec"><h2>${esc(ui.t('detailThread'))} <span class="sub">${esc(ui.t('detailThreadSub'))}</span>` +
      `<span class="seg" style="margin-left:auto">${[2, 3].map(depth => `<button data-testid="depth" data-act="detailDepth" data-arg="${depth}" class="${state.detail.depth === depth ? 'on' : ''}">${esc(ui.t(`depth${depth}`))}</button>`).join('')}</span></h2>`;
    if (!path) {
      return head + `<div class="thread-frame">${status(state, 'path', !!state.path, ui)}</div></div>`;
    }
    return head + `<div class="thread"><div class="thread-scroll">${figure(path, node, state, ui)}</div></div></div>`;
  }

  /* Columns: the farthest step left first, the node, then the steps right. */
  function columns(path, node) {
    const left = path.left.slice().reverse();
    const cols = left.map((stage, index) => ({ side: 'up', depth: left.length - index, stage }));
    cols.push({ side: 'center', depth: 0, stage: { nodes: [], more: 0 } });
    path.right.forEach((stage, index) => cols.push({ side: 'down', depth: index + 1, stage }));
    return cols;
  }

  const colHeight = col => Math.max(BH, (col.stage.nodes.length + (col.stage.more ? 1 : 0)) * (BH + GY));

  /* Give every column its x and every box its y, centered on the middle. */
  function placeColumns(cols, node, height) {
    const at = new Map();
    cols.forEach((col, index) => {
      col.x = 20 + index * (BW + GX);
      const h = colHeight(col);
      col.y0 = TOP + (height - TOP - h) / 2;
      if (col.side === 'center') at.set(`center:${node.id}`, { x: col.x, y: col.y0, node, center: true });
      col.stage.nodes.forEach((item, row) => at.set(`${col.side}:${item.id}`, { x: col.x, y: col.y0 + row * (BH + GY), node: item, center: false }));
    });
    return at;
  }

  function stageLabels(cols, kind, ui) {
    return cols.map(col => {
      const label = col.side === 'center'
        ? fill(ui.t('stageCenter'), { kind: esc(ui.t(kind)) })
        : col.side === 'up' ? fill(ui.t('stageUp'), { n: col.depth }) : fill(ui.t('stageDown'), { n: col.depth });
      return `<text class="ch" x="${col.x}" y="16">${esc(label)}</text>`;
    }).join('');
  }

  /* One line from the parent to the child, and the relation beside the child. */
  function edgeLine(child, parent, item, label, ui) {
    const left = child.x < parent.x ? child : parent;
    const right = child.x < parent.x ? parent : child;
    const x1 = left.x + BW;
    const y1 = left.y + BH / 2;
    const x2 = right.x;
    const y2 = right.y + BH / 2;
    const title = item.mark ? `<title>${esc(item.mark)}</title>` : '';
    const path = `<path class="e" data-to="${esc(item.id)}" d="M${x1},${y1} C${x1 + GX / 2},${y1} ${x2 - GX / 2},${y2} ${x2},${y2}">${title}</path>`;
    if (!label || !item.relation) return path;
    /* The relation sits beside the child, on the side the line leaves from. */
    const down = child.x > parent.x;
    const x = down ? child.x + 4 : child.x + BW - 4;
    return path + `<text class="rl" x="${x}" y="${child.y - 5}" text-anchor="${down ? 'start' : 'end'}">${esc(ui.t(`rel.${item.relation}`))}</text>`;
  }

  function edgesSvg(cols, at, centerId, ui) {
    let svg = '';
    for (const col of cols) {
      if (col.side === 'center') continue;
      for (const item of col.stage.nodes) {
        const child = at.get(`${col.side}:${item.id}`);
        const parent = item.parent ? at.get(`${col.side}:${item.parent}`) || at.get(`center:${item.parent}`) : null;
        if (child && parent) svg += edgeLine(child, parent, item, item.parent === centerId, ui);
      }
    }
    return svg;
  }

  function boxInner(item, kind, state, scope, ui) {
    const openState = window.GyShape.isOpen(state);
    const colourName = colour(kind);
    const word = window.GyShape.state(state);
    return `<rect width="${BW}" height="${BH}" rx="7" class="bx-rect ${openState ? 'open' : 'closed'}" stroke="var(--${colourName})"/>` +
      `<circle cx="11" cy="14" r="4" fill="${openState ? `var(--${colourName})` : 'none'}" stroke="var(--${colourName})"/>` +
      `<text class="id" x="21" y="17">${esc(item.id)} · ${esc(ui.t(kind))} · ${esc(ui.t(`st.${kind}.${word}`))}${scope ? ` · ${esc(scope)}` : ''}</text>` +
      `<text x="9" y="36">${esc(short(item.title))}</text>`;
  }

  function boxesSvg(at, node, ui) {
    let svg = '';
    for (const [key, box] of at) {
      if (key.startsWith('center:')) continue;
      const item = box.node;
      const kind = window.GyShape.kind(item.kind, item.id);
      const scope = item.scope && item.scope !== node.scope ? item.scope : '';
      svg += `<a href="#/n/${encodeURIComponent(item.id)}"><g class="bx" data-id="${esc(item.id)}" transform="translate(${box.x},${box.y})" data-testid="threadBox">${boxInner(item, kind, item.state, scope, ui)}</g></a>`;
    }
    return svg;
  }

  function centerSvg(node, kind, center, ui) {
    if (!center) return '';
    return `<g class="bx ctr" data-testid="threadCenter" transform="translate(${center.x},${center.y})">${boxInner(node, kind, node.state || condition(node, kind), '', ui)}</g>`;
  }

  function hiddenSvg(cols, ui) {
    return cols.filter(col => col.stage.more).map(col => {
      const y = col.y0 + col.stage.nodes.length * (BH + GY);
      return `<text class="more" x="${col.x + BW / 2}" y="${y + 20}" text-anchor="middle">${esc(fill(ui.t('hiddenNodes'), { n: col.stage.more }))}</text>`;
    }).join('');
  }

  function figure(path, node, state, ui) {
    const kind = window.GyShape.kind(node.kind, node.id);
    const cols = columns(path, node);
    const height = Math.max(150, Math.max(...cols.map(colHeight)) + TOP + 24);
    const width = cols.length * (BW + GX) - GX + 40;
    const at = placeColumns(cols, node, height);
    return `<svg width="${width}" height="${height}" role="img" aria-label="${esc(ui.t('detailThread'))}">` +
      stageLabels(cols, kind, ui) + edgesSvg(cols, at, node.id, ui) + boxesSvg(at, node, ui) +
      centerSvg(node, kind, at.get(`center:${node.id}`), ui) + hiddenSvg(cols, ui) + `</svg>`;
  }

  const colour = kind => ({ Need: 'need', Question: 'question', Decision: 'decision', Requirement: 'requirement', Criterion: 'criterion' }[kind] || 'paper-2');

  /* ---- the facts, the body, and the writes ---- */
  const kv = (ui, key, value) => (value ? `<div class="k2">${esc(ui.t(key))}</div><div class="v">${value}</div>` : '');

  function needFacts(node, ui) {
    const closed = (node.data.Need || {}).closed;
    const word = closed ? 'closed' : 'open';
    return kv(ui, 'stateWord', esc(ui.t(`st.Need.${word}`))) +
      kv(ui, 'howClosed', closed ? `<b>${esc(closed.by)}</b> ${esc(closed.evidence || '')}` : esc(ui.t('keepGoing')));
  }

  function questionFacts(node, ui) {
    const question = node.data.Question || {};
    const options = question.options || [];
    const list = options.length
      ? `<ul class="opts">${options.map((option, index) => `<li><span class="tag">${String.fromCharCode(65 + index)}</span>${esc(option)}</li>`).join('')}</ul>`
      : '';
    return kv(ui, 'decider', esc(question.decider || '')) + kv(ui, 'options', list) +
      kv(ui, 'closedHow', question.closure ? `<b>${esc(question.closure)}</b> ${esc(question.evidence || '')}` : '');
  }

  /* The decisions that narrowed or replaced a passage of this one (n-0c6f). */
  function retraction(node, ui) {
    const cancelled = node.cancellation;
    if (!cancelled) return '';
    const said = [
      ...(cancelled.superseded_by || []).map(id => `${esc(ui.t('rel.superseded-by'))} <b>${esc(id)}</b>`),
      ...(cancelled.narrowed || []).map(narrow => `${esc(ui.t('rel.narrowed-by'))} <b>${esc(narrow.by)}</b> (${esc(narrow.mark)})`),
    ];
    return said.length ? `<div class="retract">${said.join('<br>')}</div>` : '';
  }

  /* The decisions before and after this one, with the passage each quoted
     (d-e239: the detail screen gives the thread). */
  function decisionFacts(node, ui) {
    const text = node.decision_scope_marked || (node.data.Decision && node.data.Decision.scope && node.data.Decision.scope.text) || ui.t('unrecorded');
    const edges = node.edges || [];
    const before = ['narrows', 'widens', 'supersedes', 'completes'];
    const after = ['narrowed-by', 'widened-by', 'superseded-by', 'completed-by'];
    const row = edge => {
      const link = `<a href="#/n/${encodeURIComponent(edge.to)}">${esc(edge.alias || edge.to)}</a>`;
      const mark = edge.mark ? ` <span class="dim">「${esc(edge.mark)}」</span>` : '';
      return `${esc(ui.t(`rel.${edge.name}`))} ${link}${mark}`;
    };
    const beforeRows = edges.filter(edge => before.includes(edge.name)).map(row).join('<br>');
    const afterRows = edges.filter(edge => after.includes(edge.name)).map(row).join('<br>');
    return kv(ui, 'scopeOf', esc(text)) +
      kv(ui, 'prevDecisions', beforeRows) + kv(ui, 'nextDecisions', afterRows) + retraction(node, ui);
  }

  function requirementFacts(node, ui, lang) {
    const requirement = node.data.Requirement || {};
    const at = value => window.GyTime.dateTime(value, lang);
    const step = (label, when) => `<div class="hi"><span class="when">${when ? at(when) : ui.t('pendingOn')}</span><span class="src">${esc(label)}</span></div>`;
    const heard = requirement.approval && requirement.approval.heard_by ? ` · ${esc(ui.t('heardBy'))} ${esc(requirement.approval.heard_by)}` : '';
    const end = requirement.cancellation
      ? step(ui.t('cancelledOn'), requirement.cancellation.at)
      : step(ui.t('doneOn'), requirement.completion && requirement.completion.at);
    const revisions = requirement.revisions && requirement.revisions.length ? String(requirement.revisions.length) : '';
    return `<div class="hist">` + step(ui.t('filedOn'), node.created) +
      step(ui.t('approvedOn') + heard, requirement.approval && requirement.approval.at) + end + `</div>` +
      kv(ui, 'reqRef', requirement.reference ? `<span class="mono">${esc(requirement.reference)}</span>` : '') +
      kv(ui, 'revisions', revisions);
  }

  function criterionFacts(node, ui) {
    const criterion = node.data.Criterion || {};
    const attributes = node.attributes || {};
    return kv(ui, 'metEvidence', criterion.evidence ? esc(criterion.evidence) : esc(ui.t('measure'))) +
      kv(ui, 'due', attributes.due ? `<span class="mono">${esc(attributes.due)}</span>` : '') +
      kv(ui, 'responsible', esc(attributes.responsible || '')) +
      kv(ui, 'then', esc(attributes.then || ''));
  }

  const FACT_MAKERS = { Need: needFacts, Question: questionFacts, Decision: decisionFacts, Requirement: requirementFacts, Criterion: criterionFacts };

  function facts(node, kind, ui, lang) {
    const maker = FACT_MAKERS[kind];
    const rows = maker ? maker(node, ui, lang) : '';
    return `<div class="sec" data-testid="facts"><h2>${esc(ui.t('factsTitle'))}</h2><div class="kv">${rows}</div></div>`;
  }

  function body(node, ui) {
    const text = (node.body_marked || node.body || '').trim() || ui.t('noBody');
    return `<div class="sec"><h2>${esc(ui.t('bodyTitle'))}</h2><pre class="body">${esc(text)}</pre></div>`;
  }

  /* The ways from this node to the crossing screen: its week, its scope around
     its creation, and a few words of its title. */
  function crossOut(node, ui) {
    const day = window.GyShape.at(node.created);
    const links = [];
    if (day !== null) {
      const date = new Date(day * 1000);
      const monday = day - ((date.getDay() + 6) % 7) * DAY;
      const dateOf = at => {
        const one = new Date(at * 1000);
        return `${one.getFullYear()}-${String(one.getMonth() + 1).padStart(2, '0')}-${String(one.getDate()).padStart(2, '0')}`;
      };
      links.push(`<a href="#/cross?from=${dateOf(monday)}&to=${dateOf(monday + 6 * DAY)}">${esc(fill(ui.t('madeWeek'), { from: dateOf(monday), to: dateOf(monday + 6 * DAY) }))}</a>`);
      if (node.scope) links.push(`<a href="#/cross?scope=${encodeURIComponent(node.scope)}&from=${dateOf(day - 3 * DAY)}&to=${dateOf(day + 3 * DAY)}">${esc(fill(ui.t('around3'), { scope: node.scope }))}</a>`);
    }
    const tokens = [...new Set((String(node.title).match(/[A-Za-z][A-Za-z0-9_.]{2,}|[ァ-ヴー]{3,}|[一-龥]{2,4}/g) || []))].slice(0, 6);
    for (const token of tokens) links.push(`<a href="#/cross?q=${encodeURIComponent(token)}">${esc(fill(ui.t('wordLink'), { w: token }))}</a>`);
    return `<div class="sec"><h2>${esc(ui.t('crossOut'))} <span class="sub">${esc(ui.t('crossOutSub'))}</span></h2><div class="outs" data-testid="crossLinks">${links.join('')}</div></div>`;
  }

  function history(node, ui, lang) {
    const rows = node.history || [];
    const list = rows.length
      ? rows.map(entry => `<div class="hi" role="listitem"><span class="when">${window.GyTime.dateTime(entry.at, lang)} · ${entry.seq}</span><span class="who">${esc(entry.actor)}</span><span><div class="src">${esc(entry.source)}</div><div class="why">${esc(entry.why)}</div></span></div>`).join('')
      : `<div class="empty">${esc(ui.t('migrated'))}</div>`;
    return `<div class="sec"><h2>${esc(ui.t('histTitle'))} <span class="sub">${esc(fill(ui.t('historySub'), { n: rows.length }))}</span></h2><div class="hist" role="list" aria-label="${esc(ui.t('histTitle'))}">${list}</div></div>`;
  }

  function status(state, kind, loaded, ui) {
    if (state.failed[kind]) {
      return `<div class="load fail" data-testid="failed"><b>${esc(ui.t('loadFailed'))}</b> <span>${esc(state.failed[kind])}</span><button data-act="retry">${esc(ui.t('retry'))}</button></div>`;
    }
    if (!loaded) return `<div class="load" data-testid="loading">${esc(ui.t('loading'))}</div>`;
    return '';
  }

  window.GyNode = { render };
})();
