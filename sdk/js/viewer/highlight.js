export function bindHighlight() {
  let current = null;
  return {
    hide() {
      if (current) current.hidden = true;
      current = null;
    },
    show(el, css) {
      if (!el) return;
      if (current && current !== el) current.hidden = true;
      current = el;
      el.hidden = false;
      el.style.left = `${css.left}px`;
      el.style.top = `${css.top}px`;
      el.style.width = `${css.width}px`;
      el.style.height = `${css.height}px`;
    },
  };
}
