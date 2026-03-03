import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { TAURI_EVENTS } from '../constants';

interface UseIterationEventsOptions {
  onIterationCreated?: () => void;
  onIterationStarted?: () => void;
  onIterationContinued?: () => void;
  onIterationCompleted?: () => void;
  onIterationFailed?: () => void;
  onAgentEvent?: (content: string) => void;
  onExecutingIdChange?: (id: string | null) => void;
}

/**
 * Hook for listening to iteration-related Tauri events
 * Separated from data fetching logic for better separation of concerns
 */
export function useIterationEvents(options: UseIterationEventsOptions = {}) {
  const {
    onIterationCreated,
    onIterationStarted,
    onIterationContinued,
    onIterationCompleted,
    onIterationFailed,
    onAgentEvent,
    onExecutingIdChange,
  } = options;

  useEffect(() => {
    const unlistenCreated = listen(TAURI_EVENTS.ITERATION_CREATED, () => {
      onIterationCreated?.();
    });

    const unlistenStarted = listen(TAURI_EVENTS.ITERATION_STARTED, () => {
      setTimeout(() => onIterationStarted?.(), 500);
    });

    const unlistenContinued = listen(TAURI_EVENTS.ITERATION_CONTINUED, () => {
      setTimeout(() => onIterationContinued?.(), 500);
    });

    const unlistenAgentEvent = listen(TAURI_EVENTS.AGENT_EVENT, (event) => {
      const content = (event.payload as { content?: string })?.content || "";
      if (content.includes("Starting stage:") && onAgentEvent) {
        setTimeout(() => onAgentEvent(content), 100);
      }
    });

    const unlistenCompleted = listen(TAURI_EVENTS.ITERATION_COMPLETED, () => {
      onIterationCompleted?.();
      onExecutingIdChange?.(null);
    });

    const unlistenFailed = listen(TAURI_EVENTS.ITERATION_FAILED, () => {
      onIterationFailed?.();
      onExecutingIdChange?.(null);
    });

    return () => {
      unlistenCreated.then((fn) => fn()).catch(() => {});
      unlistenStarted.then((fn) => fn()).catch(() => {});
      unlistenContinued.then((fn) => fn()).catch(() => {});
      unlistenAgentEvent.then((fn) => fn()).catch(() => {});
      unlistenCompleted.then((fn) => fn()).catch(() => {});
      unlistenFailed.then((fn) => fn()).catch(() => {});
    };
  }, [
    onIterationCreated,
    onIterationStarted,
    onIterationContinued,
    onIterationCompleted,
    onIterationFailed,
    onAgentEvent,
    onExecutingIdChange,
  ]);
}
