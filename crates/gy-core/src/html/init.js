// ---------- tabs ----------
document.querySelectorAll('#tabs button').forEach(b => b.addEventListener('click', () => gotoTab(b.dataset.tab)));

// ---------- init ----------
initLayout();
buildGenealogy();
makeDefs();
resetView();
window.addEventListener('resize', draw);

// ---------- graph export? no (single-file, no external) ----------
})();
