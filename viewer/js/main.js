import { mountK2fViewer } from "../../sdk/js/viewer.js";
import { bindSearch } from "./search.js";

const root = document.getElementById("viewer-root");
const file = document.getElementById("file");
let handle = null;

const search = bindSearch(
  document.getElementById("search"),
  document.getElementById("search-hits"),
  (q, isId) => {
    if (!handle || !handle.viewer) return;
    if (isId) {
      jumpTo(q);
      return;
    }
    search.render(JSON.parse(handle.viewer.search(q)));
  },
);

function jumpTo(id) {
  const boxes = JSON.parse(handle.viewer.boxes_for(id));
  if (boxes.length) handle.goPage(boxes[0].page);
  handle.selectId(id);
}

async function openBytes(bytes) {
  if (handle) {
    await handle.open(bytes);
    return;
  }
  handle = await mountK2fViewer(root, bytes, { editable: true });
}

function onFile(dropped) {
  dropped.arrayBuffer().then((buf) => openBytes(new Uint8Array(buf)));
}

file.addEventListener("change", (e) => {
  const next = e.target.files && e.target.files[0];
  if (next) onFile(next);
});
document.addEventListener("dragover", (e) => e.preventDefault());
document.addEventListener("drop", (e) => {
  e.preventDefault();
  const next = e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0];
  if (next) onFile(next);
});
