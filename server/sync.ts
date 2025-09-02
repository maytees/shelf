import { zValidator } from "@hono/zod-validator";
import { Hono } from "hono";
import {
  deleteCommand,
  getShelfData,
  replaceCommand,
  saveCommand,
} from "./config.ts";
import { shelfDataSchema } from "./schema.ts";
import type { SavedCommand } from "./types.ts";

const sync = new Hono();

// Excludes timestamp and id
function commandsAreDifferent(cmd1: SavedCommand, cmd2: SavedCommand): boolean {
  return (
    cmd1.command !== cmd2.command ||
    cmd1.description !== cmd2.description ||
    cmd1.is_template !== cmd2.is_template ||
    JSON.stringify(cmd1.tags?.sort()) !== JSON.stringify(cmd2.tags?.sort())
  );
}

sync.post("/", zValidator("json", shelfDataSchema), async (c) => {
  // New - from device
  const { commands } = c.req.valid("json");
  // Old - from server
  const { commands: data } = getShelfData();

  // Check timestamps and update/add/remove accordingly
  for (const command of commands) {
    const rule = (value: SavedCommand) => value.id === command.id;
    const serverHasCommand = data.some(rule);

    // If true, then command already exists on backend, so check timestamps
    const existing = data.find(rule);

    // Doesn't exist on server, add it
    if (!serverHasCommand) {
      saveCommand({ ...command, timestamp: new Date().toISOString() });
      continue;
    }

    if (!existing) continue;

    // Add timestamp now
    if (!existing.timestamp) existing.timestamp = new Date().toISOString();

    if (new Date(command.timestamp) >= new Date(existing.timestamp)) {
      // Server has the command, but check if it needs updating via timestamp
      // Needs update because the device gave a newer one

      replaceCommand(existing.id, {
        ...command,
        // Updates to now (last updated)
        timestamp: new Date().toISOString(),
      });
    }
  }

  const clientIds = new Set(commands.map((cmd) => cmd.id));
  for (const serverCommand of data) {
    // Deletion or from another device
    if (!clientIds.has(serverCommand.id)) {
      deleteCommand(serverCommand.id);
    }
  }

  // Return updated version of everything based on server
  return await c.json(getShelfData());
});

export default sync;
