/* Make only genuinely scrollable document regions keyboard-reachable. */
(() => {
  let queued = false;
  function update() {
    queued = false;
    document.querySelectorAll('table,pre,.source-clause,.retained-group,.fig-scroll').forEach(el => {
      const css = getComputedStyle(el);
      const visible = el.getClientRects().length && el.clientWidth > 0;
      const scrolls = visible && ((/auto|scroll/.test(css.overflowX) && el.scrollWidth > el.clientWidth + 1) || (/auto|scroll/.test(css.overflowY) && el.scrollHeight > el.clientHeight + 1));
      if (scrolls && el.tabIndex < 0) {
        el.tabIndex = 0;
        el.dataset.scrollFocus = 'managed';
        el.classList.add('keyboard-scroll');
        if (!el.hasAttribute('aria-label')) {
          const heading = el.closest('article,section,main')?.querySelector('h4,h3,h2,h1');
          el.setAttribute('aria-label', `Scrollable ${el.tagName === 'TABLE' ? 'table' : 'content'}${heading ? ': ' + heading.textContent.trim() : ''}`);
          el.dataset.scrollLabel = 'managed';
        }
      } else if (!scrolls && el.dataset.scrollFocus === 'managed') {
        el.removeAttribute('tabindex');
        el.classList.remove('keyboard-scroll');
        delete el.dataset.scrollFocus;
        if (el.dataset.scrollLabel === 'managed') { el.removeAttribute('aria-label'); delete el.dataset.scrollLabel; }
      }
    });
  }
  const schedule = () => { if (!queued) { queued = true; requestAnimationFrame(update); } };
  document.addEventListener('DOMContentLoaded', schedule);
  document.addEventListener('toggle', schedule, true);
  document.addEventListener('htmx:afterSwap', schedule);
  window.addEventListener('resize', schedule);
  if (document.fonts) document.fonts.ready.then(schedule);
  schedule();
})();
