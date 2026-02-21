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
} from "antd";
import {
  FolderOpenOutlined,
  DeleteOutlined,
  EditOutlined,
  CheckCircleOutlined,
  ClockCircleOutlined,
  PlusOutlined,
  FolderOutlined,
} from "@ant-design/icons";

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
  const [showOpenDirModal, setShowOpenDirModal] = useState(false);
  const [selectedProject, setSelectedProject] = useState<ProjectData | null>(null);
  const [openDirPath, setOpenDirPath] = useState("");
  const [editProjectName, setEditProjectName] = useState("");
  const [editProjectDescription, setEditProjectDescription] = useState("");
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newProjectName, setNewProjectName] = useState("");

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
      console.log("[ProjectsPanel] Project loaded event received, reloading projects...");
      loadProjects();
    });

    const unlistenProjectCreated = listen("project_created", () => {
      console.log("[ProjectsPanel] Project created event received, reloading projects...");
      loadProjects();
    });

    return () => {
      unlistenProjectLoaded.then((fn) => fn()).catch((e) => console.error("[ProjectsPanel] Failed to unlisten project_loaded:", e));
      unlistenProjectCreated.then((fn) => fn()).catch((e) => console.error("[ProjectsPanel] Failed to unlisten project_created:", e));
    };
  }, []);

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

  const handleOpenDirectory = async () => {
    if (!openDirPath.trim()) {
      message.warning("Please enter a directory path");
      return;
    }

    try {
      const hasProject = await invoke<boolean>("has_open_project");

      if (hasProject) {
        const projectName = openDirPath.split(/[/\\]/).pop() || openDirPath;
        await invoke("register_project", {
          workspacePath: openDirPath,
          name: projectName,
          description: `Project at ${openDirPath}`,
        });

        const allProjects = await invoke<ProjectData[]>("get_all_projects", { status: null, search: null, limit: null });
        const newProject = allProjects.find((p) => p.workspacePath === openDirPath || p.workspace_path === openDirPath);

        if (newProject) {
          await invoke("open_project", { projectId: newProject.project_id });
          message.success("Opening project in new window...");
        } else {
          message.error("Failed to find newly registered project");
        }

        setShowOpenDirModal(false);
        setOpenDirPath("");
        loadProjects();
      } else {
        await invoke("set_workspace", { workspacePath: openDirPath });
        message.success("Workspace set successfully");
        setShowOpenDirModal(false);
        setOpenDirPath("");
      }
    } catch (error) {
      message.error("Failed to open directory: " + error);
    }
  };

  const handleCreateProject = async () => {
    if (!newProjectName.trim()) {
      message.warning("Please enter a project name");
      return;
    }

    try {
      const project = await invoke<{ name: string }>("gui_init_project", { name: newProjectName });
      message.success(`Project created: ${project.name}`);
      setShowCreateModal(false);
      setNewProjectName("");
      loadProjects();
    } catch (error) {
      message.error("Failed to create project: " + error);
    }
  };

  const handleSelectDirectory = async () => {
    try {
      const selected = await open({ directory: true, multiple: false, title: "Select Directory" });
      if (selected && typeof selected === "string") {
        setOpenDirPath(selected);
      }
    } catch (error) {
      console.error("Failed to open directory dialog:", error);
      message.error("Failed to open directory dialog: " + error);
    }
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
        <Space>
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setShowCreateModal(true)}>Create Project</Button>
          <Button icon={<FolderOpenOutlined />} onClick={() => setShowOpenDirModal(true)}>Open Directory</Button>
        </Space>
      </div>

      {loading ? (
        <div style={{ textAlign: "center", padding: "60px 0" }}>
          <Spin size="large" />
          <div style={{ marginTop: "16px", color: "#999" }}>Loading projects...</div>
        </div>
      ) : projects.length === 0 ? (
        <Empty description="No projects yet" image={Empty.PRESENTED_IMAGE_SIMPLE}>
          <Button type="primary" icon={<FolderOpenOutlined />} onClick={() => setShowOpenDirModal(true)}>Open Directory to Start</Button>
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
                          <Tag key={idx}  color="blue">{tech}</Tag>
                        ))}
                        {project.metadata.technology_stack.length > 4 && (
                          <Tag  color="default">+{project.metadata.technology_stack.length - 4}</Tag>
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

      <Modal title="Create New Project" open={showCreateModal} onOk={handleCreateProject} onCancel={() => { setShowCreateModal(false); setNewProjectName(""); }} okText="Create" cancelText="Cancel">
        <div>
          <label style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}>Project Name:</label>
          <Input value={newProjectName} onChange={(e) => setNewProjectName(e.target.value)} placeholder="Enter project name" autoFocus onPressEnter={handleCreateProject} />
          <div style={{ marginTop: "8px", fontSize: "12px", color: "#888" }}>
            <FolderOutlined style={{ marginRight: "4px" }} />
            Project will be created in the current workspace directory
          </div>
        </div>
      </Modal>

      <Modal title="Edit Project" open={showEditModal} onOk={handleEditProject} onCancel={() => { setShowEditModal(false); setSelectedProject(null); setEditProjectName(""); setEditProjectDescription(""); }} okText="Save" cancelText="Cancel">
        <div style={{ marginBottom: "16px" }}>
          <label style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}>Project Name:</label>
          <Input value={editProjectName} onChange={(e) => setEditProjectName(e.target.value)} placeholder="Project name" autoFocus />
        </div>
        <div>
          <label style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}>Description:</label>
          <Input.TextArea value={editProjectDescription} onChange={(e) => setEditProjectDescription(e.target.value)} placeholder="Project description" rows={4} />
        </div>
      </Modal>

      <Modal title="Open Directory" open={showOpenDirModal} onOk={handleOpenDirectory} onCancel={() => { setShowOpenDirModal(false); setOpenDirPath(""); }} okText="Open" cancelText="Cancel">
        <div>
          <label style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}>Directory Path:</label>
          <Space.Compact style={{ width: "100%" }}>
            <Input value={openDirPath} onChange={(e) => setOpenDirPath(e.target.value)} placeholder="e.g., D:\\Workspace\\tmp\\cowork_workspace" autoFocus />
            <Button icon={<FolderOpenOutlined />} onClick={handleSelectDirectory}>Browse</Button>
          </Space.Compact>
          <div style={{ marginTop: "8px", fontSize: "12px", color: "#999" }}>
            Path to any directory (with or without .cowork folder). This will open the directory in a new workspace context.
          </div>
        </div>
      </Modal>
    </div>
  );
};

export default ProjectsPanel;
