import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { App, Modal, Input } from "antd";
import type { ProjectData, UpdateProjectRequest } from '../../types';

interface EditProjectModalProps {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
  project: ProjectData | null;
}

/**
 * Modal for editing project information
 * Extracted from ProjectsPanel.tsx
 */
const EditProjectModal: React.FC<EditProjectModalProps> = ({
  open,
  onClose,
  onSuccess,
  project,
}) => {
  const { message } = App.useApp();
  const [projectName, setProjectName] = useState(project?.name || "");
  const [projectDescription, setProjectDescription] = useState(project?.description || "");

  // Update state when project changes
  useEffect(() => {
    if (project) {
      setProjectName(project.name);
      setProjectDescription(project.description || "");
    }
  }, [project]);

  const handleEdit = async () => {
    if (!projectName.trim()) {
      message.warning("Please enter project name");
      return;
    }

    if (!project) return;

    try {
      const request: UpdateProjectRequest = {
        projectId: project.project_id,
        name: projectName,
        description: projectDescription || null,
        status: null,
      };

      await invoke("update_project", { request });
      message.success("Project updated successfully");
      handleClose();
      onSuccess();
    } catch (error) {
      message.error("Failed to update project: " + error);
    }
  };

  const handleClose = () => {
    setProjectName("");
    setProjectDescription("");
    onClose();
  };

  return (
    <Modal
      title="Edit Project"
      open={open}
      onOk={handleEdit}
      onCancel={handleClose}
      okText="Save"
      cancelText="Cancel"
    >
      <div style={{ marginBottom: "16px" }}>
        <label style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}>
          Project Name:
        </label>
        <Input
          value={projectName}
          onChange={(e) => setProjectName(e.target.value)}
          placeholder="Project name"
          autoFocus
        />
      </div>
      <div>
        <label style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}>
          Description:
        </label>
        <Input.TextArea
          value={projectDescription}
          onChange={(e) => setProjectDescription(e.target.value)}
          placeholder="Project description"
          rows={4}
        />
      </div>
    </Modal>
  );
};

export default EditProjectModal;
