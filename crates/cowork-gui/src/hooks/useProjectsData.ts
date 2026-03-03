import { useState, useEffect } from "react";
import { App } from "antd";
import { invoke } from "@tauri-apps/api/core";
import type { ProjectData } from '../types';
import { useProjectEvents } from './useProjectEvents';

interface UseProjectsDataResult {
  projects: ProjectData[];
  loading: boolean;
  loadProjects: () => Promise<void>;
}

/**
 * Hook for managing projects data and state
 * Data fetching only - event handling separated to useProjectEvents
 */
export function useProjectsData(): UseProjectsDataResult {
  const { message } = App.useApp();
  const [projects, setProjects] = useState<ProjectData[]>([]);
  const [loading, setLoading] = useState(false);

  const loadProjects = async () => {
    setLoading(true);
    try {
      const data = await invoke<ProjectData[]>("get_all_projects", {
        status: null,
        search: null,
        limit: null,
      });
      setProjects(data || []);
    } catch (error) {
      console.error("[useProjectsData] Failed to load projects:", error);
      message.error("Failed to load projects: " + error);
    } finally {
      setLoading(false);
    }
  };

  // Initial data load
  useEffect(() => {
    loadProjects();
  }, []);

  // Listen to project events and refresh data
  useProjectEvents({
    onProjectLoaded: loadProjects,
    onProjectCreated: loadProjects,
  });

  return {
    projects,
    loading,
    loadProjects,
  };
}
