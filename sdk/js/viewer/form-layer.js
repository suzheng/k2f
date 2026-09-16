import { boxToCss } from "./coords.js";

/** Native inputs over lock field boxes. Positions come from `form_fields` + `boxToCss`. */

export function buildFormLayer(
  pageWrap,
  fieldsOnThisPage,
  zoom,
  { values, onChange, onFocus, readOnly } = {},
) {
  pageWrap.querySelector(".k2f-form-layer")?.remove();
  if (!fieldsOnThisPage?.length) return null;
  const layer = document.createElement("div");
  layer.className = "k2f-form-layer";
  if (readOnly) layer.dataset.readonly = "true";
  for (const field of fieldsOnThisPage) {
    layer.appendChild(controlFor(field, zoom, values, onChange, onFocus, readOnly));
  }
  pageWrap.appendChild(layer);
  return layer;
}

export function removeFormLayer(pageWrap) {
  pageWrap.querySelector(".k2f-form-layer")?.remove();
}

export function fieldsOnPage(fields, page) {
  return (fields ?? []).filter((f) => f.page === page);
}

function controlFor(field, zoom, values, onChange, onFocus, readOnly) {
  const css = boxToCss(field, zoom);
  const kind = field.kind === "multiline" || field.kind === "checkbox" ? field.kind : "text";
  const current = readValue(values, field.id, field.value ?? "");
  const el =
    kind === "checkbox"
      ? checkboxControl(current, readOnly)
      : textControl(kind, current, field, readOnly);
  el.className = `k2f-form-control k2f-form-${kind}`;
  el.dataset.fieldId = field.id;
  el.setAttribute("aria-label", field.placeholder || field.id);
  place(el, css, kind, zoom);
  if (!readOnly) {
    const event = kind === "checkbox" ? "change" : "input";
    el.addEventListener(event, () => onChange?.(field.id, writeValue(kind, el)));
    el.addEventListener("focus", () => onFocus?.(field.id));
  }
  return el;
}

function textControl(kind, current, field, readOnly) {
  const el = document.createElement(kind === "multiline" ? "textarea" : "input");
  if (kind !== "multiline") el.type = "text";
  el.value = current;
  el.readOnly = Boolean(readOnly);
  el.autocomplete = "off";
  el.spellcheck = false;
  if (field.placeholder) el.placeholder = field.placeholder;
  if (field.max_length) el.maxLength = field.max_length;
  if (field.required) el.required = true;
  return el;
}

function checkboxControl(current, readOnly) {
  const el = document.createElement("input");
  el.type = "checkbox";
  el.checked = current === "true";
  el.disabled = Boolean(readOnly);
  return el;
}

function place(el, css, kind, zoom) {
  const inset = kind === "checkbox" ? 0 : Math.max(1, 2 * zoom);
  el.style.left = `${css.left + inset}px`;
  el.style.top = `${css.top + inset}px`;
  el.style.width = `${Math.max(0, css.width - inset * 2)}px`;
  el.style.height = `${Math.max(0, css.height - inset * 2)}px`;
  el.style.fontSize = `${fontSizePx(kind, css)}px`;
}

function fontSizePx(kind, css) {
  const ratio = kind === "multiline" ? 0.18 : 0.55;
  return Math.max(8, css.height * ratio);
}

function readValue(values, id, fallback) {
  if (!values) return fallback;
  if (typeof values.get === "function") return values.has(id) ? values.get(id) : fallback;
  return Object.hasOwn(values, id) ? values[id] : fallback;
}

function writeValue(kind, el) {
  if (kind === "checkbox") return el.checked ? "true" : "";
  return el.value;
}
