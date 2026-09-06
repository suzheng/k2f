/** Toolbar Edit/Done toggle. View mode is default even when editing is allowed. */

export function bindEditMode({ button, canEdit, signal, onChange }) {
  let editing = false;

  if (!canEdit) {
    button.hidden = true;
    button.disabled = true;
    return {
      editing: () => false,
      setEditing(next) {},
    };
  }

  button.hidden = false;

  function sync() {
    button.textContent = editing ? "Done" : "Edit";
    button.setAttribute("aria-pressed", editing ? "true" : "false");
    onChange?.(editing);
  }

  button.textContent = "Edit";
  button.setAttribute("aria-pressed", "false");

  button.addEventListener(
    "click",
    () => {
      editing = !editing;
      sync();
    },
    { signal },
  );

  return {
    editing: () => editing,
    setEditing(next) {
      editing = Boolean(next);
      sync();
    },
  };
}
