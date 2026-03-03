import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  App,
  Modal,
  Input,
  Button,
  Space,
  Alert,
  Divider,
  Typography,
  Spin,
} from "antd";
import {
  FolderOpenOutlined,
  FolderOutlined,
  FolderAddOutlined,
  CheckCircleOutlined,
  RocketOutlined,
} from "@ant-design/icons";

const { Text, Paragraph } = Typography;

interface CreateProjectModalProps {
  open: boolean;
  onClose: () => void;
  onSuccess: (projectId: string, projectName: string) => void;
}

/**
 * Modal for creating a new project with path validation
 * Extracted from ProjectsPanel.tsx
 */
const CreateProjectModal: React.FC<CreateProjectModalProps> = ({
  open,
  onClose,
  onSuccess,
}) => {
  const { message } = App.useApp();
  const [projectPath, setProjectPath] = useState("");
  const [projectName, setProjectName] = useState("");
  const [projectDescription, setProjectDescription] = useState("");
  const [loading, setLoading] = useState(false);
  const [pathExists, setPathExists] = useState<boolean | null>(null);
  const [checkingPath, setCheckingPath] = useState(false);

  // Check if path exists when path changes
  useEffect(() => {
    if (!projectPath.trim()) {
      setPathExists(null);
      return;
    }

    const checkPath = async () => {
      setCheckingPath(true);
      try {
        const exists = await invoke<boolean>("path_exists", { path: projectPath });
        setPathExists(exists);
      } catch {
        setPathExists(false);
      } finally {
        setCheckingPath(false);
      }
    };

    const timer = setTimeout(checkPath, 300);
    return () => clearTimeout(timer);
  }, [projectPath]);

  // Auto-fill project name from path
  useEffect(() => {
    if (projectPath && !projectName) {
      const parts = projectPath.split(/[/\\]/);
      const folderName = parts[parts.length - 1] || parts[parts.length - 2] || "";
      if (folderName) {
        setProjectName(folderName);
      }
    }
  }, [projectPath]);

  const handleSelectDirectory = async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Select Project Directory",
      });
      if (selected && typeof selected === "string") {
        setProjectPath(selected);
      }
    } catch (error) {
      console.error("Failed to open directory dialog:", error);
      message.error("Failed to open directory dialog: " + error);
    }
  };

  const handleCreate = async () => {
    if (!projectPath.trim()) {
      message.warning("Please select or enter a project directory");
      return;
    }

    if (!projectName.trim()) {
      message.warning("Please enter a project name");
      return;
    }

    setLoading(true);
    try {
      const result = await invoke<{ project_id: string; created_dir: boolean }>(
        "create_project_at_path",
        {
          path: projectPath,
          name: projectName,
          description: projectDescription || null,
        }
      );

      if (result.created_dir) {
        message.success(`Project created and directory initialized: ${projectName}`);
      } else {
        message.success(`Project created: ${projectName}`);
      }

      handleClose();
      onSuccess(result.project_id, projectName);
    } catch (error) {
      message.error("Failed to create project: " + error);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setProjectPath("");
    setProjectName("");
    setProjectDescription("");
    setPathExists(null);
    onClose();
  };

  return (
    <Modal
      title={
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <FolderAddOutlined />
          <span>Create New Project</span>
        </div>
      }
      open={open}
      onOk={handleCreate}
      onCancel={handleClose}
      okText="Create Project"
      cancelText="Cancel"
      confirmLoading={loading}
      width={600}
      okButtonProps={{ disabled: !projectPath.trim() || !projectName.trim() }}
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
            value={projectPath}
            onChange={(e) => setProjectPath(e.target.value)}
            placeholder="e.g., D:\Projects\my-awesome-project"
            autoFocus
          />
          <Button icon={<FolderOpenOutlined />} onClick={handleSelectDirectory}>
            Browse
          </Button>
        </Space.Compact>
        <div style={{ marginTop: "6px", minHeight: "22px" }}>
          {checkingPath && <Spin size="small" />}
          {!checkingPath && pathExists === true && (
            <Text type="success">
              <CheckCircleOutlined style={{ marginRight: "4px" }} />
              Directory exists - will use existing folder
            </Text>
          )}
          {!checkingPath && pathExists === false && projectPath.trim() && (
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
          value={projectName}
          onChange={(e) => setProjectName(e.target.value)}
          placeholder="My Awesome Project"
        />
      </div>

      <div>
        <label style={{ display: "block", marginBottom: "8px", fontWeight: 500 }}>
          Description <Text type="secondary">(Optional)</Text>
        </label>
        <Input.TextArea
          value={projectDescription}
          onChange={(e) => setProjectDescription(e.target.value)}
          placeholder="Brief description of what this project is about..."
          rows={2}
        />
      </div>
    </Modal>
  );
};

export default CreateProjectModal;
