/** Header hide-on-scroll and idle status bar. */

export function bannerIsPinned(banner) {
  if (!banner || banner.hidden) return false;
  return (
    banner.classList.contains("banner-broken") ||
    banner.classList.contains("banner-signed-broken")
  );
}

export function formatBytes(n) {
  const bytes = Number(n) || 0;
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function bindChromeScroll({ root, chrome, stage, banner, signal }) {
  let last = stage.scrollTop;

  function show() {
    chrome.style.marginTop = "0px";
    root.classList.remove("is-chrome-hidden");
  }

  function hide() {
    if (bannerIsPinned(banner) || stage.scrollTop < 8) {
      show();
      return;
    }
    chrome.style.marginTop = `-${chrome.offsetHeight}px`;
    root.classList.add("is-chrome-hidden");
  }

  function syncPin() {
    if (bannerIsPinned(banner)) {
      root.classList.add("is-chrome-pinned");
      show();
    } else {
      root.classList.remove("is-chrome-pinned");
    }
  }

  stage.addEventListener(
    "scroll",
    () => {
      const y = stage.scrollTop;
      const dy = y - last;
      last = y;
      if (bannerIsPinned(banner)) return;
      if (y < 8 || dy < -4) show();
      else if (dy > 4) hide();
    },
    { signal, passive: true },
  );

  return { syncPin, show, hide };
}

export function bindStatusBar({ stage, status, signal }) {
  let timer = 0;

  function show() {
    status.classList.add("is-visible");
    clearTimeout(timer);
    timer = setTimeout(() => {
      status.classList.remove("is-visible");
    }, 2000);
  }

  stage.addEventListener("pointermove", show, { signal });
  stage.addEventListener("scroll", show, { signal, passive: true });
  signal.addEventListener("abort", () => clearTimeout(timer));
  show();
  return { show };
}
