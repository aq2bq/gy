/* Live updates (n-478f): one long poll on /api/wait. When the ledger moves,
   the band, the sidebar, and the page redraw without a reload. */
(function () {
  /* The server answers a wait at 25 s; the cut covers a line that hangs. */
  const CUT = 30000;
  const RETRY = 2000;

  let known = null;

  const dot = () => document.querySelector('.live');
  const setLive = on => dot() && dot().classList.toggle('off', !on);
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
    if (window.GyTime) await window.GyTime.refresh();
    if (window.GyShell.at === null) await window.GyShell.reload();
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
