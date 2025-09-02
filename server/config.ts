import { existsSync } from "jsr:@std/fs/exists";
import { parse, stringify } from "jsr:@std/toml";
import type { SavedCommand, ShelfData } from "./types.ts";

// TODO: Update TEMP_PATH to a valid path
const TEMP_PATH = "./data.toml";

export function getShelfData(): ShelfData {
  const path = TEMP_PATH;
  if (!existsSync(path)) {
    Deno.writeTextFileSync(path, "");
  }
  const content: string = Deno.readTextFileSync(path);

  // Handle empty content or whitespace-only content
  if (!content.trim()) {
    return { commands: [] };
  }

  // Parse and provide fallback if parse returns undefined/null
  const parsed = parse(content) as ShelfData;
  return parsed || { commands: [] };
}

export function writeShelfData(data: ShelfData): void {
  const path = TEMP_PATH;
  const content: string = stringify(data);
  Deno.writeTextFileSync(path, content);
}

export function saveCommand(command: SavedCommand): void {
  const dat = getShelfData();
  dat.commands.push(command);
  writeShelfData(dat);
}

export function deleteCommand(id: number): void {
  const dat = getShelfData();
  dat.commands = dat.commands.filter((command) => command.id !== id);
  writeShelfData(dat);
}

export function replaceCommand(id: number, newCommand: SavedCommand): void {
  const dat = getShelfData();
  dat.commands = dat.commands.map((c) => (c.id === id ? newCommand : c));
  writeShelfData(dat);
}
