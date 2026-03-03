import { Modal, Badge, Progress, Tag } from "antd";
import { ReactElement } from "react";

interface StageDef {
  key: string;
  label: string;
  icon: ReactElement;
  color: string;
}

const STAGES: StageDef[] = [
  { key: "idea", label: "Idea", icon: null as unknown as ReactElement, color: "#1890ff" },
  { key: "prd", label: "PRD", icon: null as unknown as ReactElement, color: "#52c41a" },
  { key: "design", label: "Design", icon: null as unknown as ReactElement, color: "#722ed1" },
  { key: "plan", label: "Plan", icon: null as unknown as ReactElement, color: "#fa8c16" },
  { key: "coding", label: "Coding", icon: null as unknown as ReactElement, color: "#13c2c2" },
  { key: "check", label: "Check", icon: null as unknown as ReactElement, color: "#eb2f96" },
  { key: "delivery", label: "Delivery", icon: null as unknown as ReactElement, color: "#52c41a" },
];

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

const calculateProgress = (completedStages?: string[]): number => {
  if (!completedStages) return 0;
  return Math.round((completedStages.length / STAGES.length) * 100);
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
  if (!iteration) return null;

  return (
    <Modal open={open} onCancel={onClose} footer={null} width={680}>
      <div>
        <div style={{ marginBottom: "20px", paddingBottom: "16px", borderBottom: "1px solid #e8e8e8" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "8px" }}>
            <span style={{ fontSize: "13px", fontWeight: 600, background: "#f0f0f0", padding: "2px 10px", borderRadius: "4px" }}>
              #{iteration.number}
            </span>
            <Badge status={getStatusColor(iteration.status)} text={iteration.status} />
          </div>
          <h2 style={{ margin: 0, fontSize: "20px", fontWeight: 600, marginBottom: "6px" }}>
            {iteration.title}
          </h2>
          <div style={{ fontSize: "12px", color: "#999", fontFamily: "monospace" }}>
            ID: {iteration.id}
          </div>
        </div>

        <div style={{ marginBottom: "16px" }}>
          <div style={{ padding: "12px", background: "#fafafa", borderRadius: "6px", lineHeight: "1.5", fontSize: "14px", color: "#666" }}>
            {iteration.description}
          </div>
        </div>

        {iteration.base_iteration_id && (
          <div style={{ marginBottom: "16px" }}>
            <div style={{ fontSize: "13px", color: "#999", marginBottom: "4px" }}>
              Based on: <code style={{ background: "#f5f5f5", padding: "2px 6px", borderRadius: "3px", fontSize: "11px" }}>
                {iteration.base_iteration_id.substring(0, 20)}...
              </code>
              <Tag style={{ marginLeft: "6px", fontSize: "11px" }}>{iteration.inheritance}</Tag>
            </div>
          </div>
        )}

        <div style={{ marginBottom: "16px" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "10px" }}>
            <span style={{ fontSize: "13px", fontWeight: 600 }}>Progress</span>
            <span style={{ fontSize: "13px", color: "#666" }}>{calculateProgress(iteration.completed_stages)}%</span>
          </div>
          <Progress percent={calculateProgress(iteration.completed_stages)} strokeColor="#52c41a" style={{ marginBottom: "12px" }} />
          <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
            {STAGES.map((stage) => {
              const currentStageIndex = STAGES.findIndex((s) => s.key === iteration.current_stage);
              const stageIndex = STAGES.findIndex((s) => s.key === stage.key);
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

        <div style={{ display: "flex", gap: "24px", paddingTop: "16px", borderTop: "1px solid #e8e8e8" }}>
          <div>
            <div style={{ fontSize: "12px", color: "#999", marginBottom: "2px" }}>Created</div>
            <div style={{ fontSize: "13px", color: "#333" }}>{formatDate(iteration.created_at)}</div>
          </div>
          {iteration.completed_at && (
            <div>
              <div style={{ fontSize: "12px", color: "#999", marginBottom: "2px" }}>Completed</div>
              <div style={{ fontSize: "13px", color: "#333" }}>{formatDate(iteration.completed_at)}</div>
            </div>
          )}
        </div>
      </div>
    </Modal>
  );
};

export default IterationDetailsModal;
