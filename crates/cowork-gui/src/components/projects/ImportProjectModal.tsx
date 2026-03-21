import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
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
  Steps,
  Checkbox,
  Tag,
  List,
  Card,
  Row,
  Col,
  Progress,
} from "antd";
import {
  FolderOpenOutlined,
  FileTextOutlined,
  RocketOutlined,
  CheckCircleOutlined,
  WarningOutlined,
  ProjectOutlined,
  SettingOutlined,
  LoadingOutlined,
} from "@ant-design/icons";
import type { CreateProjectResponse } from "../../types";

const { Text, Paragraph, Title } = Typography;

// Progress event from backend
interface ImportProgressEvent {
  step: string;
  message: string;
  progress: number;
}

interface ImportProjectModalProps {
  open: boolean;
  onClose: () => void;
  onSuccess: (projectId: string, projectName: string) => void;
}

interface ProjectPreview {
  name: string;
  path: string;
  technologies: string[];
  files_to_scan: string[];
  artifacts_to_generate: string[];
  warnings: string[];
}

interface ArtifactOptions {
  generate_idea: boolean;
  generate_prd: boolean;
  generate_design: boolean;
  generate_plan: boolean;
  scan_readme: boolean;
  scan_docs: boolean;
}

/**
 * Modal for importing an existing project into Cowork Forge
 * Supports multiple steps: select directory, preview, configure, and import
 */
const ImportProjectModal: React.FC<ImportProjectModalProps> = ({
  open,
  onClose,
  onSuccess,
}) => {
  const { message } = App.useApp();
  const [currentStep, setCurrentStep] = useState(0);
  const [projectPath, setProjectPath] = useState("");
  const [projectName, setProjectName] = useState("");
  const [loading, setLoading] = useState(false);
  const [preview, setPreview] = useState<ProjectPreview | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [artifactOptions, setArtifactOptions] = useState<ArtifactOptions>({
    generate_idea: true,
    generate_prd: true,
    generate_design: true,
    generate_plan: true,
    scan_readme: true,
    scan_docs: true,
  });

  // Import progress state
  const [importProgress, setImportProgress] = useState<ImportProgressEvent | null>(null);
  const [generatedArtifacts, setGeneratedArtifacts] = useState<string[]>([]);

  // Listen to import progress events
  useEffect(() => {
    const unlisten = listen<ImportProgressEvent>("import_progress", (event) => {
      setImportProgress(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Reset state when modal opens
  useEffect(() => {
    if (open) {
      setCurrentStep(0);
      setProjectPath("");
      setProjectName("");
      setPreview(null);
      setPreviewError(null);
      setImportProgress(null);
      setGeneratedArtifacts([]);
      setArtifactOptions({
        generate_idea: true,
        generate_prd: true,
        generate_design: true,
        generate_plan: true,
        scan_readme: true,
        scan_docs: true,
      });
    }
  }, [open]);

  // Handle directory selection
  const handleSelectDirectory = async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Select Project to Import",
      });
      if (selected && typeof selected === "string") {
        setProjectPath(selected);
        // Extract project name from path
        const parts = selected.split(/[/\\]/);
        setProjectName(parts[parts.length - 1] || parts[parts.length - 2] || "");
      }
    } catch (error) {
      console.error("Failed to open directory dialog:", error);
      message.error("Failed to open directory dialog: " + error);
    }
  };

  // Load preview when path changes
  useEffect(() => {
    if (!projectPath.trim()) {
      setPreview(null);
      setPreviewError(null);
      return;
    }

    const loadPreview = async () => {
      setPreviewLoading(true);
      setPreviewError(null);
      try {
        const result = await invoke<{ success: boolean; preview?: ProjectPreview; error?: string }>(
          "preview_import",
          { path: projectPath }
        );

        if (result.success && result.preview) {
          setPreview(result.preview);
        } else {
          setPreviewError(result.error || "Failed to preview project");
          setPreview(null);
        }
      } catch (error) {
        console.error("Failed to preview import:", error);
        setPreviewError(String(error));
        setPreview(null);
      } finally {
        setPreviewLoading(false);
      }
    };

    const timer = setTimeout(loadPreview, 500);
    return () => clearTimeout(timer);
  }, [projectPath]);

  // Handle import
  const handleImport = async () => {
    if (!projectPath.trim()) {
      message.warning("Please select a project directory");
      return;
    }

    setLoading(true);
    setImportProgress({ step: "start", message: "Starting import...", progress: 0 });
    
    try {
      const result = await invoke<{
        success: boolean;
        project_id?: string;
        project_name?: string;
        artifacts?: string[];
        error?: string;
      }>("import_project", {
        path: projectPath,
        projectName: projectName || null,
        generateIdea: artifactOptions.generate_idea,
        generatePrd: artifactOptions.generate_prd,
        generateDesign: artifactOptions.generate_design,
        generatePlan: artifactOptions.generate_plan,
        scanReadme: artifactOptions.scan_readme,
        scanDocs: artifactOptions.scan_docs,
      });

      if (result.success && result.project_id) {
        setGeneratedArtifacts(result.artifacts || []);
        setImportProgress({ step: "complete", message: "Import completed!", progress: 100 });
        message.success(`Project imported successfully: ${result.project_name}`);
        // Don't close immediately - show the results first
        // The user can close manually or we auto-close after a delay
        setTimeout(() => {
          handleClose();
          onSuccess(result.project_id!, result.project_name || projectName);
        }, 2000);
      } else {
        setImportProgress(null);
        message.error(result.error || "Failed to import project");
      }
    } catch (error) {
      console.error("Failed to import project:", error);
      setImportProgress(null);
      message.error("Failed to import project: " + error);
    } finally {
      setLoading(false);
    }
  };

  // Handle close
  const handleClose = () => {
    setCurrentStep(0);
    setProjectPath("");
    setProjectName("");
    setPreview(null);
    setPreviewError(null);
    setImportProgress(null);
    setGeneratedArtifacts([]);
    onClose();
  };

  // Step items for the wizard
  const stepItems = [
    { key: 0, title: "Select Directory" },
    { key: 1, title: "Preview" },
    { key: 2, title: "Configure" },
    { key: 3, title: "Import" },
  ];

  // Render step content
  const renderStepContent = () => {
    switch (currentStep) {
      case 0:
        return (
          <div>
            <Alert
              type="info"
              showIcon
              icon={<ProjectOutlined />}
              message="Import Existing Project"
              description="Select a directory containing an existing project. Cowork Forge will analyze the project and generate the necessary artifacts."
              style={{ marginBottom: 24 }}
            />

            <div style={{ marginBottom: 16 }}>
              <label style={{ display: "block", marginBottom: 8, fontWeight: 500 }}>
                Project Directory <Text type="danger">*</Text>
              </label>
              <Space.Compact style={{ width: "100%" }}>
                <Input
                  value={projectPath}
                  onChange={(e) => setProjectPath(e.target.value)}
                  placeholder="Select or enter project directory path"
                />
                <Button
                  icon={<FolderOpenOutlined />}
                  onClick={handleSelectDirectory}
                >
                  Browse
                </Button>
              </Space.Compact>
            </div>

            {previewLoading && (
              <div style={{ textAlign: "center", padding: 24 }}>
                <Spin size="large" />
                <Text type="secondary" style={{ display: "block", marginTop: 8 }}>
                  Analyzing project...
                </Text>
              </div>
            )}

            {previewError && (
              <Alert
                type="error"
                message="Cannot Import"
                description={previewError}
                style={{ marginTop: 16 }}
              />
            )}

            {preview && (
              <Alert
                type="success"
                icon={<CheckCircleOutlined />}
                message="Project Detected"
                description={`Found project: ${preview.name}`}
                style={{ marginTop: 16 }}
              />
            )}
          </div>
        );

      case 1:
        return (
          <div>
            {previewLoading ? (
              <div style={{ textAlign: "center", padding: 24 }}>
                <Spin size="large" />
                <Text type="secondary" style={{ display: "block", marginTop: 8 }}>
                  Analyzing project...
                </Text>
              </div>
            ) : preview ? (
              <>
                <Title level={5}>Project Analysis</Title>
                <Card size="small" style={{ marginBottom: 16 }}>
                  <Row gutter={16}>
                    <Col span={12}>
                      <Text type="secondary">Project Name:</Text>
                      <br />
                      <Text strong>{preview.name}</Text>
                    </Col>
                    <Col span={12}>
                      <Text type="secondary">Location:</Text>
                      <br />
                      <Text ellipsis>{preview.path}</Text>
                    </Col>
                  </Row>
                </Card>

                <Title level={5}>Detected Technologies</Title>
                <div style={{ marginBottom: 16 }}>
                  {preview.technologies.length > 0 ? (
                    preview.technologies.map((tech, i) => (
                      <Tag key={i} color="blue">
                        {tech}
                      </Tag>
                    ))
                  ) : (
                    <Text type="secondary">No technologies detected</Text>
                  )}
                </div>

                <Title level={5}>Artifacts to Generate</Title>
                <List
                  size="small"
                  bordered
                  dataSource={preview.artifacts_to_generate}
                  renderItem={(item) => (
                    <List.Item>
                      <FileTextOutlined style={{ marginRight: 8 }} />
                      {item}
                    </List.Item>
                  )}
                />

                {preview.warnings.length > 0 && (
                  <>
                    <Title level={5} style={{ marginTop: 16 }}>
                      <WarningOutlined /> Warnings
                    </Title>
                    {preview.warnings.map((warning, i) => (
                      <Alert
                        key={i}
                        type="warning"
                        message={warning}
                        style={{ marginBottom: 8 }}
                      />
                    ))}
                  </>
                )}
              </>
            ) : (
              <Alert
                type="error"
                message="No project preview available"
                description="Please go back and select a valid project directory"
              />
            )}
          </div>
        );

      case 2:
        return (
          <div>
            <Title level={5}>Artifact Generation Options</Title>
            <Paragraph type="secondary">
              Configure which artifacts should be generated during import.
            </Paragraph>

            <Card size="small" style={{ marginBottom: 16 }}>
              <Checkbox
                checked={artifactOptions.generate_idea}
                onChange={(e) =>
                  setArtifactOptions({
                    ...artifactOptions,
                    generate_idea: e.target.checked,
                  })
                }
              >
                <Text strong>idea.md</Text>
                <br />
                <Text type="secondary" style={{ marginLeft: 24 }}>
                  Project overview, background, and key features
                </Text>
              </Checkbox>
            </Card>

            <Card size="small" style={{ marginBottom: 16 }}>
              <Checkbox
                checked={artifactOptions.generate_prd}
                onChange={(e) =>
                  setArtifactOptions({
                    ...artifactOptions,
                    generate_prd: e.target.checked,
                  })
                }
              >
                <Text strong>prd.md</Text>
                <br />
                <Text type="secondary" style={{ marginLeft: 24 }}>
                  Product requirements and functional specifications
                </Text>
              </Checkbox>
            </Card>

            <Card size="small" style={{ marginBottom: 16 }}>
              <Checkbox
                checked={artifactOptions.generate_design}
                onChange={(e) =>
                  setArtifactOptions({
                    ...artifactOptions,
                    generate_design: e.target.checked,
                  })
                }
              >
                <Text strong>design.md</Text>
                <br />
                <Text type="secondary" style={{ marginLeft: 24 }}>
                  Technical architecture and design decisions
                </Text>
              </Checkbox>
            </Card>

            <Card size="small" style={{ marginBottom: 16 }}>
              <Checkbox
                checked={artifactOptions.generate_plan}
                onChange={(e) =>
                  setArtifactOptions({
                    ...artifactOptions,
                    generate_plan: e.target.checked,
                  })
                }
              >
                <Text strong>plan.md</Text>
                <br />
                <Text type="secondary" style={{ marginLeft: 24 }}>
                  Implementation plan and next steps
                </Text>
              </Checkbox>
            </Card>

            <Divider />

            <Title level={5}>Source Scanning Options</Title>

            <Checkbox
              checked={artifactOptions.scan_readme}
              onChange={(e) =>
                setArtifactOptions({
                  ...artifactOptions,
                  scan_readme: e.target.checked,
                })
              }
              style={{ marginRight: 24 }}
            >
              Scan README.md
            </Checkbox>

            <Checkbox
              checked={artifactOptions.scan_docs}
              onChange={(e) =>
                setArtifactOptions({
                  ...artifactOptions,
                  scan_docs: e.target.checked,
                })
              }
            >
              Scan docs/ directory
            </Checkbox>
          </div>
        );

      case 3:
        return (
          <div>
            {loading ? (
              // Show progress during import
              <div style={{ textAlign: "center", padding: "24px 0" }}>
                <Spin indicator={<LoadingOutlined style={{ fontSize: 48 }} spin />} />
                <div style={{ marginTop: 24, marginBottom: 16 }}>
                  <Text strong style={{ fontSize: 16 }}>{importProgress?.message || "Importing project..."}</Text>
                </div>
                <Progress 
                  percent={importProgress?.progress || 0} 
                  status="active"
                  strokeColor={{
                    '0%': '#108ee9',
                    '100%': '#87d068',
                  }}
                />
                <Text type="secondary" style={{ marginTop: 8, display: "block" }}>
                  Step: {importProgress?.step || "initializing"}
                </Text>
              </div>
            ) : importProgress?.step === "complete" ? (
              // Show results after completion
              <div>
                <Alert
                  type="success"
                  icon={<CheckCircleOutlined />}
                  message="Import Completed Successfully!"
                  description="Your project has been imported and the following artifacts have been generated:"
                  style={{ marginBottom: 24 }}
                  showIcon
                />

                <Title level={5}>Generated Artifacts</Title>
                <List
                  size="small"
                  bordered
                  dataSource={generatedArtifacts}
                  renderItem={(item) => (
                    <List.Item>
                      <CheckCircleOutlined style={{ color: "#52c41a", marginRight: 8 }} />
                      <Text>{item}</Text>
                    </List.Item>
                  )}
                />

                {generatedArtifacts.length === 0 && (
                  <Alert
                    type="warning"
                    message="No artifacts were generated"
                    description="Please check your artifact options and try again."
                    style={{ marginTop: 16 }}
                  />
                )}

                <Alert
                  type="info"
                  message="Next Steps"
                  description="You can now start a new iteration to enhance your project. The generated artifacts provide a foundation for further development."
                  style={{ marginTop: 16 }}
                />
              </div>
            ) : (
              // Initial state - ready to import
              <div>
                <Alert
                  type="info"
                  icon={<RocketOutlined />}
                  message="Ready to Import"
                  description="The following will be created in your project:"
                  style={{ marginBottom: 24 }}
                />

                <Title level={5}>Project</Title>
                <Card size="small" style={{ marginBottom: 16 }}>
                  <Text strong>{preview?.name || projectName}</Text>
                  <br />
                  <Text type="secondary">{projectPath}</Text>
                </Card>

                <Title level={5}>Files to be Created</Title>
                <List size="small" bordered>
                  <List.Item>
                    <Text>.cowork-v2/ directory structure</Text>
                  </List.Item>
                  {artifactOptions.generate_idea && (
                    <List.Item>artifacts/idea.md</List.Item>
                  )}
                  {artifactOptions.generate_prd && (
                    <List.Item>artifacts/prd.md</List.Item>
                  )}
                  {artifactOptions.generate_design && (
                    <List.Item>artifacts/design.md</List.Item>
                  )}
                  {artifactOptions.generate_plan && (
                    <List.Item>artifacts/plan.md</List.Item>
                  )}
                </List>

                <Alert
                  type="warning"
                  message="Note"
                  description="This will create a .cowork-v2 directory in your project folder. The original project files will not be modified."
                  style={{ marginTop: 16 }}
                />
              </div>
            )}
          </div>
        );

      default:
        return null;
    }
  };

  // Determine next button state
  const canProceed = () => {
    switch (currentStep) {
      case 0:
        return projectPath.trim() && preview && !previewError;
      case 1:
        return preview !== null;
      case 2:
        return (
          artifactOptions.generate_idea ||
          artifactOptions.generate_prd ||
          artifactOptions.generate_design ||
          artifactOptions.generate_plan
        );
      case 3:
        return true;
      default:
        return false;
    }
  };

  return (
    <Modal
      title={
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <FolderOpenOutlined />
          <span>Import Existing Project</span>
        </div>
      }
      open={open}
      onCancel={handleClose}
      width={700}
      footer={
        <Space>
          {currentStep > 0 && !loading && importProgress?.step !== "complete" && (
            <Button onClick={() => setCurrentStep(currentStep - 1)}>Back</Button>
          )}
          {currentStep < 3 && !loading && (
            <Button
              type="primary"
              disabled={!canProceed()}
              onClick={() => setCurrentStep(currentStep + 1)}
            >
              Next
            </Button>
          )}
          {currentStep === 3 && importProgress?.step !== "complete" && (
            <Button
              type="primary"
              loading={loading}
              disabled={loading}
              onClick={handleImport}
              icon={<RocketOutlined />}
            >
              Import Project
            </Button>
          )}
          {importProgress?.step === "complete" && (
            <Button type="primary" onClick={handleClose}>
              Done
            </Button>
          )}
        </Space>
      }
    >
      <Steps current={currentStep} items={stepItems} style={{ marginBottom: 24 }} />
      {renderStepContent()}
    </Modal>
  );
};

export default ImportProjectModal;
