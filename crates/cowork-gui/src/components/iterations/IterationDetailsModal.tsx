import { useMemo } from "react";
import { Modal, Badge, Progress, Tag } from "antd";
import type { IterationInfo, StageDef } from '../../types';
import { STAGES } from '../../constants';
import { useConfigStore } from '../../stores/configStore';

interface IterationDetailsModalProps {
  open: boolean;
  onClose: () => void;
  iteration: IterationInfo | null;
}

const getStatusColor = (status: string): "success" | "processing" | "warning" | "error" | "default" => {
  switch (status?.toLowerCase()) {
    case "completed": return "success";
    case "running": return "processing";
    case "paused": return "warning";
    case "failed": return "error";
    default: return "default";
  }
};

const getStageColor = (stageId: string): string => {
  const colors: Record<string, string> = {
    idea: '#1890ff',
    prd: '#52c41a',
    design: '#722ed1',
    plan: '#fa8c16',
    coding: '#13c2c2',
    check: '#eb2f96',
    delivery: '#52c41a',
  };
  return colors[stageId] || '#666';
};

const calculateProgress = (completedStages?: string[], totalStages?: number): number => {
  if (!completedStages) return 0;
  const total = totalStages || STAGES.length;
  return Math.round((completedStages.length / total) * 100);
};

const formatDate = (dateString?: string): string => {
  if (!dateString) return "N/A";
  return new Date(dateString).toLocaleString();
};

/**
 * Modal for displaying iteration details
 * Extracted from IterationsPanel.tsx
 */
const IterationDetailsModal: React.FC<IterationDetailsModalProps> = ({
  open,
  onClose,
  iteration
}) => {
  // Get stages from current flow configuration
  const { flows, default_flow_id } = useConfigStore();
  const currentStages: StageDef[] = useMemo(() => {
    if (default_flow_id && flows[default_flow_id]) {
      const flow = flows[default_flow_id];
      return flow.stages.map(s => ({
        key: s.stage_id,
        label: s.alias || s.stage_id,
        color: getStageColor(s.stage_id),
      }));
    }
    return STAGES;
  }, [flows, default_flow_id]);

  if (!iteration) return null;

  return (
    <Modal open={open} onCancel={onClose} footer={null} width={680}>
      <div>
        <div style={{ marginBottom: "20px", paddingBottom: "16px", borderBottom: "1px solid var(--border-color)" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "8px" }}>
            <span style={{ fontSize: "13px", fontWeight: 600, background: "var(--bg-elevated)", color: "var(--text-primary)", padding: "2px 10px", borderRadius: "4px", border: "1px solid var(--border-light)" }}>
              #{iteration.number}
            </span>
            <Badge status={getStatusColor(iteration.status)} text={iteration.status} />
          </div>
          <h2 style={{ margin: 0, fontSize: "20px", fontWeight: 600, marginBottom: "6px", color: "var(--text-primary)" }}>
            {iteration.title}
          </h2>
          <div style={{ fontSize: "12px", color: "var(--text-tertiary)", fontFamily: "monospace" }}>
            ID: {iteration.id}
          </div>
        </div>

        <div style={{ marginBottom: "16px" }}>
          <div style={{ padding: "12px", background: "var(--bg-container)", borderRadius: "6px", lineHeight: "1.5", fontSize: "14px", color: "var(--text-secondary)", border: "1px solid var(--border-light)" }}>
            {iteration.description}
          </div>
        </div>

        {iteration.base_iteration_id && (
          <div style={{ marginBottom: "16px" }}>
            <div style={{ fontSize: "13px", color: "var(--text-tertiary)", marginBottom: "4px" }}>
              Based on: <code style={{ background: "var(--bg-elevated)", padding: "2px 6px", borderRadius: "3px", fontSize: "11px", color: "var(--text-secondary)", border: "1px solid var(--border-light)" }}>
                {iteration.base_iteration_id.substring(0, 20)}...
              </code>
              <Tag style={{ marginLeft: "6px", fontSize: "11px" }}>{iteration.inheritance}</Tag>
            </div>
          </div>
        )}

        <div style={{ marginBottom: "16px" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "10px" }}>
            <span style={{ fontSize: "13px", fontWeight: 600, color: "var(--text-primary)" }}>Progress</span>
            <span style={{ fontSize: "13px", color: "var(--text-secondary)" }}>{calculateProgress(iteration.completed_stages, currentStages.length)}%</span>
          </div>
          <Progress percent={calculateProgress(iteration.completed_stages, currentStages.length)} strokeColor="var(--success)" style={{ marginBottom: "12px" }} />
          <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
            {currentStages.map((stage) => {
              const currentStageIndex = currentStages.findIndex((s) => s.key === iteration.current_stage);
              const stageIndex = currentStages.findIndex((s) => s.key === stage.key);
              const isCompleted = iteration.status === "Completed"
                ? iteration.completed_stages?.includes(stage.key)
                : currentStageIndex >= 0 && stageIndex < currentStageIndex;
              const isCurrent = iteration.current_stage === stage.key;
              return (
                <Tag
                  key={stage.key}
                  color={isCompleted ? "success" : isCurrent ? "processing" : "default"}
                  style={{ margin: 0, fontSize: "12px", borderRadius: "4px", opacity: isCompleted || isCurrent ? 1 : 0.4 }}
                >
                  {stage.label}
                </Tag>
              );
            })}
          </div>
        </div>

        <div style={{ display: "flex", gap: "24px", paddingTop: "16px", borderTop: "1px solid var(--border-color)" }}>
          <div>
            <div style={{ fontSize: "12px", color: "var(--text-tertiary)", marginBottom: "2px" }}>Created</div>
            <div style={{ fontSize: "13px", color: "var(--text-primary)" }}>{formatDate(iteration.created_at)}</div>
          </div>
          {iteration.completed_at && (
            <div>
              <div style={{ fontSize: "12px", color: "var(--text-tertiary)", marginBottom: "2px" }}>Completed</div>
              <div style={{ fontSize: "13px", color: "var(--text-primary)" }}>{formatDate(iteration.completed_at)}</div>
            </div>
          )}
        </div>
      </div>
    </Modal>
  );
};

export default IterationDetailsModal;
