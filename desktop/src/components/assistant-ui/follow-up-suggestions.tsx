"use client";

import { AuiIf, useAuiState, ThreadPrimitive } from "@assistant-ui/react";
import type { FC } from "react";

export const ThreadFollowupSuggestions: FC = () => {
  const suggestions = useAuiState((s) => s.thread.suggestions);
  return (
    <AuiIf
      condition={(s) =>
        !s.thread.isEmpty &&
        !s.thread.isRunning &&
        s.thread.suggestions.length > 0
      }
    >
      <div className="aui-thread-followup-suggestions flex max-h-20 min-h-8 items-center gap-2 overflow-x-auto overflow-y-hidden px-1 pb-1">
        {suggestions.map((suggestion, idx) => (
          <ThreadPrimitive.Suggestion
            key={idx}
            className="aui-thread-followup-suggestion max-w-60 shrink-0 rounded-full bg-[color:var(--color-surface)] px-3 py-2 text-left text-xs leading-snug text-[color:var(--color-fg-sub)] transition-colors ease-in hover:bg-[color:var(--color-card-hov)] hover:text-[color:var(--color-fg)]"
            prompt={suggestion.prompt}
            method="replace"
            autoSend
          >
            {suggestion.prompt}
          </ThreadPrimitive.Suggestion>
        ))}
      </div>
    </AuiIf>
  );
};
