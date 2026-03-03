import { useState, useEffect } from "react";
import { App } from "antd";
import { invoke } from "@tauri-apps/api/core";
import type { IterationInfo, ProjectInfo } from '../types';
import { useIterationEvents } from './useIterationEvents';

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
 * Data fetching only - event handling separated to useIterationEvents
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

  // Initial data load
  useEffect(() => {
    loadData();
  }, []);

  // Listen to iteration events and refresh data
  useIterationEvents({
    onIterationCreated: loadData,
    onIterationStarted: loadData,
    onIterationContinued: loadData,
    onIterationCompleted: loadData,
    onIterationFailed: loadData,
    onAgentEvent: loadData,
    onExecutingIdChange: setExecutingId,
  });

  return {
    iterations,
    project,
    loading,
    executingId,
    setExecutingId,
    loadData
  };
}
