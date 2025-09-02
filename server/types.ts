export type ShelfData = {
  // TODO: Add Devices
  commands: SavedCommand[];
};

export type SavedCommand = {
  id: number;
  command: string;
  description?: string;
  tags?: string[];
  is_template: boolean;
  timestamp?: string; // ISO
};

export type ServerCommand = SavedCommand & {
  // Auth token of device the command belongs to
  owner_token: string;
};
