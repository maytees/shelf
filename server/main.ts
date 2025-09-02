import { Hono } from "hono";
import sync from "./sync.ts";

const app = new Hono();

app.route("/sync", sync);

app.get("/", (c) => {
  return c.json("Hello World");
});

Deno.serve(app.fetch);
