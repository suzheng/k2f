export function bindSearch(input, list, onPick) {
  input.addEventListener("keydown", (e) => {
    if (e.key !== "Enter") return;
    onPick(input.value);
  });
  return {
    render(ids) {
      list.replaceChildren();
      for (const id of ids) {
        const li = document.createElement("li");
        const btn = document.createElement("button");
        btn.type = "button";
        btn.textContent = id;
        btn.addEventListener("click", () => onPick(id, true));
        li.appendChild(btn);
        list.appendChild(li);
      }
    },
  };
}
