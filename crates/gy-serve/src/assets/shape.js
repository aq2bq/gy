/* The shape the three screens read (a part): the API's answer, made into the
   fields the regions draw. The keys are the ones the server's doc comments
   name (crates/gy-serve/src/api/{overview,cross,path}.rs, r-44c7, d-6d90), and
   this file is the one place to change when they move. It judges nothing about
   the record: stage, blocked, blockers, and the counts come from the server. */
(function () {
  const KINDS = {
    n: 'Need', need: 'Need', q: 'Question', question: 'Question', d: 'Decision', decision: 'Decision',
    r: 'Requirement', requirement: 'Requirement', a: 'Criterion', ac: 'Criterion', criterion: 'Criterion',
  };
  const STAGE = {
    before: 'before', '着手前': 'before', build: 'build', '作っている': 'build',
    verify: 'verify', '確かめ待ち': 'verify', close: 'close', '閉じ待ち': 'close',
  };
  const OPEN = new Set(['open', 'filed', 'approved', 'unmet', 'unsatisfied', 'live', 'in progress', '進行中', '起票', '確定', '未達', '有効']);
  const DAY = 86400;

  const first = (object, names) => {
    for (const name of names) {
      if (object && object[name] !== undefined && object[name] !== null) return object[name];
    }
    return undefined;
  };

  /* A kind from `Need`, `need`, or an id's prefix (`n-`, `ac-`). */
  function kind(value, id) {
    const text = String(value ?? '').toLowerCase();
    if (KINDS[text]) return KINDS[text];
    const prefix = String(id ?? text).split('-')[0].toLowerCase();
    return KINDS[prefix] || String(value ?? '');
  }

  /* A state the screens test: the server's word, lowercased. */
  const stateOf = value => String(value ?? '').toLowerCase();
  const isOpen = value => OPEN.has(stateOf(value));

  /* A stored instant as epoch seconds: an RFC 3339 string, a `YYYY-MM-DD` day,
     a number, or an all-digit string. Null when it is none. */
  function at(value) {
    if (value === null || value === undefined || value === '') return null;
    if (typeof value === 'number') return value;
    const text = String(value);
    if (/^-?\d+$/.test(text)) return Number(text);
    const parsed = /^\d{4}-\d{2}-\d{2}$/.test(text)
      ? new Date(`${text}T00:00:00`)
      : new Date(text);
    return Number.isNaN(parsed.getTime()) ? null : Math.round(parsed.getTime() / 1000);
  }

  /* The local calendar day's first second, for the pulse's window. */
  function dayStart(value) {
    const date = new Date(value * 1000);
    date.setHours(0, 0, 0, 0);
    return Math.round(date.getTime() / 1000);
  }

  /* A waiting item: the node that stops a need, with who decides a question. */
  function waiting(item) {
    return {
      id: first(item, ['id', 'node']),
      kind: kind(first(item, ['kind', 'type']), first(item, ['id', 'node'])),
      title: first(item, ['title', 'name']) || '',
      decider: first(item, ['decider', 'who']) || '',
    };
  }

  /* One need on the board: the server's stage and blocked mark, its waiting,
     the requirements and criteria it holds, and when it last moved. */
  function need(item) {
    const id = first(item, ['id', 'node']);
    const requirements = first(item, ['requirements', 'reqs']) || {};
    const criteria = first(item, ['criteria', 'acs']) || {};
    return {
      id,
      scope: first(item, ['scope']) || '',
      title: first(item, ['title', 'name']) || '',
      stage: STAGE[String(first(item, ['stage', 'progress']) ?? '').toLowerCase()] || 'before',
      blocked: !!first(item, ['blocked', 'stopped', 'stuck']),
      requirements: {
        filed: Number(first(requirements, ['filed']) ?? 0),
        approved: Number(first(requirements, ['approved']) ?? 0),
        done: Number(first(requirements, ['done']) ?? 0),
        cancelled: Number(first(requirements, ['cancelled']) ?? 0),
      },
      criteria: {
        satisfied: Number(first(criteria, ['satisfied']) ?? 0),
        unsatisfied: Number(first(criteria, ['unsatisfied']) ?? 0),
        withdrawn: Number(first(criteria, ['withdrawn']) ?? 0),
      },
      waiting: (first(item, ['waiting', 'waits', 'blockers']) || []).map(waiting),
      last: at(first(item, ['last_moved', 'last_activity', 'last'])),
    };
  }

  /* One root of a jam: the node the waiting ends at, and the needs it holds. */
  function root(item) {
    const list = value => (Array.isArray(value) ? value : []);
    const ids = value => list(value).map(entry => (typeof entry === 'string' ? entry : first(entry, ['id', 'node']))).filter(Boolean);
    const id = first(item, ['id', 'node']);
    return {
      id,
      kind: kind(first(item, ['kind', 'type']), id),
      scope: first(item, ['scope']) || '',
      title: first(item, ['title', 'name']) || '',
      decider: first(item, ['decider', 'who']) || '',
      needs: ids(first(item, ['needs', 'blocked', 'held'])),
      via: ids(first(item, ['via', 'through', 'path'])),
    };
  }

  /* One week of the flow: how many needs were opened and finished. */
  function week(item, index) {
    return {
      from: at(first(item, ['from', 'start', 'week'])) ?? index,
      opened: Number(first(item, ['opened', 'added', 'raised']) ?? 0),
      done: Number(first(item, ['done', 'finished', 'closed']) ?? 0),
    };
  }

  /* The pulse as 49 days ending on the answer's day: the API leaves out the
     quiet days, the screen draws them. */
  function densePulse(points, now) {
    const end = now === null ? null : dayStart(now);
    if (end === null) return [];
    const start = end - 48 * DAY;
    const values = new Array(49).fill(0);
    for (const point of points || []) {
      const day = at(first(point, ['day', 'from', 'at']) ?? point);
      const writes = Number(first(point, ['writes', 'count', 'n']) ?? (typeof point === 'number' ? point : 0));
      if (day === null) continue;
      const index = Math.round((day - start) / DAY);
      if (index >= 0 && index < values.length) values[index] += writes;
    }
    return values;
  }

  /* One scope row: its counts and the pulse of writes. */
  function scopeRow(item, now) {
    return {
      name: first(item, ['name', 'scope']) || '',
      open: Number(first(item, ['open', 'opened']) ?? 0),
      done: Number(first(item, ['closed', 'done', 'finished']) ?? 0),
      blocked: Number(first(item, ['blocked', 'stopped', 'stuck']) ?? 0),
      pulse: densePulse(first(item, ['pulse', 'writes', 'days']), now),
    };
  }

  /* One node of the unheld list or the overdue list. */
  function held(item) {
    return {
      id: first(item, ['id', 'node']),
      kind: kind(first(item, ['kind', 'type']), first(item, ['id', 'node'])),
      scope: first(item, ['scope']) || '',
      state: stateOf(first(item, ['state', 'status'])),
      title: first(item, ['title', 'name']) || '',
      due: first(item, ['due']) || '',
      responsible: first(item, ['responsible']) || '',
      then: first(item, ['then']) || '',
    };
  }

  const listOf = (body, names) => first(body, names) || [];
  const objectOf = (body, names) => first(body, names) || {};

  function totalsOf(counts) {
    return {
      open: Number(first(counts, ['open_needs', 'open', 'openNeeds']) ?? 0),
      blocked: Number(first(counts, ['blocked_needs', 'blocked', 'stopped']) ?? 0),
      questions: Number(first(counts, ['open_questions', 'questions', 'openQuestions']) ?? 0),
      unmet: Number(first(counts, ['unmet_criteria', 'unmet', 'unmetCriteria']) ?? 0),
      done: Number(first(counts, ['closed_needs', 'done', 'doneNeeds']) ?? 0),
    };
  }

  function decidersOf(body) {
    return listOf(body, ['deciders', 'who_decides']).map(item => ({
      name: first(item, ['name', 'who']) || '',
      count: Number(first(item, ['count', 'n']) ?? 0),
    }));
  }

  /* GET /api/overview: the first screen's whole answer. */
  function overview(body) {
    const data = body || {};
    const scopes = listOf(data, ['scopes', 'scope_rows', 'scopeRows']);
    const now = at(first(data, ['at', 'now']));
    return {
      needs: listOf(data, ['needs', 'open_needs', 'openNeeds']).map(need),
      roots: listOf(data, ['blockers', 'roots', 'jams']).map(root),
      weeks: listOf(data, ['weeks', 'flow', 'weekly']).map(week),
      scopes: scopes.map(item => scopeRow(item, now)),
      deciders: decidersOf(data),
      overdue: listOf(data, ['overdue', 'late']).map(held),
      unheld: listOf(data, ['unheld', 'unowned']).map(held),
      totals: totalsOf(objectOf(data, ['counts', 'totals', 'summary'])),
      now,
      seq: first(data, ['seq']),
      scopesKnown: scopes.map(item => ({ name: first(item, ['name', 'scope']) || '' })),
    };
  }

  /* One node of the cross answer. */
  function crossNode(item) {
    const id = first(item, ['id', 'node']);
    return {
      id,
      kind: kind(first(item, ['kind', 'type']), id),
      scope: first(item, ['scope']) || '',
      state: stateOf(first(item, ['state', 'status'])),
      created: at(first(item, ['created', 'at'])),
      title: first(item, ['title', 'name']) || '',
      degree: Number(first(item, ['degree', 'links', 'connections']) ?? 0),
    };
  }

  /* One kind's count within a scope. */
  function kindCount(item, name) {
    return {
      kind: kind(name || first(item, ['kind', 'type']), null),
      total: Number(first(item, ['total', 'count', 'n']) ?? 0),
      open: Number(first(item, ['open', 'opened']) ?? 0),
    };
  }

  /* A scope's counts: `kinds: [{kind, total, open}]`. */
  function scopeCounts(item) {
    return {
      name: first(item, ['name', 'scope']) || '',
      kinds: (first(item, ['kinds', 'counts_by_kind', 'by_kind']) || []).map(entry => kindCount(entry, first(entry, ['kind', 'type']))),
    };
  }

  /* An event count: a list of `{what, count}`. */
  function events(value) {
    return (Array.isArray(value) ? value : []).map(item => ({
      kind: String(first(item, ['what', 'kind', 'event', 'name']) || '').toLowerCase(),
      count: Number(first(item, ['count', 'n']) ?? 0),
    }));
  }

  /* A heat cell: `scope × week`, as a count. */
  function cell(item) {
    return {
      scope: first(item, ['scope', 'name']) || '',
      week: at(first(item, ['from', 'week', 'start'])),
      count: Number(first(item, ['moves', 'count', 'n']) ?? 0),
    };
  }

  /* GET /api/cross: the narrowed set, or the grid when it is not narrowed or
     too large (capped). `nodes` and `edges` are absent when capped. */
  function cross(body) {
    body = body || {};
    const nodes = first(body, ['nodes', 'rows']) || [];
    const edges = first(body, ['edges', 'links']) || [];
    const scopes = first(body, ['scopes', 'scope_counts', 'counts']) || [];
    return {
      capped: !!first(body, ['capped', 'truncated']),
      total: nodes.length,
      seq: first(body, ['seq']),
      now: at(first(body, ['at', 'now'])),
      nodes: nodes.map(crossNode),
      edges: edges.map(link => ({
        from: first(link, ['from', 'a', 'source']),
        to: first(link, ['to', 'b', 'target']),
        relation: first(link, ['relation', 'rel', 'name']) || '',
      })),
      scopes: (Array.isArray(scopes) ? scopes : []).map(scopeCounts),
      events: events(first(body, ['events', 'actions', 'happened'])),
      cells: (first(body, ['weeks', 'cells', 'grid', 'heat']) || []).map(cell),
    };
  }

  /* One node of a stage in the thread. */
  function stageNode(item) {
    const id = first(item, ['id', 'node']);
    return {
      id,
      kind: kind(first(item, ['kind', 'type']), id),
      scope: first(item, ['scope']) || '',
      state: stateOf(first(item, ['state', 'status'])),
      title: first(item, ['title', 'name']) || '',
      relation: first(item, ['relation', 'rel', 'name']) || '',
      parent: first(item, ['parent']) || null,
      mark: first(item, ['mark', 'note']) || '',
    };
  }

  /* One step: `{nodes, hidden}`. */
  function stage(item) {
    if (Array.isArray(item)) return { nodes: item.map(stageNode), more: 0 };
    return {
      nodes: (first(item, ['nodes', 'items', 'rows']) || []).map(stageNode),
      more: Number(first(item, ['hidden', 'more', 'rest']) ?? 0),
    };
  }

  const side = value => (Array.isArray(value) ? value.map(stage) : []);

  /* GET /api/node/<id>/path?depth=2|3: the thread, left (`up`) and right
     (`down`), around the center. */
  function path(body) {
    body = body || {};
    return {
      id: first(body, ['id', 'node']) || (body.center && body.center.id),
      center: body.center ? stageNode(body.center) : null,
      depth: Number(first(body, ['depth']) ?? 2),
      left: side(first(body, ['up', 'left', 'before', 'parents'])),
      right: side(first(body, ['down', 'right', 'after', 'children'])),
    };
  }

  window.GyShape = { kind, state: stateOf, isOpen, at, overview, cross, path, stageNode };
})();
