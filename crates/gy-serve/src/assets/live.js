/* Live updates (n-478f): one long poll on /api/wait. When the ledger moves,
   the band, the sidebar, and the page redraw without a reload. */
(function () {
  /* The server answers a wait at 25 s; the cut covers a line that hangs. */
  const CUT = 30000;
  const RETRY = 2000;

  let known = null;

  const dot = () => document.querySelector('.live');
  const setLive = on => {
    const mark = document.querySelector('.wordmark .logo');
    if (mark) mark.classList.toggle('off', !on);
    const d = dot();
    if (d) d.classList.toggle('off', !on);
  };
  const pause = ms => new Promise(done => setTimeout(done, ms));

  /* The sequence the page is showing now. */
  async function head() {
    const res = await fetch('/api/shell');
    return (await res.json()).seq;
  }

  async function wait(after) {
    const control = new AbortController();
    const timer = setTimeout(() => control.abort(), CUT);
    try {
      const res = await fetch(`/api/wait?after=${after}`, { signal: control.signal });
      return (await res.json()).seq;
    } finally {
      clearTimeout(timer);
    }
  }

  /* The band follows the log; the sidebar and the page only at the head,
     never while a past point is chosen (n-32a9). */
  async function update() {
    /* Temporary until n-45ca folds this loop into root: the root re-reads the
       band, and the rest of the page, at the head. */
    if (window.GyRoot) await window.GyRoot.moved();
  }

  async function loop() {
    for (;;) {
      if (known === null) {
        try {
          known = await head();
        } catch {
          setLive(false);
          await pause(RETRY);
          continue;
        }
      }
      let seq;
      try {
        seq = await wait(known);
      } catch {
        setLive(false);
        await pause(RETRY);
        continue;
      }
      setLive(true);
      if (seq <= known) {
        continue;
      }
      known = seq;
      try {
        await update();
      } catch {
        setLive(false);
      }
    }
  }

  loop();
})();
