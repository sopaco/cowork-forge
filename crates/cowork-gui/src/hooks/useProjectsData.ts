import { useState, useEffect } from "react";
import { App } from "antd";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface ProjectMetadata {
  session_count: number;
  technology_stack: string[];
}

interface ProjectData {
  project_id: string;
  projectId?: string;
  name: string;
  description?: string;
  status: string;
  workspacePath?: string;
  workspace_path?: string;
  last_opened_at?: string;
  metadata: ProjectMetadata;
}

interface UseProjectsDataResult {
  projects: ProjectData[];
  loading: boolean;
  loadProjects: () => Promise<void>;
}

/**
 * Hook for managing projects data and state
 * Extracts data loading logic from ProjectsPanel.tsx
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

  useEffect(() => {
    loadProjects();

    const unlistenProjectLoaded = listen("project_loaded", () => {
      loadProjects();
    });

    const unlistenProjectCreated = listen("project_created", () => {
      loadProjects();
    });

    return () => {
      unlistenProjectLoaded.then((fn) => fn()).catch(() => {});
      unlistenProjectCreated.then((fn) => fn()).catch(() => {});
    };
  }, []);

  return {
    projects,
    loading,
    loadProjects,
  };
}

export type { ProjectData, ProjectMetadata };
