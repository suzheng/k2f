/** Map engine errors to `{ code, message }` for agents. WASM often throws a raw string. */

export function errorMessage(err) {
  if (typeof err === "string") return err;
  if (err && err.message) return err.message;
  return String(err);
}

export function agentError(err) {
  let s = errorMessage(err);
  if (s.startsWith("Error: ")) s = s.slice(7);
  const e = new Error(s);
  const i = s.indexOf(":");
  e.code = i === -1 ? "INVALID_ARGUMENT" : s.slice(0, i);
  return e;
}

export function call(fn) {
  try {
    return fn();
  } catch (err) {
    throw agentError(err);
  }
}
