/**
 * Status color mappings
 */

export const ITERATION_STATUS_COLORS = {
  completed: 'success',
  running: 'processing',
  paused: 'warning',
  failed: 'error',
  draft: 'default',
} as const;

export const PROJECT_STATUS_COLORS = {
  active: 'green',
  archived: 'default',
  deleted: 'red',
} as const;

export type IterationStatusColor = typeof ITERATION_STATUS_COLORS[keyof typeof ITERATION_STATUS_COLORS];
export type ProjectStatusColor = typeof PROJECT_STATUS_COLORS[keyof typeof PROJECT_STATUS_COLORS];
