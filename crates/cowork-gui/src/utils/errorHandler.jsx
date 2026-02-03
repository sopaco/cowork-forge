// Error Handler - Convert technical errors to user-friendly messages
import { message } from 'antd';

// Error message mappings and solutions
const ERROR_MAPPINGS = {
  // File system errors
  'Failed to read file': {
    userMessage: '无法读取文件',
    solution: '请检查文件路径是否正确，确保文件存在且有读取权限',
    severity: 'error'
  },
  'Failed to write file': {
    userMessage: '无法写入文件',
    solution: '请检查是否有写入权限，或磁盘空间是否充足',
    severity: 'error'
  },
  'Failed to create directory': {
    userMessage: '无法创建目录',
    solution: '请检查路径是否有效，或是否有创建目录的权限',
    severity: 'error'
  },
  
  // Session errors
  'No project found': {
    userMessage: '未找到项目',
    solution: '请先创建一个项目，或确保项目目录存在',
    severity: 'warning'
  },
  'No active session found': {
    userMessage: '未找到活动会话',
    solution: '请先启动一个项目会话',
    severity: 'warning'
  },
  'Session not found': {
    userMessage: '会话不存在',
    solution: '该会话可能已被删除，请刷新页面后重试',
    severity: 'warning'
  },
  
  // Project errors
  'Failed to load project index': {
    userMessage: '无法加载项目索引',
    solution: '项目可能已损坏，请尝试重新初始化项目',
    severity: 'error'
  },
  'Failed to save project index': {
    userMessage: '无法保存项目索引',
    solution: '请检查磁盘空间和写入权限',
    severity: 'error'
  },
  
  // LLM errors
  'Failed to load LLM config': {
    userMessage: '无法加载 AI 配置',
    solution: '请检查 config.toml 文件是否存在且格式正确',
    severity: 'error'
  },
  'Failed to create LLM client': {
    userMessage: '无法创建 AI 客户端',
    solution: '请检查 API 密钥是否正确，网络连接是否正常',
    severity: 'error'
  },
  'LLM request failed': {
    userMessage: 'AI 请求失败',
    solution: '请检查网络连接和 API 配额',
    severity: 'error'
  },
  
  // Pipeline errors
  'Failed to create pipeline': {
    userMessage: '无法创建工作流',
    solution: '请检查项目配置是否完整',
    severity: 'error'
  },
  'Pipeline execution failed': {
    userMessage: '工作流执行失败',
    solution: '请查看日志了解详细错误信息，然后重试',
    severity: 'error'
  },
  
  // Memory errors
  'Failed to read session index': {
    userMessage: '无法读取会话索引',
    solution: '会话数据可能已损坏，请尝试从备份恢复',
    severity: 'error'
  },
  'Failed to save session memory index': {
    userMessage: '无法保存会话记忆',
    solution: '请检查磁盘空间和写入权限',
    severity: 'error'
  },
  
  // Preview errors
  'Failed to start preview server': {
    userMessage: '无法启动预览服务器',
    solution: '端口可能被占用，请尝试使用其他端口',
    severity: 'error'
  },
  'Failed to stop preview server': {
    userMessage: '无法停止预览服务器',
    solution: '服务器可能已经停止，请刷新页面',
    severity: 'warning'
  },
  
  // Runner errors
  'Failed to start': {
    userMessage: '无法启动项目',
    solution: '请检查项目配置和依赖是否正确安装',
    severity: 'error'
  },
  'Failed to stop': {
    userMessage: '无法停止项目',
    solution: '进程可能已经停止，请刷新页面',
    severity: 'warning'
  },
  'Code directory not found': {
    userMessage: '代码目录不存在',
    solution: '项目可能还没有生成代码，请先运行代码生成',
    severity: 'warning'
  },
  
  // Project manager errors
  'Failed to acquire lock': {
    userMessage: '无法获取项目锁',
    solution: '其他操作可能正在进行，请稍后重试',
    severity: 'warning'
  },
  'Failed to open project': {
    userMessage: '无法打开项目',
    solution: '项目路径可能不存在或无效',
    severity: 'error'
  },
  
  // Generic errors
  'Failed to get project root': {
    userMessage: '无法获取项目根目录',
    solution: '请确保在有效的项目目录中运行',
    severity: 'error'
  },
  'Failed to get current dir': {
    userMessage: '无法获取当前目录',
    solution: '请确保有权限访问当前目录',
    severity: 'error'
  },
};

/**
 * Parse error and return user-friendly information
 * @param {string} error - The error message from backend
 * @returns {object} - Parsed error information
 */
export function parseError(error) {
  const errorStr = String(error);
  
  // Try to find a matching error mapping
  for (const [key, mapping] of Object.entries(ERROR_MAPPINGS)) {
    if (errorStr.includes(key)) {
      return {
        originalError: errorStr,
        userMessage: mapping.userMessage,
        solution: mapping.solution,
        severity: mapping.severity
      };
    }
  }
  
  // If no match found, return generic error
  return {
    originalError: errorStr,
    userMessage: '发生未知错误',
    solution: '请查看详细错误信息，如果问题持续存在，请联系支持',
    severity: 'error'
  };
}

/**
 * Display error to user with user-friendly message and solution
 * @param {string} error - The error message
 * @param {string} customMessage - Optional custom message to override default
 */
export function showError(error, customMessage = null) {
  const parsed = parseError(error);
  const displayMessage = customMessage || parsed.userMessage;
  
  // Show error with solution
  message.error({
    content: (
      <div>
        <div style={{ fontWeight: 'bold', marginBottom: 4 }}>{displayMessage}</div>
        <div style={{ fontSize: '12px', color: '#888' }}>
          {parsed.solution}
        </div>
      </div>
    ),
    duration: 5
  });
  
  // Log original error for debugging
  console.error('[Error Handler]', parsed.originalError);
}

/**
 * Display warning to user
 * @param {string} warning - The warning message
 */
export function showWarning(warning) {
  const parsed = parseError(warning);
  
  message.warning({
    content: (
      <div>
        <div style={{ fontWeight: 'bold', marginBottom: 4 }}>{parsed.userMessage}</div>
        <div style={{ fontSize: '12px', color: '#888' }}>
          {parsed.solution}
        </div>
      </div>
    ),
    duration: 3
  });
}

/**
 * Display success message
 * @param {string} success - The success message
 */
export function showSuccess(success) {
  message.success(success, 3);
}

/**
 * Display info message
 * @param {string} info - The info message
 */
export function showInfo(info) {
  message.info(info, 3);
}

/**
 * Try-execute async function with error handling
 * @param {Function} fn - Async function to execute
 * @param {string} customErrorMessage - Optional custom error message
 * @returns {Promise} - Promise with result or null on error
 */
export async function tryExecute(fn, customErrorMessage = null) {
  try {
    return await fn();
  } catch (error) {
    showError(error, customErrorMessage);
    return null;
  }
}

export default {
  parseError,
  showError,
  showWarning,
  showSuccess,
  showInfo,
  tryExecute
};