import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type {
  AgentDefinition,
  StageDefinition,
  FlowDefinition,
  SkillManifest,
  IntegrationDefinition,
  ConfigRegistryState,
  ValidationResult,
  BuiltinInstruction,
} from '../types/config';

interface ConfigState extends ConfigRegistryState {
  loading: boolean;
  error: string | null;
  selectedFlow: string | null;
  selectedAgent: string | null;
  selectedStage: string | null;
  selectedSkill: string | null;
  selectedIntegration: string | null;

  // Actions
  loadConfigs: () => Promise<void>;
  resetConfigs: () => Promise<void>;
  selectFlow: (id: string | null) => void;
  selectAgent: (id: string | null) => void;
  selectStage: (id: string | null) => void;
  selectSkill: (id: string | null) => void;
  selectIntegration: (id: string | null) => void;

  // CRUD Operations
  saveAgent: (agent: AgentDefinition) => Promise<void>;
  deleteAgent: (id: string) => Promise<void>;
  saveStage: (stage: StageDefinition) => Promise<void>;
  deleteStage: (id: string) => Promise<void>;
  saveFlow: (flow: FlowDefinition) => Promise<void>;
  deleteFlow: (id: string) => Promise<void>;
  setDefaultFlow: (id: string) => Promise<void>;
  installSkill: (skillPath: string) => Promise<void>;
  uninstallSkill: (id: string) => Promise<void>;
  saveIntegration: (integration: IntegrationDefinition) => Promise<void>;
  deleteIntegration: (id: string) => Promise<void>;

  // Validation
  validateAgent: (agent: AgentDefinition) => Promise<ValidationResult>;
  validateFlow: (flow: FlowDefinition) => Promise<ValidationResult>;

  // Export/Import
  exportConfig: (type: 'agent' | 'stage' | 'flow', id: string) => Promise<string>;
  importConfig: (type: 'agent' | 'stage' | 'flow', jsonData: string) => Promise<void>;

  // Builtin Instructions
  getBuiltinInstructions: () => Promise<BuiltinInstruction[]>;
}

export const useConfigStore = create<ConfigState>((set, get) => ({
  // Initial State
  agents: {},
  stages: {},
  flows: {},
  skills: {},
  integrations: {},
  default_flow_id: null,
  loading: false,
  error: null,
  selectedFlow: null,
  selectedAgent: null,
  selectedStage: null,
  selectedSkill: null,
  selectedIntegration: null,

  // Load all configs from backend
  loadConfigs: async () => {
    set({ loading: true, error: null });
    try {
      const configState = await invoke<ConfigRegistryState>('gui_get_config_registry');
      set({
        ...configState,
        loading: false,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to load configs',
        loading: false,
      });
    }
  },

  // Reset configs to default built-in configurations
  resetConfigs: async () => {
    set({ loading: true, error: null });
    try {
      const configState = await invoke<ConfigRegistryState>('gui_reset_config_registry');
      set({
        ...configState,
        loading: false,
        selectedFlow: null,
        selectedAgent: null,
        selectedStage: null,
        selectedSkill: null,
        selectedIntegration: null,
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to reset configs',
        loading: false,
      });
      throw error;
    }
  },

  // Selection
  selectFlow: (id) => set({ selectedFlow: id }),
  selectAgent: (id) => set({ selectedAgent: id }),
  selectStage: (id) => set({ selectedStage: id }),
  selectSkill: (id) => set({ selectedSkill: id }),
  selectIntegration: (id) => set({ selectedIntegration: id }),

  // CRUD Operations
  saveAgent: async (agent) => {
    try {
      await invoke('gui_save_agent_config', { agent });
      const agents = { ...get().agents, [agent.id]: agent };
      set({ agents });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to save agent' });
      throw error;
    }
  },

  deleteAgent: async (id) => {
    try {
      await invoke('gui_delete_agent_config', { agentId: id });
      const agents = { ...get().agents };
      delete agents[id];
      set({ agents, selectedAgent: get().selectedAgent === id ? null : get().selectedAgent });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to delete agent' });
      throw error;
    }
  },

  saveStage: async (stage) => {
    try {
      await invoke('gui_save_stage_config', { stage });
      const stages = { ...get().stages, [stage.id]: stage };
      set({ stages });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to save stage' });
      throw error;
    }
  },

  deleteStage: async (id) => {
    try {
      await invoke('gui_delete_stage_config', { stageId: id });
      const stages = { ...get().stages };
      delete stages[id];
      set({ stages, selectedStage: get().selectedStage === id ? null : get().selectedStage });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to delete stage' });
      throw error;
    }
  },

  saveFlow: async (flow) => {
    try {
      await invoke('gui_save_flow_config', { flow });
      const flows = { ...get().flows, [flow.id]: flow };
      set({ flows });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to save flow' });
      throw error;
    }
  },

  deleteFlow: async (id) => {
    try {
      await invoke('gui_delete_flow_config', { flowId: id });
      const flows = { ...get().flows };
      delete flows[id];
      set({ flows, selectedFlow: get().selectedFlow === id ? null : get().selectedFlow });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to delete flow' });
      throw error;
    }
  },

  setDefaultFlow: async (id) => {
    try {
      await invoke('gui_set_default_flow', { flowId: id });
      set({ default_flow_id: id });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to set default flow' });
      throw error;
    }
  },

  installSkill: async (skillPath) => {
    try {
      const skill = await invoke<SkillManifest>('gui_install_skill', { skillPath });
      const skills = { ...get().skills, [skill.id]: skill };
      set({ skills });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to install skill' });
      throw error;
    }
  },

  uninstallSkill: async (id) => {
    try {
      await invoke('gui_uninstall_skill', { skillId: id });
      const skills = { ...get().skills };
      delete skills[id];
      set({ skills, selectedSkill: get().selectedSkill === id ? null : get().selectedSkill });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to uninstall skill' });
      throw error;
    }
  },

  saveIntegration: async (integration) => {
    try {
      await invoke('gui_save_integration_config', { integration });
      const integrations = { ...get().integrations, [integration.id]: integration };
      set({ integrations });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to save integration' });
      throw error;
    }
  },

  deleteIntegration: async (id) => {
    try {
      await invoke('gui_delete_integration_config', { integrationId: id });
      const integrations = { ...get().integrations };
      delete integrations[id];
      set({ integrations, selectedIntegration: get().selectedIntegration === id ? null : get().selectedIntegration });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to delete integration' });
      throw error;
    }
  },

  // Validation
  validateAgent: async (agent) => {
    try {
      return await invoke<ValidationResult>('gui_validate_agent_config', { agent });
    } catch (error) {
      return {
        valid: false,
        issues: [{
          path: '',
          message: error instanceof Error ? error.message : 'Validation failed',
          severity: 'error' as const,
        }],
      };
    }
  },

  validateFlow: async (flow) => {
    try {
      return await invoke<ValidationResult>('gui_validate_flow_config', { flow });
    } catch (error) {
      return {
        valid: false,
        issues: [{
          path: '',
          message: error instanceof Error ? error.message : 'Validation failed',
          severity: 'error' as const,
        }],
      };
    }
  },

  // Export/Import
  exportConfig: async (type, id) => {
    try {
      return await invoke<string>('gui_export_config', { configType: type, configId: id });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to export config' });
      throw error;
    }
  },

  importConfig: async (type, jsonData) => {
    try {
      await invoke('gui_import_config', { configType: type, jsonData });
      await get().loadConfigs();
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to import config' });
      throw error;
    }
  },

  // Builtin Instructions
  getBuiltinInstructions: async () => {
    try {
      return await invoke<BuiltinInstruction[]>('gui_get_builtin_instructions');
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to get builtin instructions' });
      return [];
    }
  },
}));
