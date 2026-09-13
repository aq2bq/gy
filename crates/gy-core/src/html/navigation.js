function startHop(id) {
  const pick = pickHop(id);           // H27
  hopFrom = id; hopN = pick.n; hopInfo = pick;
  genealogyMode = false;
  redraw();
}

function continuousRedraw() { draw(); }

