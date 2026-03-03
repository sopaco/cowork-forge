import { useState } from "react";
import { App, Modal, Input } from "antd";
import { invoke } from "@tauri-apps/api/core";

interface InitProjectModalProps {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

/**
 * Modal for initializing a new project
 * Extracted from IterationsPanel.tsx
 */
const InitProjectModal: React.FC<InitProjectModalProps> = ({
  open,
  onClose,
  onSuccess
}) => {
  const { message } = App.useApp();
  const [projectName, setProjectName] = useState("");

  const handleInit = async () => {
    if (!projectName.trim()) {
      message.warning("Please enter a project name");
      return;
    }

    try {
      await invoke("gui_init_project", { name: projectName });
      message.success("Project initialized successfully");
      handleClose();
      onSuccess();
    } catch (error) {
      message.error("Failed to initialize project: " + error);
    }
  };

  const handleClose = () => {
    setProjectName("");
    onClose();
  };

  return (
    <Modal
      title="Initialize Project"
      open={open}
      onOk={handleInit}
      onCancel={handleClose}
      okText="Initialize"
      cancelText="Cancel"
    >
      <div style={{ marginBottom: "16px" }}>
        <label style={{ display: "block", marginBottom: "8px", fontWeight: "bold" }}>
          Project Name:
        </label>
        <Input
          value={projectName}
          onChange={(e) => setProjectName(e.target.value)}
          placeholder="Enter project name"
          autoFocus
          onPressEnter={handleInit}
        />
      </div>
    </Modal>
  );
};

export default InitProjectModal;
