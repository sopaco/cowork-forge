import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  App,
  Card,
  Button,
  Modal,
  Tag,
  Empty,
  Spin,
  Tooltip,
  Space,
} from "antd";
import {
  FolderOpenOutlined,
  DeleteOutlined,
  EditOutlined,
  CheckCircleOutlined,
  ClockCircleOutlined,
  PlusOutlined,
  ImportOutlined,
} from "@ant-design/icons";

import { useProjectsData } from '../hooks';
import { CreateProjectModal, EditProjectModal, ImportProjectModal } from './projects';
import type { ProjectData } from '../types';

const ProjectsPanel: React.FC = () => {
  const { message } = App.useApp();
  const { projects, loading, loadProjects } = useProjectsData();

  // Modal states
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);
  const [selectedProject, setSelectedProject] = useState<ProjectData | null>(null);

  // Project actions
  const handleDeleteProject = async (project: ProjectData) => {
    Modal.confirm({
      title: "Delete Project",
      content: `Remove "${project.name}" from project list? The project files will remain on disk.`,
      okText: "Delete",
      okType: "danger",
      onOk: async () => {
        try {
          await invoke("delete_project", { projectId: project.project_id });
          message.success("Project removed from list");
          loadProjects();
        } catch (error) {
          message.error("Failed to delete project: " + error);
        }
      },
    });
  };

  const handleOpenProject = async (projectId: string) => {
    try {
      const hasProject = await invoke<boolean>("has_open_project");

      if (hasProject) {
        await invoke("open_project", { projectId });
        message.info("Opening project in new window...");
      } else {
        await invoke("open_project_in_current_window", { projectId });
        message.success("Project opened successfully");
      }
    } catch (error) {
      message.error("Failed to open project: " + error);
    }
  };

  const handleOpenEditModal = (project: ProjectData) => {
    setSelectedProject(project);
    setShowEditModal(true);
  };

  const handleProjectCreated = async (projectId: string, projectName: string) => {
    loadProjects();

    // Ask if user wants to open the project
    Modal.confirm({
      title: "Open Project?",
      content: `Would you like to open "${projectName}" now?`,
      okText: "Open Project",
      cancelText: "Later",
      onOk: async () => {
        try {
          const hasProject = await invoke<boolean>("has_open_project");
          if (hasProject) {
            await invoke("open_project", { projectId });
            message.info("Opening project in new window...");
          } else {
            await invoke("open_project_in_current_window", { projectId });
            message.success("Project opened successfully");
          }
        } catch (error) {
          message.error("Failed to open project: " + error);
        }
      },
    });
  };

  const handleProjectImported = async (projectId: string, projectName: string) => {
    loadProjects();
    message.success(`Project "${projectName}" imported successfully!`);

    // Ask if user wants to open the project
    Modal.confirm({
      title: "Open Project?",
      content: `Would you like to open "${projectName}" now?`,
      okText: "Open Project",
      cancelText: "Later",
      onOk: async () => {
        try {
          const hasProject = await invoke<boolean>("has_open_project");
          if (hasProject) {
            await invoke("open_project", { projectId });
            message.info("Opening project in new window...");
          } else {
            await invoke("open_project_in_current_window", { projectId });
            message.success("Project opened successfully");
          }
        } catch (error) {
          message.error("Failed to open project: " + error);
        }
      },
    });
  };

  // Utility functions
  const formatDate = (dateString?: string): string => {
    if (!dateString) return "Never";
    const date = new Date(dateString);
    return date.toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const getDisplayPath = (fullPath?: string): string => {
    if (!fullPath) return "No path";
    const parts = fullPath.split(/[/\\]/);
    if (parts.length >= 2) {
      return ".../" + parts.slice(-2).join("/");
    }
    return fullPath;
  };

  const getStatusColor = (status: string): "green" | "default" | "red" => {
    switch (status) {
      case "active":
        return "green";
      case "archived":
        return "default";
      case "deleted":
        return "red";
      default:
        return "default";
    }
  };

  // Render
  if (loading) {
    return (
      <div style={{ textAlign: "center", padding: "60px 0" }}>
        <Spin size="large" />
        <div style={{ marginTop: "16px", color: "#999" }}>Loading projects...</div>
      </div>
    );
  }

  return (
    <div style={{ padding: "24px" }}>
      {/* Header */}
      <div
        style={{
          marginBottom: "24px",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <h2 style={{ margin: 0 }}>Projects</h2>
        <Space>
          <Button icon={<ImportOutlined />} onClick={() => setShowImportModal(true)}>
            Import Project
          </Button>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setShowCreateModal(true)}>
            New Project
          </Button>
        </Space>
      </div>

      {/* Project list */}
      {projects.length === 0 ? (
        <Empty description="No projects yet" image={Empty.PRESENTED_IMAGE_SIMPLE}>
          <Space direction="vertical">
            <Button type="primary" icon={<PlusOutlined />} onClick={() => setShowCreateModal(true)}>
              Create Your First Project
            </Button>
            <Button icon={<ImportOutlined />} onClick={() => setShowImportModal(true)}>
              Import Existing Project
            </Button>
          </Space>
        </Empty>
      ) : (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "repeat(auto-fill, minmax(350px, 1fr))",
            gap: "20px",
          }}
        >
          {projects.map((project) => (
            <Card
              key={project.project_id}
              hoverable
              actions={[
                <Button
                  type="link"
                  icon={<FolderOpenOutlined />}
                  onClick={() => handleOpenProject(project.project_id || project.projectId || "")}
                  style={{ color: "#1890ff", width: "90%" }}
                >
                  Open
                </Button>,
                <Button
                  type="link"
                  icon={<EditOutlined />}
                  style={{ color: "#1890ff", width: "90%" }}
                  onClick={() => handleOpenEditModal(project)}
                >
                  Edit
                </Button>,
                <Button
                  type="link"
                  danger
                  icon={<DeleteOutlined />}
                  style={{ width: "90%" }}
                  onClick={() => handleDeleteProject(project)}
                >
                  Delete
                </Button>,
              ]}
            >
              <Card.Meta
                title={
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <span>{project.name}</span>
                    <Tag color={getStatusColor(project.status)} style={{ margin: 0 }}>
                      {project.status}
                    </Tag>
                  </div>
                }
                description={
                  <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                    <Tooltip title={project.description || "No description provided"}>
                      <div
                        style={{
                          color: "#666",
                          fontSize: "14px",
                          whiteSpace: "nowrap",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          cursor: "help",
                        }}
                      >
                        {project.description || "No description provided"}
                      </div>
                    </Tooltip>
                    <div style={{ fontSize: "12px", color: "#999", minWidth: 0 }}>
                      <Tooltip title={project.workspace_path || project.workspacePath || "No path"}>
                        <span
                          style={{
                            display: "flex",
                            alignItems: "center",
                            maxWidth: "100%",
                            overflow: "hidden",
                            textOverflow: "ellipsis",
                            whiteSpace: "nowrap",
                          }}
                        >
                          <FolderOpenOutlined style={{ marginRight: "4px", flexShrink: 0 }} />
                          {getDisplayPath(project.workspace_path || project.workspacePath)}
                        </span>
                      </Tooltip>
                    </div>
                    <div style={{ display: "flex", gap: "16px", fontSize: "12px", color: "#999" }}>
                      <span>
                        <CheckCircleOutlined style={{ marginRight: "4px" }} />
                        {project.metadata?.session_count || 0} sessions
                      </span>
                      <span>
                        <ClockCircleOutlined style={{ marginRight: "4px" }} />
                        Last opened: {formatDate(project.last_opened_at)}
                      </span>
                    </div>
                    {project.metadata?.technology_stack?.length > 0 && (
                      <div style={{ display: "flex", flexWrap: "wrap", gap: "4px" }}>
                        {project.metadata.technology_stack.slice(0, 4).map((tech, idx) => (
                          <Tag key={idx} color="blue">
                            {tech}
                          </Tag>
                        ))}
                        {project.metadata.technology_stack.length > 4 && (
                          <Tag color="default">+{project.metadata.technology_stack.length - 4}</Tag>
                        )}
                      </div>
                    )}
                  </div>
                }
              />
            </Card>
          ))}
        </div>
      )}

      {/* Modals */}
      <CreateProjectModal
        open={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onSuccess={handleProjectCreated}
      />

      <EditProjectModal
        open={showEditModal}
        onClose={() => {
          setShowEditModal(false);
          setSelectedProject(null);
        }}
        onSuccess={loadProjects}
        project={selectedProject}
      />

      <ImportProjectModal
        open={showImportModal}
        onClose={() => setShowImportModal(false)}
        onSuccess={handleProjectImported}
      />
    </div>
  );
};

export default ProjectsPanel;
