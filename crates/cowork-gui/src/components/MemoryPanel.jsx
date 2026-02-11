import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Card,
  Input,
  Select,
  Button,
  Tag,
  Empty,
  Spin,
  Modal,
  Tabs,
  Typography,
  Space,
  Divider,
  message,
} from "antd";
import {
  SearchOutlined,
  EyeOutlined,
  DatabaseOutlined,
  ClockCircleOutlined,
} from "@ant-design/icons";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import rehypeRaw from "rehype-raw";
import "highlight.js/styles/github.css";

const { TextArea } = Input;
const { Option } = Select;
const { Text, Paragraph } = Typography;

function MemoryPanel({ currentSession, refreshTrigger }) {
  const [memories, setMemories] = useState([]);
  const [loading, setLoading] = useState(false);
  const [queryType, setQueryType] = useState("all");
  const [category, setCategory] = useState("all");
  const [stage, setStage] = useState("");
  const [limit, setLimit] = useState(20);
  const [selectedMemory, setSelectedMemory] = useState(null);
  const [memoryDetail, setMemoryDetail] = useState(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    loadMemories();
  }, [queryType, category, stage, limit, currentSession, refreshTrigger]);

  const loadMemories = async () => {
    setLoading(true);
    try {
      const params = {
        queryType: queryType,
        category: category,
        stage: stage || null,
        limit: limit,
        iterationId: currentSession || null,
      };

      const result = await invoke("query_memory_index", params);
      setMemories(result.results || []);
      setTotal(result.total || 0);
    } catch (error) {
      console.error("[MemoryPanel] Failed to load memories:", error);
      message.error("Failed to load memories: " + error);
      setMemories([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = () => {
    loadMemories();
  };

  const handleViewDetail = async (memory) => {
    setSelectedMemory(memory);
    setDetailLoading(true);
    try {
      const detail = await invoke("load_memory_detail", {
        memoryId: memory.id,
        file: memory.file,
        iterationId: currentSession || null,
      });
      setMemoryDetail(detail);
    } catch (error) {
      console.error("[MemoryPanel] Failed to load memory detail:", error);
      message.error("Failed to load memory detail: " + error);
      setMemoryDetail(null);
    } finally {
      setDetailLoading(false);
    }
  };

  const getCategoryColor = (cat) => {
    switch (cat) {
      case "decision":
        return "blue";
      case "experience":
        return "green";
      case "pattern":
        return "purple";
      case "record":
        return "orange";
      default:
        return "default";
    }
  };

  const getImpactColor = (impact) => {
    switch (impact) {
      case "high":
        return "red";
      case "medium":
        return "orange";
      case "low":
        return "green";
      default:
        return "default";
    }
  };

  const formatDate = (dateStr) => {
    if (!dateStr) return "N/A";
    try {
      const date = new Date(dateStr);
      return date.toLocaleString();
    } catch {
      return dateStr;
    }
  };

  return (
    <div
      style={{
        padding: "20px",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: "20px",
          flexShrink: 0,
        }}
      >
        <h2
          style={{
            margin: 0,
            display: "flex",
            alignItems: "center",
            gap: "10px",
          }}
        >
          <DatabaseOutlined />
          Memory Browser
        </h2>
        <Space>
          <Text type="secondary">Total: {total}</Text>
          <Button
            icon={<SearchOutlined />}
            onClick={handleSearch}
            loading={loading}
          >
            Refresh
          </Button>
        </Space>
      </div>

      <Card size="small" style={{ marginBottom: "20px", flexShrink: 0 }}>
        <Space direction="vertical" style={{ width: "100%" }} size="small">
          <div
            style={{
              display: "flex",
              gap: "10px",
              alignItems: "center",
              flexWrap: "wrap",
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: "8px",
                minWidth: "150px",
              }}
            >
              <Text strong>Query Type:</Text>
              <Select
                value={queryType}
                onChange={setQueryType}
                style={{ flex: 1, minWidth: "120px" }}
                size="small"
              >
                <Option value="all">All</Option>
                <Option value="project">Project</Option>
                <Option value="session">Session</Option>
              </Select>
            </div>

            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: "8px",
                minWidth: "150px",
              }}
            >
              <Text strong>Category:</Text>
              <Select
                value={category}
                onChange={setCategory}
                style={{ flex: 1, minWidth: "120px" }}
                size="small"
              >
                <Option value="all">All</Option>
                <Option value="decision">Decisions</Option>
                <Option value="experience">Experiences</Option>
                <Option value="pattern">Patterns</Option>
                <Option value="record">Records</Option>
              </Select>
            </div>

            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: "8px",
                minWidth: "150px",
              }}
            >
              <Text strong>Stage:</Text>
              <Select
                value={stage}
                onChange={setStage}
                style={{ flex: 1, minWidth: "120px" }}
                size="small"
                allowClear
                placeholder="All"
              >
                <Option value="idea">Idea</Option>
                <Option value="prd">PRD</Option>
                <Option value="design">Design</Option>
                <Option value="plan">Plan</Option>
                <Option value="coding">Coding</Option>
                <Option value="check">Check</Option>
              </Select>
            </div>

            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: "8px",
                minWidth: "100px",
              }}
            >
              <Text strong>Limit:</Text>
              <Select
                value={limit}
                onChange={setLimit}
                style={{ flex: 1, minWidth: "80px" }}
                size="small"
              >
                <Option value={10}>10</Option>
                <Option value={20}>20</Option>
                <Option value={50}>50</Option>
                <Option value={100}>100</Option>
              </Select>
            </div>
          </div>
        </Space>
      </Card>

      <div
        style={{
          flex: 1,
          overflow: "auto",
          display: "flex",
          flexDirection: "column",
        }}
      >
        {loading ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
            }}
          >
            <Spin size="large" />
          </div>
        ) : memories.length === 0 ? (
          <Empty
            description="No memories found"
            style={{ marginTop: "50px" }}
          />
        ) : (
          <div
            style={{ display: "flex", flexDirection: "column", gap: "10px" }}
          >
            {memories.map((memory) => (
              <Card
                key={memory.id}
                size="small"
                hoverable
                style={{ cursor: "pointer" }}
                onClick={() => handleViewDetail(memory)}
              >
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "flex-start",
                    marginBottom: "8px",
                  }}
                >
                  <div style={{ flex: 1 }}>
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: "8px",
                        marginBottom: "4px",
                      }}
                    >
                      <Text strong>{memory.title}</Text>
                      <Tag color={getCategoryColor(memory.category)}>
                        {memory.category}
                      </Tag>
                      {memory.stage && <Tag>{memory.stage}</Tag>}
                    </div>
                    <Paragraph
                      ellipsis={{ rows: 2 }}
                      style={{ margin: 0, fontSize: "13px", color: "#888" }}
                    >
                      {memory.summary}
                    </Paragraph>
                  </div>
                  <Button
                    type="text"
                    icon={<EyeOutlined />}
                    size="small"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleViewDetail(memory);
                    }}
                  />
                </div>
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    fontSize: "12px",
                    color: "#666",
                  }}
                >
                  <Space size="small">
                    <span>
                      <ClockCircleOutlined /> {formatDate(memory.created_at)}
                    </span>
                    {memory.impact && (
                      <Tag color={getImpactColor(memory.impact)} size="small">
                        {memory.impact}
                      </Tag>
                    )}
                  </Space>
                  <Space size="small">
                    {memory.tags &&
                      memory.tags.slice(0, 2).map((tag, idx) => (
                        <Tag key={idx} style={{ fontSize: "11px" }}>
                          {tag}
                        </Tag>
                      ))}
                  </Space>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>

      <Modal
        title={selectedMemory?.title}
        open={!!selectedMemory}
        onCancel={() => {
          setSelectedMemory(null);
          setMemoryDetail(null);
        }}
        footer={[
          <Button
            key="close"
            onClick={() => {
              setSelectedMemory(null);
              setMemoryDetail(null);
            }}
          >
            Close
          </Button>,
        ]}
        width={800}
        style={{ top: "5vh" }}
        styles={{
          body: { maxHeight: "75vh", overflow: "auto" },
        }}
      >
        {detailLoading ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "200px",
            }}
          >
            <Spin size="large" />
          </div>
        ) : (
          <Tabs defaultActiveKey="content">
            <Tabs.TabPane tab="Content" key="content">
              {memoryDetail ? (
                <div>
                  <div
                    style={{
                      marginBottom: "16px",
                      padding: "20px",
                      background: "#f5f5f5",
                      borderRadius: "4px",
                    }}
                  >
                    <Space
                      direction="vertical"
                      style={{ width: "100%" }}
                      size="small"
                    >
                      <div>
                        <Text strong>ID: </Text>
                        <Text code>{selectedMemory.id}</Text>
                      </div>
                      <div>
                        <Text strong>Category: </Text>
                        <Tag color={getCategoryColor(selectedMemory.category)}>
                          {selectedMemory.category}
                        </Tag>
                      </div>
                      {selectedMemory.stage && (
                        <div>
                          <Text strong>Stage: </Text>
                          <Tag>{selectedMemory.stage}</Tag>
                        </div>
                      )}
                      <div>
                        <Text strong>Created: </Text>
                        <Text>{formatDate(selectedMemory.created_at)}</Text>
                      </div>
                      {selectedMemory.impact && (
                        <div>
                          <Text strong>Impact: </Text>
                          <Tag color={getImpactColor(selectedMemory.impact)}>
                            {selectedMemory.impact}
                          </Tag>
                        </div>
                      )}
                      {selectedMemory.tags &&
                        selectedMemory.tags.length > 0 && (
                          <div>
                            <Text strong>Tags: </Text>
                            <Space size="small">
                              {selectedMemory.tags.map((tag, idx) => (
                                <Tag key={idx}>{tag}</Tag>
                              ))}
                            </Space>
                          </div>
                        )}
                    </Space>
                  </div>
                  <div
                    style={{
                      maxHeight: "50vh",
                      overflow: "auto",
                      padding: "20px",
                      backgroundColor: "#fafafa",
                      borderRadius: "4px",
                    }}
                  >
                    <ReactMarkdown
                      remarkPlugins={[remarkGfm]}
                      rehypePlugins={[rehypeHighlight, rehypeRaw]}
                      style={{ lineHeight: "1.8", fontSize: "14px" }}
                    >
                      {memoryDetail.content}
                    </ReactMarkdown>
                  </div>
                </div>
              ) : (
                <Empty description="Failed to load memory detail" />
              )}
            </Tabs.TabPane>
            <Tabs.TabPane tab="Summary" key="summary">
              {selectedMemory && (
                <div>
                  <div style={{ marginBottom: "16px" }}>
                    <Text strong>Summary</Text>
                    <Divider style={{ margin: "8px 0" }} />
                    <div
                      style={{
                        padding: "20px",
                        backgroundColor: "#fafafa",
                        borderRadius: "4px",
                        maxHeight: "60vh",
                        overflow: "auto",
                      }}
                    >
                      <ReactMarkdown
                        remarkPlugins={[remarkGfm]}
                        rehypePlugins={[rehypeHighlight, rehypeRaw]}
                        style={{ lineHeight: "1.8", fontSize: "14px" }}
                      >
                        {selectedMemory.summary}
                      </ReactMarkdown>
                    </div>
                  </div>
                  {selectedMemory.file && (
                    <div>
                      <Text strong>File</Text>
                      <Divider style={{ margin: "8px 0" }} />
                      <Text code>{selectedMemory.file}</Text>
                    </div>
                  )}
                </div>
              )}
            </Tabs.TabPane>
          </Tabs>
        )}
      </Modal>
    </div>
  );
}

export default MemoryPanel;
