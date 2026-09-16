/* The camera's ease (n-88b2): while state.graph.target is set, one rAF step per
   frame moves the camera toward it. The root hands in the state's accessors and
   the one frame to draw; the curve is graph.js's, pure. */
(function () {
  function make(ctx) {
    let ease = null, frame = null;

    /* One frame: the camera at the elapsed time, then the graph redrawn. */
    function tick() {
      frame = null;
      const state = ctx.get();
      if (!ease || !state.graph.target) { ease = null; return; }
      const next = window.GyGraph.eased(ease.from, state.graph.target, performance.now() - ease.start);
      ctx.cam({ x: next.x, y: next.y, k: next.k });
      if (next.done) ctx.arrived();
      ctx.draw();
      if (!next.done && ctx.get().graph.target) {
        frame = requestAnimationFrame(tick);
        return;
      }
      ease = null;
    }

    /* Begin a fit to `target` from where the camera is now. */
    function start(target) {
      if (!target) { stop(); return; }
      ctx.aim(target);
      ease = { from: { ...ctx.get().graph.cam }, start: performance.now() };
      if (!frame) frame = requestAnimationFrame(tick);
    }

    /* Drop any ease in flight. */
    function stop() {
      ease = null;
      if (ctx.get().graph.target !== null) ctx.arrived();
    }

    return { start, stop };
  }

  window.GyEase = { make };
})();