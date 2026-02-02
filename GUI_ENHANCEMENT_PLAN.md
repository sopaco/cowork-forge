# GUI 交互增强方案

## 一、设计目标

1. **提高工作内容可见性**：在 session 维度查看 idea、prd、features、plan 等内容
2. **代码可视化**：在内置编辑器中查看编码阶段生成的内容
3. **实时预览**：验收阶段提供 preview 功能查看实际效果
4. **一键启动**：交付阶段提供启动程序的能力

## 二、界面布局设计

### 2.1 整体布局（三栏式）

```
┌────────────────────────────────────────────────────────────────────┐
│  Header: Cowork Forge  [Project Selector] [Session Selector]        │
├────────────────────────────────────────────────────────────────────┤
│                      │                                              │
│  左侧导航栏        │   主工作区                                     │
│  (Sidebar)         │   (Main Content)                              │
│                    │                                               │
│  📁 Sessions       │   ┌────────────────────────────────────────┐ │
│  📝 Artifacts      │   │                                        │ │
│  💬 Chat History   │   │   动态内容区域                          │ │
│  📂 Files          │   │   (根据当前选择动态显示)                │ │
│  ⚙️  Settings      │   │                                        │ │
│                    │   │                                        │ │
│                    │   └────────────────────────────────────────┘ │
│                    │                                               │
│                    │   ┌────────────────────────────────────────┐ │
│                    │   │   快速操作区                            │ │
│                    │   │   (根据当前阶段动态显示)                │ │
│                    │   └────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────────────────┤
│  底部状态栏: [当前阶段] [处理状态] [文件数量] [预览/启动按钮]       │
└────────────────────────────────────────────────────────────────────┘
```

### 2.2 侧边栏设计

#### Sessions 面板
```
📁 Sessions
├── 🟢 session-xxx (当前)
│   ├── 📝 idea
│   ├── 📋 prd
│   ├── 🎯 features
│   ├── 📐 design
│   └── 📋 plan
├── 🟢 session-yyy (已完成)
└── 🟡 session-zzz (失败)
```

#### Artifacts 面板（显示当前 session 的所有产物）
```
📝 Artifacts
├── 📄 idea.md
├── 📋 requirements.json
├── 🎯 feature_list.json
├── 📐 design_spec.json
├── 📋 implementation_plan.json
├── 📁 src/
│   ├── main.rs
│   └── ...
└── 📄 delivery_report.md
```

## 三、核心功能模块

### 3.1 工作内容查看器

#### 功能描述
- 在主工作区展示当前 session 的各个阶段的产物
- 支持多种格式：Markdown、JSON、代码
- 支持实时编辑（可选）
- 支持版本对比

#### 界面设计
```
┌─────────────────────────────────────────────────────┐
│ 📝 idea.md                          [编辑] [对比]    │
├─────────────────────────────────────────────────────┤
│                                                     │
│ # Project Idea                                      │
│                                                     │
│ ## Problem Statement                                │
│ ...                                                 │
│                                                     │
└─────────────────────────────────────────────────────┘
```

#### Tab 切换设计
```
[Idea] [Requirements] [Features] [Design] [Plan] [Code] [Report]
```

### 3.2 代码编辑器集成

#### 功能描述
- 使用 Monaco Editor（VS Code 同款编辑器）
- 支持语法高亮、代码补全
- 支持多文件标签页
- 支持实时保存
- 支持文件树导航

#### 界面设计
```
┌─────────────────────────────────────────────────────────────┐
│ 📂 src/                              [+新建文件] [刷新]        │
├──────────────────┬──────────────────────────────────────────┤
│                  │                                          │
│ 📁 src/          │ main.rs  *  Cargo.toml  README.md       │
│ ├── main.rs      │ ┌────────────────────────────────────┐  │
│ ├── lib.rs       │ │ fn main() {                          │  │
│ └── ...          │ │     println!("Hello World!");       │  │
│                  │ │     // ...                           │  │
│ 📁 tests/        │ │ }                                    │  │
│ └── ...          │ └────────────────────────────────────┘  │
│                  │                                          │
└──────────────────┴──────────────────────────────────────────┘
```

### 3.3 实时预览功能

#### 功能描述
- 自动检测 Web 项目
- 启动内置预览服务器
- 使用 iframe 嵌入预览
- 支持刷新、全屏

#### 界面设计
```
┌─────────────────────────────────────────────────────────────┐
│ 🌐 Preview                              [刷新] [全屏] [停止] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                                                       │  │
│  │         (iframe 显示预览内容)                        │  │
│  │                                                       │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 技术方案
1. **项目类型检测**：根据项目文件判断类型（HTML、React、Vue、Static 等）
2. **预览服务器**：使用内置 HTTP 服务器（Rust 实现）
3. **端口管理**：自动分配可用端口（5000-5999）
4. **热更新**：监控文件变化，自动刷新预览

### 3.4 一键启动功能

#### 功能描述
- 检测项目启动命令
- 在终端窗口中运行
- 显示运行日志
- 支持停止

#### 界面设计
```
┌─────────────────────────────────────────────────────────────┐
│ 🚀 Run Project                              [启动] [停止]    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [终端输出区域]                                             │
│  > cargo run                                               │
│    Compiling...                                            │
│    Finished.                                               │
│    Running `target/debug/myapp`                            │
│    Hello, World!                                           │
│    ^C (stopped)                                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 四、技术方案

### 4.1 后端 Tauri Commands

#### 新增 Commands

```rust
// 1. 获取 session 所有 artifacts
#[tauri::command]
async fn get_session_artifacts(session_id: String) -> Result<SessionArtifacts, String>

// 2. 读取文件内容
#[tauri::command]
async fn read_file_content(session_id: String, file_path: String) -> Result<String, String>

// 3. 保存文件内容
#[tauri::command]
async fn save_file_content(session_id: String, file_path: String, content: String) -> Result<(), String>

// 4. 启动预览服务器
#[tauri::command]
async fn start_preview(session_id: String) -> Result<PreviewInfo, String>

// 5. 停止预览服务器
#[tauri::command]
async fn stop_preview(session_id: String) -> Result<(), String>

// 6. 启动项目
#[tauri::command]
async fn start_project(session_id: String) -> Result<RunInfo, String>

// 7. 停止项目
#[tauri::command]
async fn stop_project(session_id: String) -> Result<(), String>

// 8. 获取项目文件树
#[tauri::command]
async fn get_file_tree(session_id: String) -> Result<FileTreeNode, String>

// 9. 执行项目命令（npm install, cargo build 等）
#[tauri::command]
async fn execute_project_command(session_id: String, command: String) -> Result<CommandResult, String>
```

#### 数据结构

```rust
#[derive(Serialize, Deserialize)]
pub struct SessionArtifacts {
    pub session_id: String,
    pub idea: Option<String>,
    pub requirements: Option<Requirements>,
    pub features: Option<FeatureList>,
    pub design: Option<DesignSpec>,
    pub plan: Option<ImplementationPlan>,
    pub code_files: Vec<FileInfo>,
    pub delivery_report: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PreviewInfo {
    pub url: String,
    pub port: u16,
    pub status: PreviewStatus,
}

#[derive(Serialize, Deserialize)]
pub enum PreviewStatus {
    Running,
    Stopped,
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub struct FileTreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileTreeNode>>,
}

#[derive(Serialize, Deserialize)]
pub struct RunInfo {
    pub status: RunStatus,
    pub process_id: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub enum RunStatus {
    Running,
    Stopped,
    Failed(String),
}
```

### 4.2 预览服务器实现

#### 技术选型
- 使用 Rust 的 `tiny_http` 或 `actix-web` 创建轻量级服务器
- 支持静态文件服务
- 支持 SPA 路由

#### 实现方案

```rust
// cowork-gui/src-tauri/src/preview_server.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tiny_http::{Server, Response, Request};
use std::thread;
use std::path::PathBuf;

pub struct PreviewServerManager {
    servers: Arc<Mutex<HashMap<String, PreviewServer>>>,
}

struct PreviewServer {
    port: u16,
    base_dir: PathBuf,
    handle: Option<thread::JoinHandle<()>>,
}

impl PreviewServerManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn start(&self, session_id: String, base_dir: PathBuf) -> Result<u16, String> {
        let port = self.find_available_port()?;
        let base_dir_clone = base_dir.clone();
        let session_id_clone = session_id.clone();
        
        let server = Server::http(format!("0.0.0.0:{}", port))
            .map_err(|e| format!("Failed to create server: {}", e))?;
        
        let handle = thread::spawn(move || {
            for request in server.incoming_requests() {
                let response = Self::handle_request(&request, &base_dir_clone);
                let _ = request.respond(response);
            }
        });
        
        let servers = self.servers.lock().unwrap();
        servers.insert(session_id_clone, PreviewServer {
            port,
            base_dir,
            handle: Some(handle),
        });
        
        Ok(port)
    }
    
    pub fn stop(&self, session_id: String) -> Result<(), String> {
        let mut servers = self.servers.lock().unwrap();
        servers.remove(&session_id)
            .ok_or_else(|| "Server not found".to_string())
            .map(|_| ())
    }
    
    fn find_available_port(&self) -> Result<u16, String> {
        for port in 5000..6000 {
            // 检查端口是否可用
            if Self::is_port_available(port) {
                return Ok(port);
            }
        }
        Err("No available port".to_string())
    }
    
    fn is_port_available(port: u16) -> bool {
        // 实现端口检查逻辑
        true
    }
    
    fn handle_request(request: &Request, base_dir: &PathBuf) -> Response {
        // 处理请求，返回静态文件
        Response::from_data(Vec::new())
    }
}
```

### 4.3 项目运行器

#### 技术选型
- 使用 `tokio::process::Command` 执行命令
- 使用 `pty` (pseudo-terminal) 提供终端体验

#### 实现方案

```rust
// cowork-gui/src-tauri/src/project_runner.rs

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::process::{Command, Child};
use tokio::sync::mpsc;

pub struct ProjectRunner {
    processes: Arc<Mutex<HashMap<String, ProjectProcess>>>,
}

struct ProjectProcess {
    child: Child,
    output_tx: mpsc::UnboundedSender<String>,
}

impl ProjectRunner {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn start(&self, session_id: String, command: String) -> Result<u32, String> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .current_dir(format!(".cowork/sessions/{}/code", session_id))
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start: {}", e))?;
        
        let pid = child.id().unwrap();
        
        // 启动输出读取任务
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        
        // ... 读取输出并发送到前端
        
        let mut processes = self.processes.lock().unwrap();
        processes.insert(session_id, ProjectProcess {
            child,
            output_tx: todo!(),
        });
        
        Ok(pid)
    }
    
    pub async fn stop(&self, session_id: String) -> Result<(), String> {
        let mut processes = self.processes.lock().unwrap();
        let process = processes.remove(&session_id)
            .ok_or_else(|| "Process not found".to_string())?;
        
        process.child.kill()
            .await
            .map_err(|e| format!("Failed to stop: {}", e))?;
        
        Ok(())
    }
}
```

### 4.4 前端组件设计

#### 主要组件

```jsx
// ArtifactsViewer.jsx - 工作内容查看器
const ArtifactsViewer = ({ sessionId, activeTab }) => {
  return (
    <div className="artifacts-viewer">
      <Tabs>
        <TabPane tab="Idea" key="idea">
          <MarkdownViewer content={ideaContent} />
        </TabPane>
        <TabPane tab="Requirements" key="requirements">
          <JsonViewer data={requirements} />
        </TabPane>
        <TabPane tab="Features" key="features">
          <FeatureTable features={features} />
        </TabPane>
        <TabPane tab="Design" key="design">
          <DesignViewer design={design} />
        </TabPane>
        <TabPane tab="Plan" key="plan">
          <PlanViewer plan={plan} />
        </TabPane>
      </Tabs>
    </div>
  );
};

// CodeEditor.jsx - 代码编辑器
const CodeEditor = ({ sessionId, activeFile }) => {
  const [content, setContent] = useState('');
  const editorRef = useRef(null);
  
  return (
    <div className="code-editor">
      <FileTree onFileSelect={handleFileSelect} />
      <Editor
        value={content}
        onChange={handleChange}
        language={getLanguage(activeFile)}
        theme="vs-dark"
        options={{
          minimap: { enabled: true },
          fontSize: 14,
        }}
      />
    </div>
  );
};

// PreviewPanel.jsx - 预览面板
const PreviewPanel = ({ sessionId }) => {
  const [previewUrl, setPreviewUrl] = useState(null);
  const [isRunning, setIsRunning] = useState(false);
  
  return (
    <div className="preview-panel">
      <div className="preview-header">
        <h3>🌐 Preview</h3>
        <Button onClick={handleRefresh}>Refresh</Button>
        <Button onClick={handleFullscreen}>Fullscreen</Button>
        <Button onClick={handleStop}>Stop</Button>
      </div>
      <iframe src={previewUrl} className="preview-frame" />
    </div>
  );
};

// ProjectRunnerPanel.jsx - 项目运行器
const ProjectRunnerPanel = ({ sessionId }) => {
  const [logs, setLogs] = useState([]);
  const [isRunning, setIsRunning] = useState(false);
  
  return (
    <div className="runner-panel">
      <div className="runner-header">
        <h3>🚀 Run Project</h3>
        <Button onClick={handleStart}>Start</Button>
        <Button onClick={handleStop}>Stop</Button>
      </div>
      <Terminal logs={logs} />
    </div>
  );
};
```

## 五、交互流程

### 5.1 Session 查看流程

```
用户点击 Session
  ↓
侧边栏显示该 Session 的 Artifacts
  ↓
主工作区默认显示 Idea Tab
  ↓
用户点击其他 Tab（Requirements/Features/Design/Plan）
  ↓
加载对应数据并显示
  ↓
用户可以编辑（可选）
  ↓
点击保存按钮，更新文件
```

### 5.2 编码阶段流程

```
编码阶段开始
  ↓
主工作区切换到代码编辑器视图
  ↓
显示项目文件树
  ↓
用户点击文件
  ↓
加载文件内容到编辑器
  ↓
实时保存到后台
  ↓
文件变化时自动刷新文件树
```

### 5.3 预览流程

```
用户点击预览按钮
  ↓
后端检测项目类型
  ↓
启动预览服务器
  ↓
返回预览 URL
  ↓
前端在 iframe 中加载 URL
  ↓
用户可以刷新、全屏查看
  ↓
编码阶段，文件变化自动刷新
```

### 5.4 启动流程

```
用户点击启动按钮
  ↓
后端检测项目启动命令
  ↓
在终端中执行命令
  ↓
实时输出日志到前端
  ↓
用户可以查看运行结果
  ↓
点击停止按钮终止进程
```

## 六、不影响 CLI 的设计原则

### 6.1 所有新增功能都在 GUI 层

- 新增的 Tauri Commands 不影响 CLI
- 前端组件只在 GUI 中使用
- 不修改 cowork-core 的核心逻辑

### 6.2 使用现有存储 API

- 所有数据读取使用 cowork-core 提供的 storage API
- 不修改存储格式
- 不添加新的存储文件

### 6.3 预览和运行功能可选

- CLI 继续保持简洁，不提供预览和运行功能
- GUI 提供额外的便利功能
- CLI 用户可以手动执行启动命令

## 七、实施计划

### Phase 1: 基础设施（1-2天）
1. 新增 Tauri Commands
2. 实现数据加载 API
3. 实现文件读取/保存 API

### Phase 2: 工作内容查看器（2-3天）
1. 实现 ArtifactsViewer 组件
2. 实现 MarkdownViewer 组件
3. 实现 JsonViewer 组件
4. 实现各阶段查看器

### Phase 3: 代码编辑器（2-3天）
1. 集成 Monaco Editor
2. 实现文件树组件
3. 实现多文件标签页
4. 实现实时保存

### Phase 4: 预览功能（2-3天）
1. 实现预览服务器
2. 实现 PreviewPanel 组件
3. 实现项目类型检测
4. 实现热更新

### Phase 5: 项目运行器（2-3天）
1. 实现项目运行器
2. 实现 RunnerPanel 组件
3. 实现终端组件
4. 实现命令检测

### Phase 6: 测试和优化（2-3天）
1. 功能测试
2. 性能优化
3. 用户体验优化
4. 文档完善

## 八、技术栈

### 前端
- React 18
- Monaco Editor（代码编辑器）
- react-markdown（Markdown 渲染）
- react-json-view（JSON 可视化）
- Ant Design（UI 组件库）
- Tailwind CSS（样式）

### 后端
- Tauri 2.0
- tiny-http（预览服务器）
- tokio（异步运行时）
- serde（序列化）

## 九、注意事项

1. **性能优化**
   - 大文件使用流式加载
   - 文件树使用虚拟滚动
   - 编辑器使用 Web Worker

2. **安全性**
   - 预览服务器限制访问本地文件
   - 文件读写只限制在 session 目录
   - 命令执行白名单机制

3. **兼容性**
   - 支持多种项目类型（HTML、React、Vue、Rust 等）
   - 跨平台支持（Windows、macOS、Linux）

4. **可扩展性**
   - 预留插件接口
   - 支持自定义命令
   - 支持自定义编辑器主题