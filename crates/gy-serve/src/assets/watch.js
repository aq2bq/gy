/* The wait for the ledger to move (n-39f2; was live.js, n-478f): the head of
   the log, the long poll, the 30 s cut, and the 2 s retry. It is not a read:
   a read asks and answers, this one holds a line. The root hands in what to do
   when the line comes up or drops and when the log moves. */
(function () {
  const CUT = 30000;
  const RETRY = 2000;
  const pause = ms => new Promise(done => setTimeout(done, ms));

  /* The head of the log now, as the sequence the wait starts from. */
  async function head() {
    const res = await fetch('/api/shell');
    return (await res.json()).seq;
  }

  /* The next write after `after`, or the same when the wait was cut short. */
  async function waitPast(after) {
    const control = new AbortController();
    const timer = setTimeout(() => control.abort(), CUT);
    try {
      const res = await fetch(`/api/wait?after=${after}`, { signal: control.signal });
      return (await res.json()).seq;
    } finally {
      clearTimeout(timer);
    }
  }

  /* Follow the log: `onMoved` runs on every new sequence, `onLive` on every
     change of the line, and a dropped line retries after 2 s. */
  async function follow(onMoved, onLive) {
    let known = null;
    for (;;) {
      if (known === null) {
        try {
          known = await head();
        } catch {
          onLive(false);
          await pause(RETRY);
          continue;
        }
      }
      let seq;
      try {
        seq = await waitPast(known);
      } catch {
        onLive(false);
        await pause(RETRY);
        continue;
      }
      onLive(true);
      if (seq <= known) continue;
      known = seq;
      try {
        await onMoved();
      } catch {
        onLive(false);
      }
    }
  }

  window.GyWatch = { follow };
})();