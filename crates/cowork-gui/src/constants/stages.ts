/**
 * Stage definitions constants
 */

import type { StageDef } from '../types';

export const STAGES: StageDef[] = [
  { key: 'idea', label: 'Idea', color: '#1890ff' },
  { key: 'prd', label: 'PRD', color: '#52c41a' },
  { key: 'design', label: 'Design', color: '#722ed1' },
  { key: 'plan', label: 'Plan', color: '#fa8c16' },
  { key: 'coding', label: 'Coding', color: '#13c2c2' },
  { key: 'check', label: 'Check', color: '#eb2f96' },
  { key: 'delivery', label: 'Delivery', color: '#52c41a' },
];

export const STAGE_KEYS = {
  IDEA: 'idea',
  PRD: 'prd',
  DESIGN: 'design',
  PLAN: 'plan',
  CODING: 'coding',
  CHECK: 'check',
  DELIVERY: 'delivery',
} as const;
