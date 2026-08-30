/** Current page from scroll, plus Prev/Next jump-to-page. */

export const PAGE_GAP = 24;

/**
 * Last page whose top is at or above viewportTop + 25% of viewport height.
 * `pageTops` are content Ys in the same space as `scrollTop`.
 */
export function pageAtScroll(scrollTop, viewportHeight, pageTops) {
  if (!pageTops.length) return 0;
  const probe = scrollTop + viewportHeight * 0.25;
  for (let i = pageTops.length - 1; i >= 0; i--) {
    if (pageTops[i] <= probe) return i;
  }
  return 0;
}

export function bindPageNav({ prev, next, label, wrapsOf, stage, onChange, signal }) {
  let page = 0;
  let count = 0;
  let suppress = false;

  function render() {
    prev.disabled = page <= 0;
    next.disabled = page + 1 >= count;
    label.textContent = count === 0 ? "—" : `${page + 1} / ${count}`;
  }

  function setPage(n, notify) {
    if (n < 0 || n >= count || n === page) {
      render();
      return false;
    }
    page = n;
    render();
    if (notify) onChange(page);
    return true;
  }

  function wrapTops() {
    if (!stage) return [];
    const base = stage.scrollTop;
    const stageTop = stage.getBoundingClientRect().top;
    return wrapsOf().map((w) => w.getBoundingClientRect().top - stageTop + base);
  }

  function go(n) {
    if (n < 0 || n >= count) return;
    const wraps = wrapsOf();
    const wrap = wraps[n];
    if (!wrap || !stage) {
      setPage(n, true);
      return;
    }
    suppress = true;
    wrap.scrollIntoView({ block: "start" });
    setPage(n, true);
    queueMicrotask(() => {
      suppress = false;
    });
  }

  function syncFromScroll() {
    if (suppress || !stage || count === 0) return;
    const nextPage = pageAtScroll(stage.scrollTop, stage.clientHeight, wrapTops());
    setPage(nextPage, true);
  }

  prev.addEventListener(
    "click",
    () => {
      if (page > 0) go(page - 1);
    },
    { signal },
  );
  next.addEventListener(
    "click",
    () => {
      if (page + 1 < count) go(page + 1);
    },
    { signal },
  );
  stage?.addEventListener("scroll", syncFromScroll, { signal, passive: true });

  return {
    reset(n) {
      count = n;
      page = 0;
      render();
    },
    go,
    page: () => page,
    syncFromScroll,
  };
}
