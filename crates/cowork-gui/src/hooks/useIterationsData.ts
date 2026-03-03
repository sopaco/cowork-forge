import { useState, useEffect } from "react";
import { App } from "antd";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface IterationInfo {
  id: string;
  number: number;
  title: string;
  description: string;
  status: string;
  current_stage: string | null;
  created_at: string;
  completed_at?: string;
  completed_stages: string[];
  base_iteration_id?: string;
  inheritance?: string;
}

interface ProjectInfo {
  id: string;
  name: string;
  current_iteration_id: string | null;
}

interface UseIterationsDataResult {
  iterations: IterationInfo[];
  project: ProjectInfo | null;
  loading: boolean;
  executingId: string | null;
  setExecutingId: (id: string | null) => void;
  loadData: () => Promise<void>;
}

/**
 * Hook for managing iterations data and state
 * Extracts data loading logic from IterationsPanel.tsx
 */
export function useIterationsData(): UseIterationsDataResult {
  const { message } = App.useApp();
  const [iterations, setIterations] = useState<IterationInfo[]>([]);
  const [project, setProject] = useState<ProjectInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [executingId, setExecutingId] = useState<string | null>(null);

  const loadData = async () => {
    setLoading(true);
    try {
      const projectData = await invoke<ProjectInfo | null>("gui_get_project");
      setProject(projectData);

      const iterationsData = await invoke<IterationInfo[]>("gui_get_iterations");
      const sortedIterations = (iterationsData || []).sort((a, b) => {
        return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      });
      setIterations(sortedIterations);
    } catch (error) {
      console.error("[useIterationsData] Failed to load data:", error);
      message.error("Failed to load data: " + error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();

    const unlistenCreated = listen("iteration_created", () => loadData());
    const unlistenStarted = listen("iteration_started", () => {
      setTimeout(() => loadData(), 500);
    });
    const unlistenContinued = listen("iteration_continued", () => {
      setTimeout(() => loadData(), 500);
    });
    const unlistenAgentEvent = listen("agent_event", (event) => {
      const content = (event.payload as { content?: string })?.content || "";
      if (content.includes("Starting stage:")) {
        setTimeout(() => loadData(), 100);
      }
    });
    const unlistenCompleted = listen("iteration_completed", () => {
      loadData();
      setExecutingId(null);
    });
    const unlistenFailed = listen("iteration_failed", () => {
      loadData();
      setExecutingId(null);
    });

    return () => {
      unlistenCreated.then((fn) => fn()).catch(() => {});
      unlistenStarted.then((fn) => fn()).catch(() => {});
      unlistenContinued.then((fn) => fn()).catch(() => {});
      unlistenAgentEvent.then((fn) => fn()).catch(() => {});
      unlistenCompleted.then((fn) => fn()).catch(() => {});
      unlistenFailed.then((fn) => fn()).catch(() => {});
    };
  }, []);

  return {
    iterations,
    project,
    loading,
    executingId,
    setExecutingId,
    loadData
  };
}

export type { IterationInfo, ProjectInfo };
