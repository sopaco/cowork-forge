import { create } from 'zustand';

type ViewType = 
  | 'projects' 
  | 'iterations' 
  | 'chat' 
  | 'artifacts' 
  | 'code' 
  | 'run' 
  | 'execution-memory' 
  | 'project-knowledge' 
  | 'settings';

interface UIState {
  activeView: ViewType;
  commandPaletteVisible: boolean;
  activeArtifactTab: string | null;
  artifactsRefreshTrigger: number;
  codeRefreshTrigger: number;
  memoryRefreshTrigger: number;
  knowledgeRefreshTrigger: number;
  
  setActiveView: (view: ViewType) => void;
  toggleCommandPalette: () => void;
  setCommandPaletteVisible: (visible: boolean) => void;
  setActiveArtifactTab: (tab: string | null) => void;
  triggerArtifactsRefresh: () => void;
  triggerCodeRefresh: () => void;
  triggerMemoryRefresh: () => void;
  triggerKnowledgeRefresh: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  activeView: 'projects',
  commandPaletteVisible: false,
  activeArtifactTab: null,
  artifactsRefreshTrigger: 0,
  codeRefreshTrigger: 0,
  memoryRefreshTrigger: 0,
  knowledgeRefreshTrigger: 0,
  
  setActiveView: (view) => set({ activeView: view }),
  toggleCommandPalette: () => set((state) => ({ commandPaletteVisible: !state.commandPaletteVisible })),
  setCommandPaletteVisible: (visible) => set({ commandPaletteVisible: visible }),
  setActiveArtifactTab: (tab) => set({ activeArtifactTab: tab }),
  triggerArtifactsRefresh: () => set((state) => ({ artifactsRefreshTrigger: state.artifactsRefreshTrigger + 1 })),
  triggerCodeRefresh: () => set((state) => ({ codeRefreshTrigger: state.codeRefreshTrigger + 1 })),
  triggerMemoryRefresh: () => set((state) => ({ memoryRefreshTrigger: state.memoryRefreshTrigger + 1 })),
  triggerKnowledgeRefresh: () => set((state) => ({ knowledgeRefreshTrigger: state.knowledgeRefreshTrigger + 1 })),
}));
