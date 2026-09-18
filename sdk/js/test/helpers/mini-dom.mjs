/** Tiny DOM for viewer unit tests. Not a browser replacement. */

export function installMiniDom() {
  const document = {
    createElement(tag) {
      return makeEl(tag);
    },
  };
  globalThis.document = document;
  return document;
}

function makeEl(tag) {
  const el = {
    tagName: String(tag).toUpperCase(),
    className: "",
    children: [],
    parent: null,
    style: {},
    dataset: {},
    attributes: {},
    listeners: {},
    hidden: false,
    value: "",
    checked: false,
    type: "",
    placeholder: "",
    readOnly: false,
    disabled: false,
    required: false,
    maxLength: 0,
    autocomplete: "",
    spellcheck: true,
    appendChild(child) {
      child.parent = el;
      el.children.push(child);
      return child;
    },
    append(...nodes) {
      for (const n of nodes) el.appendChild(n);
    },
    remove() {
      if (!el.parent) return;
      el.parent.children = el.parent.children.filter((c) => c !== el);
      el.parent = null;
    },
    querySelector(sel) {
      return walk(el, sel)[0] ?? null;
    },
    querySelectorAll(sel) {
      return walk(el, sel);
    },
    addEventListener(type, fn) {
      el.listeners[type] = fn;
    },
    setAttribute(name, value) {
      el.attributes[name] = String(value);
      if (name.startsWith("data-")) {
        el.dataset[dataToCamel(name.slice(5))] = String(value);
      }
    },
    getAttribute(name) {
      return el.attributes[name];
    },
    focus() {
      el.listeners.focus?.();
    },
    emit(type) {
      el.listeners[type]?.();
    },
  };
  return el;
}

function dataToCamel(name) {
  return name.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
}

function walk(root, sel) {
  const out = [];
  const stack = [...root.children];
  while (stack.length) {
    const n = stack.shift();
    if (match(n, sel)) out.push(n);
    stack.unshift(...n.children);
  }
  return out;
}

function match(el, sel) {
  if (sel.startsWith(".")) {
    return el.className.split(/\s+/).includes(sel.slice(1));
  }
  if (sel === "[data-field-id]") return Boolean(el.dataset.fieldId);
  return el.tagName === sel.toUpperCase();
}
