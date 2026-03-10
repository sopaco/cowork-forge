import { useState, useEffect, ReactElement, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  App,
  Card,
  Button,
  Modal,
  Tag,
  Empty,
  Spin,
  Progress,
  Badge,
} from "antd";
import {
  PlusOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  DeleteOutlined,
  ReloadOutlined,
  RedoOutlined,
  BranchesOutlined,
  RocketOutlined,
  CodeOutlined,
  FileTextOutlined,
  EyeOutlined,
} from "@ant-design/icons";

import { useIterationsData } from '../hooks';
import { CreateIterationModal, IterationDetailsModal, InitProjectModal } from './iterations';
import type { IterationInfo, StageDef } from '../types';
import { STAGES } from '../constants';
import { useConfigStore } from '../stores/configStore';

interface IterationsPanelProps {
  onSelectIteration?: (iterationId: string) => void;
  selectedIterationId?: string | null;
  onExecuteStatusChange?: (iterationId: string, status: string) => void;
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

const getStatusIcon = (status: string): ReactElement | null => {
  switch (status?.toLowerCase()) {
    case "completed":
      return <CheckCircleOutlined style={{ color: "#52c41a", marginRight: "5px" }} />;
    case "running":
      return <PlayCircleOutlined style={{ color: "#1890ff", marginRight: "5px" }} />;
    case "paused":
      return <PauseCircleOutlined style={{ color: "#faad14", marginRight: "5px" }} />;
    case "failed":
      return <CloseCircleOutlined style={{ color: "#ff4d4f", marginRight: "5px" }} />;
    default:
      return null;
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

const calculateProgress = (completedStages?: string[], totalStages?: number, status?: string): number => {
  // For completed iterations, always show 100%
  if (status === "Completed") return 100;
  if (!completedStages) return 0;
  const total = totalStages || STAGES.length;
  return Math.round((completedStages.length / total) * 100);
};

const formatDate = (dateString?: string): string => {
  if (!dateString) return "N/A";
  return new Date(dateString).toLocaleString();
};

const IterationsPanel: React.FC<IterationsPanelProps> = ({
  onSelectIteration,
  selectedIterationId,
  onExecuteStatusChange,
}) => {
  const { message } = App.useApp();
  const { iterations, project, loading, executingId, setExecutingId, loadData } = useIterationsData();
  
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

  // Modal states
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showDetailsModal, setShowDetailsModal] = useState(false);
  const [showInitProjectModal, setShowInitProjectModal] = useState(false);
  const [selectedIteration, setSelectedIteration] = useState<IterationInfo | null>(null);

  // Iteration actions
  const handleExecuteIteration = async (iterationId: string) => {
    try {
      setExecutingId(iterationId);
      onExecuteStatusChange?.(iterationId, "running");
      await invoke("gui_execute_iteration", { iterationId });
      message.info("Iteration execution started");
    } catch (error) {
      message.error("Failed to execute iteration: " + error);
      setExecutingId(null);
      onExecuteStatusChange?.(iterationId, "error");
    }
  };

  const handleContinueIteration = async (iterationId: string) => {
    try {
      setExecutingId(iterationId);
      onExecuteStatusChange?.(iterationId, "running");
      await invoke("gui_continue_iteration", { iterationId });
      message.info("Iteration continued");
    } catch (error) {
      message.error("Failed to continue iteration: " + error);
      setExecutingId(null);
      onExecuteStatusChange?.(iterationId, "error");
    }
  };

  const handleRetryIteration = async (iterationId: string) => {
    try {
      setExecutingId(iterationId);
      onExecuteStatusChange?.(iterationId, "running");
      await invoke("gui_retry_iteration", { iterationId });
      message.info("Retrying iteration...");
    } catch (error) {
      message.error("Failed to retry iteration: " + error);
      setExecutingId(null);
      onExecuteStatusChange?.(iterationId, "error");
    }
  };

  const handleDeleteIteration = (iteration: IterationInfo) => {
    Modal.confirm({
      title: "Delete Iteration",
      content: `Are you sure you want to delete "${iteration.title}"?`,
      okText: "Delete",
      okType: "danger",
      onOk: async () => {
        try {
          await invoke("gui_delete_iteration", { iterationId: iteration.id });
          message.success("Iteration deleted");
          loadData();
        } catch (error) {
          message.error("Failed to delete iteration: " + error);
        }
      },
    });
  };

  const handleViewDetails = (iteration: IterationInfo) => {
    setSelectedIteration(iteration);
    setShowDetailsModal(true);
  };

  // Get completed iterations for the create modal
  const completedIterations = iterations.filter((i) => i.status === "Completed");

  // Loading state
  if (loading && iterations.length === 0) {
    return (
      <div style={{ padding: "40px", textAlign: "center" }}>
        <Spin size="large" />
        <div style={{ marginTop: "16px", color: "#999" }}>Loading iterations...</div>
      </div>
    );
  }

  // No project state
  if (!project) {
    return (
      <div style={{ padding: "40px" }}>
        <Empty description="No project initialized" image={Empty.PRESENTED_IMAGE_SIMPLE}>
          <Button type="primary" onClick={() => setShowInitProjectModal(true)}>
            Initialize Project
          </Button>
        </Empty>

        <InitProjectModal
          open={showInitProjectModal}
          onClose={() => setShowInitProjectModal(false)}
          onSuccess={loadData}
        />
      </div>
    );
  }

  return (
    <div style={{ padding: "24px", height: "100%", overflow: "auto" }}>
      {/* Header */}
      <div style={{ marginBottom: "24px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <h2 style={{ margin: 0 }}>{project.name}</h2>
          <div style={{ color: "#888", fontSize: "14px", marginTop: "4px" }}>
            {iterations.length} iteration{iterations.length !== 1 ? "s" : ""}
          </div>
        </div>
        <Button type="primary" icon={<PlusOutlined />} onClick={() => setShowCreateModal(true)}>
          New Iteration
        </Button>
      </div>

      {/* Iteration list */}
      {iterations.length === 0 ? (
        <Empty description="No iterations yet" image={Empty.PRESENTED_IMAGE_SIMPLE}>
          <Button type="primary" onClick={() => setShowCreateModal(true)}>
            Create First Iteration
          </Button>
        </Empty>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
          {iterations.map((iteration) => (
            <Card
              key={iteration.id}
              hoverable
              className={selectedIterationId === iteration.id ? "selected-iteration" : ""}
              style={{ border: selectedIterationId === iteration.id ? "2px solid #1890ff" : undefined }}
              onClick={() => onSelectIteration?.(iteration.id)}
              actions={[
                iteration.status === "Draft" && (
                  <Button type="link" icon={<PlayCircleOutlined />} onClick={(e) => { e.stopPropagation(); handleExecuteIteration(iteration.id); }}>
                    Start
                  </Button>
                ),
                iteration.status === "Paused" && (
                  <Button type="link" icon={<ReloadOutlined />} onClick={(e) => { e.stopPropagation(); handleContinueIteration(iteration.id); }}>
                    Continue
                  </Button>
                ),
                iteration.status === "Failed" && (
                  <Button type="link" icon={<RedoOutlined />} onClick={(e) => { e.stopPropagation(); handleRetryIteration(iteration.id); }}>
                    Retry
                  </Button>
                ),
                iteration.status === "Running" && <Spin style={{ marginRight: "8px" }} />,
                <Button type="link" icon={<EyeOutlined />} onClick={(e) => { e.stopPropagation(); handleViewDetails(iteration); }}>
                  Details
                </Button>,
                iteration.status !== "Running" && (
                  <Button type="link" danger icon={<DeleteOutlined />} onClick={(e) => { e.stopPropagation(); handleDeleteIteration(iteration); }}>
                    Delete
                  </Button>
                ),
              ].filter(Boolean) as ReactElement[]}
            >
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                <div style={{ flex: 1 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "8px" }}>
                    <span style={{ fontSize: "18px", fontWeight: "bold" }}>#{iteration.number}</span>
                    <span style={{ fontSize: "16px" }}>{iteration.title}</span>
                    <Badge status={getStatusColor(iteration.status)} dot={false} text={<span>{getStatusIcon(iteration.status)}{iteration.status}</span>} />
                  </div>

                  <div style={{
                    color: "#666",
                    marginBottom: "12px",
                    display: "-webkit-box",
                    WebkitLineClamp: 3,
                    WebkitBoxOrient: "vertical",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    lineHeight: "1.5",
                  }}>{iteration.description}</div>

                  {iteration.base_iteration_id && (
                    <div style={{ fontSize: "12px", color: "#888", marginBottom: "8px" }}>
                      <BranchesOutlined style={{ marginRight: "4px" }} />
                      Based on iteration: {iteration.base_iteration_id.substring(0, 12)}...
                      <Tag style={{ marginLeft: "8px" }}>{iteration.inheritance}</Tag>
                    </div>
                  )}

                  <div style={{ marginTop: "12px" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
                      <span style={{ fontSize: "12px", color: "#888" }}>Progress</span>
                      <span style={{ fontSize: "12px", color: "#888" }}>{calculateProgress(iteration.completed_stages, currentStages.length, iteration.status)}%</span>
                    </div>
                    <Progress percent={calculateProgress(iteration.completed_stages, currentStages.length, iteration.status)} status={iteration.status === "Failed" ? "exception" : iteration.status === "Completed" ? "success" : "active"} />
                  </div>

                  {iteration.current_stage && (
                    <div style={{ marginTop: "8px", fontSize: "12px", color: "#1890ff" }}>
                      Current: {currentStages.find(s => s.key === iteration.current_stage)?.label || iteration.current_stage}
                    </div>
                  )}

                  <div style={{ marginTop: "12px", display: "flex", flexWrap: "wrap", gap: "4px" }}>
                    {currentStages.map((stage) => {
                      const currentStageIndex = currentStages.findIndex((s) => s.key === iteration.current_stage);
                      const stageIndex = currentStages.findIndex((s) => s.key === stage.key);
                      // For completed iterations, all stages should show as completed
                      // regardless of the completed_stages array content
                      const isCompleted = iteration.status === "Completed"
                        ? true
                        : iteration.completed_stages?.includes(stage.key) ?? (currentStageIndex >= 0 && stageIndex < currentStageIndex);
                      const isCurrent = iteration.current_stage === stage.key;

                      return (
                        <Tag key={stage.key} color={isCompleted ? "success" : isCurrent ? "processing" : "default"} style={{ opacity: isCompleted || isCurrent ? 1 : 0.5 }}>
                          {isCompleted && <CheckCircleOutlined style={{ marginRight: "4px" }} />}
                          {stage.label}
                        </Tag>
                      );
                    })}
                  </div>
                </div>

                <div style={{ textAlign: "right", color: "#888", fontSize: "12px" }}>
                  <div>Created</div>
                  <div>{formatDate(iteration.created_at)}</div>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Modals */}
      <CreateIterationModal
        open={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onSuccess={loadData}
        completedIterations={completedIterations}
      />

      <IterationDetailsModal
        open={showDetailsModal}
        onClose={() => setShowDetailsModal(false)}
        iteration={selectedIteration}
      />
    </div>
  );
};

export default IterationsPanel;
