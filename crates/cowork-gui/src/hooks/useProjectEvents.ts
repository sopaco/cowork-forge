import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { TAURI_EVENTS } from '../constants';

interface UseProjectEventsOptions {
  onProjectLoaded?: () => void;
  onProjectCreated?: () => void;
}

/**
 * Hook for listening to project-related Tauri events
 * Separated from data fetching logic for better separation of concerns
 */
export function useProjectEvents(options: UseProjectEventsOptions = {}) {
  const { onProjectLoaded, onProjectCreated } = options;

  useEffect(() => {
    const unlistenProjectLoaded = listen(TAURI_EVENTS.PROJECT_LOADED, () => {
      onProjectLoaded?.();
    });

    const unlistenProjectCreated = listen(TAURI_EVENTS.PROJECT_CREATED, () => {
      onProjectCreated?.();
    });

    return () => {
      unlistenProjectLoaded.then((fn) => fn()).catch(() => {});
      unlistenProjectCreated.then((fn) => fn()).catch(() => {});
    };
  }, [onProjectLoaded, onProjectCreated]);
}
