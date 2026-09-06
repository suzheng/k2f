/** Positioned dropdown menus inside the viewer root. */

export function createMenu(className = "") {
  const el = document.createElement("div");
  el.className = className ? `k2f-menu ${className}` : "k2f-menu";
  el.setAttribute("role", "menu");
  el.hidden = true;
  return el;
}

export function menuItem({ label, value, checked = false }) {
  const el = document.createElement("button");
  el.type = "button";
  el.className = "k2f-menu-item";
  el.setAttribute("role", "menuitem");
  el.textContent = label;
  if (value != null) el.dataset.value = value;
  el.dataset.checked = checked ? "true" : "false";
  return el;
}

export function menuSep() {
  const el = document.createElement("div");
  el.className = "k2f-menu-sep";
  el.setAttribute("role", "separator");
  return el;
}

function position(menu, trigger, root, align) {
  const rootBox = root.getBoundingClientRect();
  const btnBox = trigger.getBoundingClientRect();
  const top = btnBox.bottom - rootBox.top + 4;
  menu.style.top = `${top}px`;
  menu.style.left = "auto";
  menu.style.right = "auto";
  const width = menu.offsetWidth;
  if (align === "left") {
    const left = btnBox.left - rootBox.left;
    menu.style.left = `${Math.max(8, left)}px`;
    return;
  }
  const right = rootBox.right - btnBox.right;
  const overflow = btnBox.right - rootBox.left - width;
  if (overflow < 8) menu.style.left = "8px";
  else menu.style.right = `${Math.max(8, right)}px`;
}

export function createMenuController({ root, signal }) {
  const entries = [];

  function closeAll() {
    for (const entry of entries) entry.close();
  }

  function register({ trigger, menu, align = "right" }) {
    trigger.setAttribute("aria-haspopup", "true");
    trigger.setAttribute("aria-expanded", "false");

    function close() {
      menu.hidden = true;
      trigger.setAttribute("aria-expanded", "false");
    }

    function open() {
      closeAll();
      menu.hidden = false;
      position(menu, trigger, root, align);
      trigger.setAttribute("aria-expanded", "true");
    }

    trigger.addEventListener(
      "click",
      (e) => {
        e.stopPropagation();
        if (menu.hidden) open();
        else close();
      },
      { signal },
    );

    const entry = { open, close, menu, trigger };
    entries.push(entry);
    return entry;
  }

  document.addEventListener(
    "pointerdown",
    (e) => {
      const path = typeof e.composedPath === "function" ? e.composedPath() : [];
      if (entries.some((en) => path.includes(en.menu) || path.includes(en.trigger))) return;
      closeAll();
    },
    { signal },
  );
  document.addEventListener(
    "keydown",
    (e) => {
      if (e.key === "Escape") closeAll();
    },
    { signal },
  );

  return { register, closeAll };
}
