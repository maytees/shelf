import { z } from "zod";

export const savedCommandSchema = z.object({
  id: z.number(),
  command: z.string(),
  description: z.string(),
  tags: z.array(z.string()).optional(),
  is_template: z.boolean().default(false),
  timestamp: z.string().optional().default(new Date().toISOString()),
});

export const shelfDataSchema = z.object({
  commands: z.array(savedCommandSchema),
});
