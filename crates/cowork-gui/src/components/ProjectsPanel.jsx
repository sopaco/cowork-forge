import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
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
} from "@ant-design/icons";

const ProjectsPanel = () => {
  const [projects, setProjects] = useState([]);
  const [loading, setLoading] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showOpenDirModal, setShowOpenDirModal] = useState(false);
  const [selectedProject, setSelectedProject] = useState(null);
  const [openDirPath, setOpenDirPath] = useState("");
  const [editProjectName, setEditProjectName] = useState("");
  const [editProjectDescription, setEditProjectDescription] = useState("");

  const loadProjects = async () => {
    setLoading(true);
    try {
      const data = await invoke("get_all_projects", {
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

    // Listen for project registration events to refresh the list
    const unlistenProjectLoaded = listen("project_loaded", () => {
      console.log(
        "[ProjectsPanel] Project loaded event received, reloading projects...",
      );
      loadProjects();
    });

    const unlistenProjectCreated = listen("project_created", () => {
      console.log(
        "[ProjectsPanel] Project created event received, reloading projects...",
      );
      loadProjects();
    });

    return () => {
      unlistenProjectLoaded
        .then((fn) => fn())
        .catch((e) =>
          console.error(
            "[ProjectsPanel] Failed to unlisten project_loaded:",
            e,
          ),
        );
      unlistenProjectCreated
        .then((fn) => fn())
        .catch((e) =>
          console.error(
            "[ProjectsPanel] Failed to unlisten project_created:",
            e,
          ),
        );
    };
  }, []);

  const handleEditProject = async () => {
    if (!editProjectName.trim()) {
      message.warning("Please enter project name");
      return;
    }

    try {
      await invoke("update_project", {
        projectId: selectedProject.project_id,
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

  const handleDeleteProject = async (project) => {
    Modal.confirm({
      title: "Delete Project",
      content: `Are you sure you want to delete "${project.name}"?`,
      okText: "Delete Record Only",
      okType: "default",
      cancelText: "Delete All (Files & Record)",
      onOk: async () => {
        try {
          // Check if current window has this project open
          const currentWorkspace = await invoke("get_workspace");
          const isCurrentWindowProject =
            currentWorkspace === project.workspacePath;

          await invoke("delete_project", {
            projectId: project.project_id,
            deleteFiles: false,
          });
          message.success("Project record deleted");
          loadProjects();

          // If the deleted project was open in current window, clear workspace
          if (isCurrentWindowProject) {
            message.info(
              "Project was open in current window. Workspace cleared.",
            );
            // Reload the page to clear all state
            window.location.reload();
          }
        } catch (error) {
          message.error("Failed to delete project: " + error);
        }
      },
      onCancel: async () => {
        try {
          // Check if current window has this project open
          const currentWorkspace = await invoke("get_workspace");
          const isCurrentWindowProject =
            currentWorkspace === project.workspacePath;

          await invoke("delete_project", {
            projectId: project.project_id,
            deleteFiles: true,
          });
          message.success("Project deleted (files and record)");
          loadProjects();

          // If the deleted project was open in current window, clear workspace
          if (isCurrentWindowProject) {
            message.info(
              "Project was open in current window. Workspace cleared.",
            );
            // Reload the page to clear all state
            window.location.reload();
          }
        } catch (error) {
          message.error("Failed to delete project: " + error);
        }
      },
    });
  };

  const handleOpenProject = async (projectId) => {
    try {
      // Check if current window already has a project
      const hasProject = await invoke("has_open_project");

      if (hasProject) {
        // Open in new window
        await invoke("open_project", { projectId });
        message.info("Opening project in new window...");
      } else {
        // Open in current window
        await invoke("open_project_in_current_window", { projectId });
        message.success("Project opened successfully");
        // The frontend will automatically reload sessions when 'project_loaded' event is received
      }
    } catch (error) {
      message.error("Failed to open project: " + error);
    }
  };
  const handleOpenEditModal = (project) => {
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
      // Check if current window already has a project
      const hasProject = await invoke("has_open_project");

      if (hasProject) {
        // Register the project and open in new window
        const projectName = openDirPath.split(/[/\\]/).pop() || openDirPath;
        await invoke("register_project", {
          workspacePath: openDirPath,
          name: projectName,
          description: `Project at ${openDirPath}`,
        });

        // Get the newly registered project
        const allProjects = await invoke("get_all_projects", {
          status: null,
          search: null,
          limit: null,
        });
        const newProject = allProjects.find(
          (p) => p.workspacePath === openDirPath,
        );

        if (newProject) {
          await invoke("open_project", { projectId: newProject.projectId });
          message.success("Opening project in new window...");
        } else {
          message.error("Failed to find newly registered project");
        }

        setShowOpenDirModal(false);
        setOpenDirPath("");
        loadProjects();
      } else {
        // Open in current window
        await invoke("set_workspace", { workspacePath: openDirPath });
        message.success("Workspace set successfully");
        setShowOpenDirModal(false);
        setOpenDirPath("");
        // The frontend will automatically reload sessions when 'project_loaded' event is received
      }
    } catch (error) {
      message.error("Failed to open directory: " + error);
    }
  };

  const formatDate = (dateString) => {
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

  const getDisplayPath = (fullPath) => {
    if (!fullPath) return "No path";
    const parts = fullPath.split(/[/\\]/);
    // 只显示最后两个部分（文件夹名和它的父目录）
    if (parts.length >= 2) {
      return ".../" + parts.slice(-2).join("/");
    }
    return fullPath;
  };

  const getStatusColor = (status) => {
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

  return (
    <div style={{ padding: "24px" }}>
      <div
        style={{
          marginBottom: "24px",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <h2 style={{ margin: 0 }}>Projects</h2>
        <Button
          icon={<FolderOpenOutlined />}
          onClick={() => setShowOpenDirModal(true)}
        >
          Open Directory
        </Button>
      </div>

      {loading ? (
        <div style={{ textAlign: "center", padding: "60px 0" }}>
          <Spin size="large" />
          <div style={{ marginTop: "16px", color: "#999" }}>
            Loading projects...
          </div>
        </div>
      ) : projects.length === 0 ? (
        <Empty
          description="No projects yet"
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        >
          <Button
            type="primary"
            icon={<FolderOpenOutlined />}
            onClick={() => setShowOpenDirModal(true)}
          >
            Open Directory to Start
          </Button>
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
                  onClick={() => handleOpenProject(project.project_id)}
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
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: "8px",
                    }}
                  >
                    <span>{project.name}</span>
                    <Tag
                      color={getStatusColor(project.status)}
                      style={{ margin: 0 }}
                    >
                      {project.status}
                    </Tag>
                  </div>
                }
                description={
                  <div
                    style={{
                      display: "flex",
                      flexDirection: "column",
                      gap: "8px",
                    }}
                  >
                    <div
                      style={{
                        color: "#666",
                        fontSize: "14px",
                        display: "-webkit-box",
                        WebkitLineClamp: "2",
                        WebkitBoxOrient: "vertical",
                        overflow: "hidden",
                      }}
                    >
                      {project.description || "No description provided"}
                    </div>
                    <div
                      style={{ fontSize: "12px", color: "#999", minWidth: 0 }}
                    >
                      <Tooltip title={project.workspace_path || "No path"}>
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
                          <FolderOpenOutlined
                            style={{ marginRight: "4px", flexShrink: 0 }}
                          />
                          {getDisplayPath(project.workspace_path)}
                        </span>
                      </Tooltip>
                    </div>
                    <div
                      style={{
                        display: "flex",
                        gap: "16px",
                        fontSize: "12px",
                        color: "#999",
                      }}
                    >
                      <span>
                        <CheckCircleOutlined style={{ marginRight: "4px" }} />
                        {project.metadata.session_count} sessions
                      </span>
                      <span>
                        <ClockCircleOutlined style={{ marginRight: "4px" }} />
                        Last opened: {formatDate(project.last_opened_at)}
                      </span>
                    </div>
                    {project.metadata.technology_stack.length > 0 && (
                      <div
                        style={{
                          display: "flex",
                          flexWrap: "wrap",
                          gap: "4px",
                        }}
                      >
                        {project.metadata.technology_stack
                          .slice(0, 4)
                          .map((tech, idx) => (
                            <Tag key={idx} size="small" color="blue">
                              {tech}
                            </Tag>
                          ))}
                        {project.metadata.technology_stack.length > 4 && (
                          <Tag size="small" color="default">
                            +{project.metadata.technology_stack.length - 4}
                          </Tag>
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

      {/* Edit Project Modal */}
      <Modal
        title="Edit Project"
        open={showEditModal}
        onOk={handleEditProject}
        onCancel={() => {
          setShowEditModal(false);
          setSelectedProject(null);
          setEditProjectName("");
          setEditProjectDescription("");
        }}
        okText="Save"
        cancelText="Cancel"
      >
        <div style={{ marginBottom: "16px" }}>
          <label
            style={{
              display: "block",
              marginBottom: "8px",
              fontWeight: "bold",
            }}
          >
            Project Name:
          </label>
          <Input
            value={editProjectName}
            onChange={(e) => setEditProjectName(e.target.value)}
            placeholder="Project name"
            autoFocus
          />
        </div>
        <div>
          <label
            style={{
              display: "block",
              marginBottom: "8px",
              fontWeight: "bold",
            }}
          >
            Description:
          </label>
          <Input.TextArea
            value={editProjectDescription}
            onChange={(e) => setEditProjectDescription(e.target.value)}
            placeholder="Project description"
            rows={4}
          />
        </div>
      </Modal>

      {/* Open Directory Modal */}
      <Modal
        title="Open Directory"
        open={showOpenDirModal}
        onOk={handleOpenDirectory}
        onCancel={() => {
          setShowOpenDirModal(false);
          setOpenDirPath("");
        }}
        okText="Open"
        cancelText="Cancel"
      >
        <div>
          <label
            style={{
              display: "block",
              marginBottom: "8px",
              fontWeight: "bold",
            }}
          >
            Directory Path:
          </label>
          <Input
            value={openDirPath}
            onChange={(e) => setOpenDirPath(e.target.value)}
            placeholder="e.g., D:\\Workspace\\tmp\\cowork_workspace"
            autoFocus
          />
          <div style={{ marginTop: "8px", fontSize: "12px", color: "#999" }}>
            Path to any directory (with or without .cowork folder). This will
            open the directory in a new workspace context.
          </div>
        </div>
      </Modal>
    </div>
  );
};

export default ProjectsPanel;
