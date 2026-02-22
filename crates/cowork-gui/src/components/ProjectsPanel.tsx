import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Card,
  Button,
  Modal,
  Input,
  Tag,
  Empty,
  message,
  Spin,
  Space,
  Tooltip,
  Divider,
  Alert,
  Typography,
} from "antd";
import {
  FolderOpenOutlined,
  DeleteOutlined,
  EditOutlined,
  CheckCircleOutlined,
  ClockCircleOutlined,
  PlusOutlined,
  FolderOutlined,
  InfoCircleOutlined,
  RocketOutlined,
  FolderAddOutlined,
} from "@ant-design/icons";

const { Text, Paragraph } = Typography;

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

const ProjectsPanel: React.FC = () => {
  const [projects, setProjects] = useState<ProjectData[]>([]);
  const [loading, setLoading] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [selectedProject, setSelectedProject] = useState<ProjectData | null>(null);
  const [editProjectName, setEditProjectName] = useState("");
  const [editProjectDescription, setEditProjectDescription] = useState("");
  
  // Create project modal state
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newProjectPath, setNewProjectPath] = useState("");
  const [newProjectName, setNewProjectName] = useState("");
  const [newProjectDescription, setNewProjectDescription] = useState("");
  const [createLoading, setCreateLoading] = useState(false);
  const [pathExists, setPathExists] = useState<boolean | null>(null);
  const [checkingPath, setCheckingPath] = useState(false);

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
      console.error("[ProjectsPanel] Failed to load projects:", error);
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

  // Check if path exists when path changes
  useEffect(() => {
    if (!newProjectPath.trim()) {
      setPathExists(null);
      return;
    }

    const checkPath = async () => {
      setCheckingPath(true);
      try {
        const exists = await invoke<boolean>("path_exists", { path: newProjectPath });
        setPathExists(exists);
      } catch {
        setPathExists(false);
      } finally {
        setCheckingPath(false);
      }
    };

    const timer = setTimeout(checkPath, 300);
    return () => clearTimeout(timer);
  }, [newProjectPath]);

  // Auto-fill project name from path
  useEffect(() => {
    if (newProjectPath && !newProjectName) {
      const parts = newProjectPath.split(/[/\\]/);
      const folderName = parts[parts.length - 1] || parts[parts.length - 2] || "";
      if (folderName) {
        setNewProjectName(folderName);
      }
    }
  }, [newProjectPath]);

  const handleEditProject = async () => {
    if (!editProjectName.trim()) {
      message.warning("Please enter project name");
      return;
    }

    try {
      await invoke("update_project", {
        projectId: selectedProject?.project_id,
        name: editProjectName,
        description: editProjectDescription || null,
        status: null,
      });
      message.success("Project updated successfully");
      setShowEditModal(false);
      setSelectedProject(null);
      setEditProjectName("");
      setEditProjectDescription("");
      loadProjects();
    } catch (error) {
      message.error("Failed to update project: " + error);
    }
  };

  const handleDeleteProject = async (project: ProjectData) => {
    Modal.confirm({
      title: "Delete Project",
      content: `Are you sure you want to delete "${project.name}"?`,
      okText: "Delete Record Only",
      okType: "default",
      cancelText: "Delete All (Files & Record)",
      onOk: async () => {
        try {
          const currentWorkspace = await invoke<string | null>("get_workspace");
          const isCurrentWindowProject = currentWorkspace === (project.workspacePath || project.workspace_path);

          await invoke("delete_project", { projectId: project.project_id, deleteFiles: false });
          message.success("Project record deleted");
          loadProjects();

          if (isCurrentWindowProject) {
            message.info("Project was open in current window. Workspace cleared.");
            window.location.reload();
          }
        } catch (error) {
          message.error("Failed to delete project: " + error);
        }
      },
      onCancel: async () => {
        try {
          const currentWorkspace = await invoke<string | null>("get_workspace");
          const isCurrentWindowProject = currentWorkspace === (project.workspacePath || project.workspace_path);

          await invoke("delete_project", { projectId: project.project_id, deleteFiles: true });
          message.success("Project deleted (files and record)");
          loadProjects();

          if (isCurrentWindowProject) {
            message.info("Project was open in current window. Workspace cleared.");
            window.location.reload();
          }
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
    setEditProjectName(project.name);
    setEditProjectDescription(project.description || "");
    setShowEditModal(true);
  };

  const handleSelectDirectory = async () => {
    try {
      const selected = await open({ directory: true, multiple: false, title: "Select Project Directory" });
      if (selected && typeof selected === "string") {
        setNewProjectPath(selected);
      }
    } catch (error) {
      console.error("Failed to open directory dialog:", error);
      message.error("Failed to open directory dialog: " + error);
    }
  };

  const handleCreateProject = async () => {
    if (!newProjectPath.trim()) {
      message.warning("Please select or enter a project directory");
      return;
    }

    if (!newProjectName.trim()) {
      message.warning("Please enter a project name");
      return;
    }

    setCreateLoading(true);
    try {
      // Create project with the specified path
      const result = await invoke<{ project_id: string; created_dir: boolean }>("create_project_at_path", {
        path: newProjectPath,
        name: newProjectName,
        description: newProjectDescription || null,
      });

      if (result.created_dir) {
        message.success(`Project created and directory initialized: ${newProjectName}`);
      } else {
        message.success(`Project created: ${newProjectName}`);
      }

      setShowCreateModal(false);
      resetCreateForm();
      loadProjects();

      // Ask if user wants to open the project
      Modal.confirm({
        title: "Open Project?",
        content: `Would you like to open "${newProjectName}" now?`,
        okText: "Open Project",
        cancelText: "Later",
        onOk: async () => {
          try {
            const hasProject = await invoke<boolean>("has_open_project");
            if (hasProject) {
              await invoke("open_project", { projectId: result.project_id });
              message.info("Opening project in new window...");
            } else {
              await invoke("open_project_in_current_window", { projectId: result.project_id });
              message.success("Project opened successfully");
            }
          } catch (error) {
            message.error("Failed to open project: " + error);
          }
        },
      });
    } catch (error) {
      message.error("Failed to create project: " + error);
    } finally {
      setCreateLoading(false);
    }
  };

  const resetCreateForm = () => {
    setNewProjectPath("");
    setNewProjectName("");
    setNewProjectDescription("");
    setPathExists(null);
  };

  const formatDate = (dateString?: string): string => {
    if (!dateString) return "Never";
    const date = new Date(dateString);
    return date.toLocaleDateString("en-US", { year: "numeric", month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
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
      case "active": return "green";
      case "archived": return "default";
      case "deleted": return "red";
      default: return "default";
    }
  };

  return (
    <div style={{ padding: "24px" }}>
      <div style={{ marginBottom: "24px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h2 style={{ margin: 0 }}>Projects</h2>
        <Button type="primary" icon={<PlusOutlined />} onClick={() => setShowCreateModal(true)}>New Project</Button>
      </div>

      {loading ? (
        <div style={{ textAlign: "center", padding: "60px 0" }}>
          <Spin size="large" />
          <div style={{ marginTop: "16px", color: "#999" }}>Loading projects...</div>
        </div>
      ) : projects.length === 0 ? (
        <Empty description="No projects yet" image={Empty.PRESENTED_IMAGE_SIMPLE}>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setShowCreateModal(true)}>Create Your First Project</Button>
        </Empty>
      ) : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(350px, 1fr))", gap: "20px" }}>
          {projects.map((project) => (
            <Card
              key={project.project_id}
              hoverable
              actions={[
                <Button type="link" icon={<FolderOpenOutlined />} onClick={() => handleOpenProject(project.project_id || project.projectId || "")} style={{ color: "#1890ff", width: "90%" }}>Open</Button>,
                <Button type="link" icon={<EditOutlined />} style={{ color: "#1890ff", width: "90%" }} onClick={() => handleOpenEditModal(project)}>Edit</Button>,
                <Button type="link" danger icon={<DeleteOutlined />} style={{ width: "90%" }} onClick={() => handleDeleteProject(project)}>Delete</Button>,
              ]}
            >
              <Card.Meta
                title={
                  <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <span>{project.name}</span>
                    <Tag color={getStatusColor(project.status)} style={{ margin: 0 }}>{project.status}</Tag>
                  </div>
                }
                description={
                  <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                    <Tooltip title={project.description || "No description provided"}>
                      <div style={{ color: "#666", fontSize: "14px", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", cursor: "help" }}>
                        {project.description || "No description provided"}
                      </div>
                    </Tooltip>
                    <div style={{ fontSize: "12px", color: "#999", minWidth: 0 }}>
                      <Tooltip title={project.workspace_path || project.workspacePath || "No path"}>
                        <span style={{ display: "flex", alignItems: "center", maxWidth: "100%", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                          <FolderOpenOutlined style={{ marginRight: "4px", flexShrink: 0 }} />
                          {getDisplayPath(project.workspace_path || project.workspacePath)}
                        </span>
                      </Tooltip>
                    </div>
                    <div style={{ display: "flex", gap: "16px", fontSize: "12px", color: "#999" }}>
                      <span><CheckCircleOutlined style={{ marginRight: "4px" }} />{project.metadata?.session_count || 0} sessions</span>
                      <span><ClockCircleOutlined style={{ marginRight: "4px" }} />Last opened: {formatDate(project.last_opened_at)}</span>
                    </div>
                    {project.metadata?.technology_stack?.length > 0 && (
                      <div style={{ display: "flex", flexWrap: "wrap", gap: "4px" }}>
                        {project.metadata.technology_stack.slice(0, 4).map((tech, idx) => (
                          <Tag key={idx} color="blue">{tech}</Tag>
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

      {/* Create Project Modal */}
      <Modal
        title={
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <FolderAddOutlined />
            <span>Create New Project</span>
          </div>
        }
        open={showCreateModal}
        onOk={handleCreateProject}
        onCancel={() => { setShowCreateModal(false); resetCreateForm(); }}
        okText="Create Project"
        cancelText="Cancel"
        confirmLoading={createLoading}
        width={600}
        okButtonProps={{ disabled: !newProjectPath.trim() || !newProjectName.trim() }}
      >
        <div style={{ marginBottom: "16px" }}>
          <Alert
            type="info"
            showIcon
            icon={<RocketOutlined />}
            message="Quick Start Guide"
            description={
              <div style={{ fontSize: "13px" }}>
                <Paragraph style={{ margin: 0, marginBottom: "8px" }}>
                  1. <strong>Select a folder</strong> - Choose where your project will live
                </Paragraph>
                <Paragraph style={{ margin: 0, marginBottom: "8px" }}>
                  2. <strong>Name your project</strong> - Give it a meaningful name
                </Paragraph>
                <Paragraph style={{ margin: 0 }}>
                  3. <strong>Start building</strong> - The folder will be created if it doesn't exist
                </Paragraph>
              </div>
            }
          />
        </div>

        <Divider style={{ margin: "12px 0" }} />

        <div style={{ marginBottom: "16px" }}>
          <label style={{ display: "block", marginBottom: "8px", fontWeight: 500 }}>
            <FolderOutlined style={{ marginRight: "6px" }} />
            Project Directory <Text type="danger">*</Text>
          </label>
          <Space.Compact style={{ width: "100%" }}>
            <Input
              value={newProjectPath}
              onChange={(e) => setNewProjectPath(e.target.value)}
              placeholder="e.g., D:\Projects\my-awesome-project"
              autoFocus
            />
            <Button icon={<FolderOpenOutlined />} onClick={handleSelectDirectory}>Browse</Button>
          </Space.Compact>
          <div style={{ marginTop: "6px", minHeight: "22px" }}>
            {checkingPath && <Spin size="small" />}
            {!checkingPath && pathExists === true && (
              <Text type="success">
                <CheckCircleOutlined style={{ marginRight: "4px" }} />
                Directory exists - will use existing folder
              </Text>
            )}
            {!checkingPath && pathExists === false && newProjectPath.trim() && (
              <Text type="warning">
                <FolderAddOutlined style={{ marginRight: "4px" }} />
                Directory doesn't exist - will be created automatically
              </Text>
            )}
          </div>
        </div>

        <div style={{ marginBottom: "16px" }}>
          <label style={{ display: "block", marginBottom: "8px", fontWeight: 500 }}>
            Project Name <Text type="danger">*</Text>
          </label>
          <Input
            value={newProjectName}
            onChange={(e) => setNewProjectName(e.target.value)}
            placeholder="My Awesome Project"
          />
        </div>

        <div>
          <label style={{ display: "block", marginBottom: "8px", fontWeight: 500 }}>
            Description <Text type="secondary">(Optional)</Text>
          </label>
          <Input.TextArea
            value={newProjectDescription}
            onChange={(e) => setNewProjectDescription(e.target.value)}
            placeholder="Brief description of what this project is about..."
            rows={2}
          />
        </div>
      </Modal>

      {/* Edit Project Modal */}
      <Modal
        title="Edit Project"
        open={showEditModal}
        onOk={handleEditProject}
        onCancel={() => { setShowEditModal(false); setSelectedProject(null); setEditProjectName(""); setEditProjectDescription(""); }}
        okText="Save"
        cancelText="Cancel"
      >
        <div style={{ marginBottom: "16px" }}>
          <label style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}>Project Name:</label>
          <Input value={editProjectName} onChange={(e) => setEditProjectName(e.target.value)} placeholder="Project name" autoFocus />
        </div>
        <div>
          <label style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}>Description:</label>
          <Input.TextArea value={editProjectDescription} onChange={(e) => setEditProjectDescription(e.target.value)} placeholder="Project description" rows={4} />
        </div>
      </Modal>
    </div>
  );
};

export default ProjectsPanel;
