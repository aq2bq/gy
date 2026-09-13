// ---------- tabs ----------
document.querySelectorAll('#tabs button').forEach(b => b.addEventListener('click', () => gotoTab(b.dataset.tab)));

// ---------- init ----------
initLayout();
buildGenealogy();
makeDefs();
restoreLocation();
window.addEventListener('hashchange', restoreLocation);
// Capture also sees node clicks that stop propagation; save after their handlers.
['click', 'input', 'change', 'keydown'].forEach(event => document.addEventListener(event, () => setTimeout(saveLocation, 0), true));
window.addEventListener('resize', draw);

// ---------- graph export? no (single-file, no external) ----------
})();
