import { create } from 'zustand';
import API from '../api';

export interface ProjectInfo {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  current_iteration_id: string | null;
}

export interface IterationInfo {
  id: string;
  number: number;
  title: string;
  description: string;
  status: string;
  current_stage: string | null;
  created_at: string;
}

export interface Iteration {
  id: string;
  number: number;
  title: string;
  description: string;
  base_iteration_id: string | null;
  inheritance: string;
  status: string;
  started_at: string | null;
  completed_at: string | null;
  current_stage: string | null;
  completed_stages: string[];
  artifacts: Record<string, string>;
  created_at: string;
}

interface ProjectState {
  project: ProjectInfo | null;
  iterations: IterationInfo[];
  currentIteration: Iteration | null;
  loading: boolean;
  
  loadProject: () => Promise<void>;
  loadIterations: () => Promise<void>;
  setCurrentIteration: (iteration: Iteration | null) => void;
  updateCurrentIterationStatus: (status: string) => void;
  clearProject: () => void;
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  project: null,
  iterations: [],
  currentIteration: null,
  loading: false,
  
  loadProject: async () => {
    try {
      set({ loading: true });
      const project = await API.project.get();
      set({ project, loading: false });
      
      if (project) {
        const iterations = await API.iteration.list();
        set({ iterations: iterations || [] });
        
        const { currentIteration } = get();
        if (currentIteration) {
          const updated = iterations?.find(i => i.id === currentIteration.id);
          if (updated) {
            const fullIteration = await API.iteration.get(updated.id);
            set({ currentIteration: fullIteration });
          }
        } else if (project.current_iteration_id) {
          const iteration = iterations?.find(i => i.id === project.current_iteration_id);
          if (iteration) {
            const fullIteration = await API.iteration.get(iteration.id);
            set({ currentIteration: fullIteration });
          }
        }
      }
    } catch (error) {
      console.error('Failed to load project:', error);
      set({ loading: false });
    }
  },
  
  loadIterations: async () => {
    try {
      const iterations = await API.iteration.list();
      set({ iterations: iterations || [] });
    } catch (error) {
      console.error('Failed to load iterations:', error);
    }
  },
  
  setCurrentIteration: (iteration) => {
    set({ currentIteration: iteration });
  },
  
  updateCurrentIterationStatus: (status) => {
    const { currentIteration } = get();
    if (currentIteration) {
      set({ currentIteration: { ...currentIteration, status } });
    }
  },
  
  clearProject: () => {
    set({
      project: null,
      iterations: [],
      currentIteration: null,
    });
  },
}));
