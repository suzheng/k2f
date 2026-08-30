import { spawn } from "node:child_process";
import { createReadStream, existsSync, mkdtempSync, statSync } from "node:fs";
import http from "node:http";
import { tmpdir } from "node:os";
import { extname, join, normalize } from "node:path";

export const CHROME =
  process.env.CHROME ||
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm",
  ".K2F": "application/zip",
  ".k2f": "application/zip",
  ".json": "application/json",
};

/** Serve `root` over HTTP, open `path` in headless Chrome, return POST /__result body. */
export function runChromePage(root, path, { extra, timeoutMs = 30000 } = {}) {
  if (!existsSync(CHROME)) return null;
  return new Promise((resolve, reject) => {
    const server = http.createServer((req, res) => {
      if (req.method === "POST" && req.url === "/__result") {
        const chunks = [];
        req.on("data", (c) => chunks.push(c));
        req.on("end", () => {
          res.writeHead(204);
          res.end();
          resolve(Buffer.concat(chunks).toString("utf8"));
          server.close();
        });
        return;
      }
      if (extra && extra(req, res)) return;
      const rel = decodeURIComponent((req.url || "/").split("?")[0]);
      let file = join(root, rel === "/" ? path.replace(/^\//, "") : rel.slice(1));
      if (!normalize(file).startsWith(root)) {
        res.writeHead(404);
        res.end("not found");
        return;
      }
      if (existsSync(file) && statSync(file).isDirectory()) {
        file = join(file, "index.html");
      }
      if (!existsSync(file) || statSync(file).isDirectory()) {
        res.writeHead(404);
        res.end("not found");
        return;
      }
      res.writeHead(200, { "content-type": MIME[extname(file)] || "application/octet-stream" });
      createReadStream(file).pipe(res);
    });
    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      const profile = mkdtempSync(join(tmpdir(), "k2f-chrome-"));
      const child = spawn(
        CHROME,
        [
          "--headless=new",
          "--disable-gpu",
          "--no-first-run",
          "--disable-dev-shm-usage",
          `--user-data-dir=${profile}`,
          `http://127.0.0.1:${port}${path}`,
        ],
        { stdio: "ignore" },
      );
      const timer = setTimeout(() => {
        child.kill("SIGKILL");
        server.close();
        reject(new Error(`chrome timed out: ${path}`));
      }, timeoutMs);
      server.on("close", () => {
        clearTimeout(timer);
        child.kill("SIGKILL");
      });
    });
  });
}
