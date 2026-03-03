import { useState } from "react";
import { App, Modal, Input, Button, Form, Select, Radio } from "antd";
import { UploadOutlined } from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type { IterationInfo, CreateIterationRequest } from '../../types';

const { TextArea } = Input;
const { Option } = Select;

interface CreateIterationModalProps {
  open: boolean;
  onClose: () => void;
  onSuccess: () => void;
  completedIterations: IterationInfo[];
}

/**
 * Modal for creating a new iteration
 * Extracted from IterationsPanel.tsx
 */
const CreateIterationModal: React.FC<CreateIterationModalProps> = ({
  open,
  onClose,
  onSuccess,
  completedIterations
}) => {
  const { message } = App.useApp();
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [baseIteration, setBaseIteration] = useState<string | null>(null);
  const [inheritance, setInheritance] = useState("partial");

  const handleImportMarkdown = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [
          {
            name: "Markdown",
            extensions: ["md", "markdown", "txt"],
          },
        ],
      });

      if (selected) {
        const filePath = typeof selected === "string" ? selected : null;
        if (filePath) {
          const content = await invoke<string>("read_local_file", { filePath });
          setDescription(content);
          message.success("Markdown file imported successfully");
        }
      }
    } catch (error) {
      message.error("Failed to import file: " + error);
    }
  };

  const handleCreate = async () => {
    if (!title.trim()) {
      message.warning("Please enter a title");
      return;
    }

    try {
      const request: CreateIterationRequest = {
        title: title,
        description: description || title,
        base_iteration_id: baseIteration,
        inheritance: inheritance as 'full' | 'partial' | 'none',
      };

      await invoke("gui_create_iteration", { request });
      message.success("Iteration created successfully");
      handleClose();
      onSuccess();
    } catch (error) {
      message.error("Failed to create iteration: " + error);
    }
  };

  const handleClose = () => {
    setTitle("");
    setDescription("");
    setBaseIteration(null);
    setInheritance("partial");
    onClose();
  };

  return (
    <Modal
      title="Create New Iteration"
      open={open}
      onOk={handleCreate}
      onCancel={handleClose}
      width={600}
    >
      <Form layout="vertical">
        <Form.Item label="Title" required>
          <Input
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="e.g., Add user authentication"
            autoFocus
          />
        </Form.Item>
        <Form.Item label="Description">
          <TextArea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Describe what you want to achieve in this iteration..."
            rows={4}
          />
          <div style={{ marginTop: "8px" }}>
            <Button icon={<UploadOutlined />} onClick={handleImportMarkdown} size="small">
              Import Markdown
            </Button>
          </div>
        </Form.Item>
        {completedIterations.length > 0 && (
          <>
            <Form.Item label="Base Iteration (Optional)">
              <Select
                value={baseIteration}
                onChange={setBaseIteration}
                placeholder="Select a base iteration to inherit from"
                allowClear
              >
                {completedIterations.map((iteration) => (
                  <Option key={iteration.id} value={iteration.id}>
                    #{iteration.number} - {iteration.title}
                  </Option>
                ))}
              </Select>
              <div style={{ marginTop: "4px", fontSize: "12px", color: "#888" }}>
                Leave empty to start from scratch (Genesis iteration)
              </div>
            </Form.Item>
            {baseIteration && (
              <Form.Item label="Inheritance Mode">
                <Radio.Group value={inheritance} onChange={(e) => setInheritance(e.target.value)}>
                  <Radio.Button value="full">Full (All code)</Radio.Button>
                  <Radio.Button value="partial">Partial (Artifacts only)</Radio.Button>
                  <Radio.Button value="none">None (Fresh start)</Radio.Button>
                </Radio.Group>
              </Form.Item>
            )}
          </>
        )}
      </Form>
    </Modal>
  );
};

export default CreateIterationModal;
