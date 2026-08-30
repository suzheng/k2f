/** Toggle browser fullscreen on the viewer host element. */

export function bindFullscreen({ button, target, signal }) {
  const root = target.getRootNode?.();
  const host = root?.host ?? target;
  const req = host.requestFullscreen?.bind(host);
  if (!req) {
    button.hidden = true;
    return;
  }

  function sync() {
    const on = document.fullscreenElement === host;
    button.textContent = on ? "Exit fullscreen" : "Fullscreen";
    button.dataset.act = on ? "exit-fullscreen" : "fullscreen";
  }

  button.addEventListener(
    "click",
    () => {
      if (document.fullscreenElement === host) {
        document.exitFullscreen();
      } else {
        req();
      }
    },
    { signal },
  );
  document.addEventListener("fullscreenchange", sync, { signal });
  sync();
}
