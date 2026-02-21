import { message } from 'antd';

interface ErrorMapping {
  userMessage: string;
  solution: string;
  severity: 'error' | 'warning';
}

interface ParsedError {
  originalError: string;
  userMessage: string;
  solution: string;
  severity: 'error' | 'warning';
}

const ERROR_MAPPINGS: Record<string, ErrorMapping> = {
  'Failed to read file': { userMessage: '无法读取文件', solution: '请检查文件路径是否正确，确保文件存在且有读取权限', severity: 'error' },
  'Failed to write file': { userMessage: '无法写入文件', solution: '请检查是否有写入权限，或磁盘空间是否充足', severity: 'error' },
  'Failed to create directory': { userMessage: '无法创建目录', solution: '请检查路径是否有效，或是否有创建目录的权限', severity: 'error' },
  'No project found': { userMessage: '未找到项目', solution: '请先创建一个项目，或确保项目目录存在', severity: 'warning' },
  'No active session found': { userMessage: '未找到活动会话', solution: '请先启动一个项目会话', severity: 'warning' },
  'Session not found': { userMessage: '会话不存在', solution: '该会话可能已被删除，请刷新页面后重试', severity: 'warning' },
  'Failed to load project index': { userMessage: '无法加载项目索引', solution: '项目可能已损坏，请尝试重新初始化项目', severity: 'error' },
  'Failed to save project index': { userMessage: '无法保存项目索引', solution: '请检查磁盘空间和写入权限', severity: 'error' },
  'Failed to load LLM config': { userMessage: '无法加载 AI 配置', solution: '请检查 config.toml 文件是否存在且格式正确', severity: 'error' },
  'Failed to create LLM client': { userMessage: '无法创建 AI 客户端', solution: '请检查 API 密钥是否正确，网络连接是否正常', severity: 'error' },
  'LLM request failed': { userMessage: 'AI 请求失败', solution: '请检查网络连接和 API 配额', severity: 'error' },
  'Failed to create pipeline': { userMessage: '无法创建工作流', solution: '请检查项目配置是否完整', severity: 'error' },
  'Pipeline execution failed': { userMessage: '工作流执行失败', solution: '请查看日志了解详细错误信息，然后重试', severity: 'error' },
  'Failed to read session index': { userMessage: '无法读取会话索引', solution: '会话数据可能已损坏，请尝试从备份恢复', severity: 'error' },
  'Failed to save session memory index': { userMessage: '无法保存会话记忆', solution: '请检查磁盘空间和写入权限', severity: 'error' },
  'Failed to start preview server': { userMessage: '无法启动预览服务器', solution: '端口可能被占用，请尝试使用其他端口', severity: 'error' },
  'Failed to stop preview server': { userMessage: '无法停止预览服务器', solution: '服务器可能已经停止，请刷新页面', severity: 'warning' },
  'Failed to start': { userMessage: '无法启动项目', solution: '请检查项目配置和依赖是否正确安装', severity: 'error' },
  'Failed to stop': { userMessage: '无法停止项目', solution: '进程可能已经停止，请刷新页面', severity: 'warning' },
  'Code directory not found': { userMessage: '代码目录不存在', solution: '项目可能还没有生成代码，请先运行代码生成', severity: 'warning' },
  'Failed to acquire lock': { userMessage: '无法获取项目锁', solution: '其他操作可能正在进行，请稍后重试', severity: 'warning' },
  'Failed to open project': { userMessage: '无法打开项目', solution: '项目路径可能不存在或无效', severity: 'error' },
  'Failed to get project root': { userMessage: '无法获取项目根目录', solution: '请确保在有效的项目目录中运行', severity: 'error' },
  'Failed to get current dir': { userMessage: '无法获取当前目录', solution: '请确保有权限访问当前目录', severity: 'error' },
};

export function parseError(error: unknown): ParsedError {
  const errorStr = String(error);
  for (const [key, mapping] of Object.entries(ERROR_MAPPINGS)) {
    if (errorStr.includes(key)) {
      return { originalError: errorStr, ...mapping };
    }
  }
  return { originalError: errorStr, userMessage: '发生未知错误', solution: '请查看详细错误信息，如果问题持续存在，请联系支持', severity: 'error' };
}

export function showError(error: unknown, customMessage: string | null = null): void {
  const parsed = parseError(error);
  const displayMessage = customMessage || parsed.userMessage;
  message.error({
    content: `${displayMessage}\n${parsed.solution}`,
    duration: 5
  });
  console.error('[Error Handler]', parsed.originalError);
}

export function showWarning(warning: unknown): void {
  const parsed = parseError(warning);
  message.warning({
    content: `${parsed.userMessage}\n${parsed.solution}`,
    duration: 3
  });
}

export function showSuccess(msg: string): void {
  message.success(msg, 3);
}

export function showInfo(info: string): void {
  message.info(info, 3);
}

export async function tryExecute<T>(fn: () => Promise<T>, customErrorMessage: string | null = null): Promise<T | null> {
  try {
    return await fn();
  } catch (error) {
    showError(error, customErrorMessage);
    return null;
  }
}

export default { parseError, showError, showWarning, showSuccess, showInfo, tryExecute };
