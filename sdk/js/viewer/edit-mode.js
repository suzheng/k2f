/** Toolbar Edit/Done toggle. View mode is default even when editing is allowed. */

export function bindEditMode({ button, menuButton, canEdit, signal, onChange }) {
  let editing = false;

  if (!canEdit) {
    button.hidden = true;
    button.disabled = true;
    if (menuButton) {
      menuButton.hidden = true;
      menuButton.disabled = true;
    }
    return {
      editing: () => false,
      setEditing(next) {},
    };
  }

  button.hidden = false;
  if (menuButton) menuButton.hidden = false;

  function sync() {
    const label = editing ? "Done" : "Edit";
    button.textContent = label;
    button.setAttribute("aria-pressed", editing ? "true" : "false");
    if (menuButton) {
      menuButton.textContent = label;
      menuButton.setAttribute("aria-pressed", editing ? "true" : "false");
    }
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
