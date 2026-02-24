import { useState, useCallback } from 'react';

export function useLoading<T>(
  asyncFn: (...args: unknown[]) => Promise<T>
): [typeof asyncFn, boolean, Error | null] {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  
  const wrappedFn = useCallback(
    async (...args: Parameters<typeof asyncFn>) => {
      setLoading(true);
      setError(null);
      try {
        const result = await asyncFn(...args);
        setLoading(false);
        return result;
      } catch (e) {
        setError(e instanceof Error ? e : new Error(String(e)));
        setLoading(false);
        throw e;
      }
    },
    [asyncFn]
  );
  
  return [wrappedFn, loading, error];
}

export function useAsyncAction<T>(
  asyncFn: () => Promise<T>,
  deps: unknown[] = []
): [() => Promise<void>, boolean, Error | null] {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  
  const execute = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await asyncFn();
    } catch (e) {
      setError(e instanceof Error ? e : new Error(String(e)));
      throw e;
    } finally {
      setLoading(false);
    }
  }, deps);
  
  return [execute, loading, error];
}
