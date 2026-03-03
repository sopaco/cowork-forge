/**
 * Unified error handling utilities
 */

import type { MessageInstance } from 'antd/es/message/interface';
import { message as globalMessage } from 'antd';

export class AppError extends Error {
  constructor(
    message: string,
    public code?: string,
    public cause?: unknown
  ) {
    super(message);
    this.name = 'AppError';
  }
}

export interface ErrorHandlerOptions {
  showMessage?: boolean;
  fallback?: () => void;
  onError?: (error: AppError) => void;
}

/**
 * Handle errors with consistent messaging
 */
export function handleError(
  error: unknown,
  message: MessageInstance,
  context: string,
  options: ErrorHandlerOptions = {}
): AppError {
  const { showMessage = true, fallback, onError } = options;

  let appError: AppError;

  if (error instanceof AppError) {
    appError = error;
  } else if (error instanceof Error) {
    appError = new AppError(error.message, undefined, error);
  } else if (typeof error === 'string') {
    appError = new AppError(error);
  } else {
    appError = new AppError('An unknown error occurred', undefined, error);
  }

  // Log to console for debugging
  console.error(`[${context}]`, appError);

  // Show user-friendly message
  if (showMessage) {
    message.error(`${context}: ${appError.message}`);
  }

  // Execute fallback if provided
  if (fallback) {
    fallback();
  }

  // Execute custom error handler
  if (onError) {
    onError(appError);
  }

  return appError;
}

/**
 * Async error wrapper for consistent error handling
 */
export async function withErrorHandling<T>(
  fn: () => Promise<T>,
  message: MessageInstance,
  context: string,
  options: ErrorHandlerOptions = {}
): Promise<T | null> {
  try {
    return await fn();
  } catch (error) {
    handleError(error, message, context, options);
    return null;
  }
}

/**
 * Format error message for display
 */
export function formatErrorMessage(error: unknown): string {
  if (error instanceof AppError) {
    return error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  return 'An unknown error occurred';
}

/**
 * Simple message utilities for backward compatibility
 */

export function showSuccess(msg: string): void {
  globalMessage.success(msg);
}

export function showError(error: unknown): void {
  const errorMsg = formatErrorMessage(error);
  globalMessage.error(errorMsg);
}

export function showWarning(msg: string): void {
  globalMessage.warning(msg);
}

export function showInfo(msg: string): void {
  globalMessage.info(msg);
}

/**
 * Try-execute wrapper with automatic error handling
 */
export async function tryExecute<T>(
  fn: () => Promise<T>,
  errorContext?: string
): Promise<T | null> {
  try {
    return await fn();
  } catch (error) {
    const context = errorContext || 'Operation failed';
    showError(`${context}: ${formatErrorMessage(error)}`);
    console.error(`[${context}]`, error);
    return null;
  }
}
