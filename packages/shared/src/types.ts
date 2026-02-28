import { z } from "zod";

export const HookEventSchema = z.enum([
  "SessionStart",
  "UserPromptSubmit",
  "PreToolUse",
  "PostToolUse",
  "PostToolUseFailure",
  "Stop",
  "PreCompact",
  "SessionEnd",
]);

export const PermissionDecisionSchema = z.enum(["allow", "deny", "ask"]);

export const HookInputSchema = z.object({
  hook_event_name: HookEventSchema,
  session_id: z.string().optional(),
  cwd: z.string().optional(),
  tool_name: z.string().optional(),
  tool_input: z.record(z.unknown()).optional(),
  tool_response: z.string().optional(),
  prompt: z.string().optional(),
  error: z.string().optional(),
  last_assistant_message: z.string().optional(),
});

export type HookInput = z.infer<typeof HookInputSchema>;

export interface HookOutput {
  hookSpecificOutput?: {
    hookEventName?: string;
    permissionDecision?: "allow" | "deny" | "ask";
    permissionDecisionReason?: string;
    additionalContext?: string;
  };
}
