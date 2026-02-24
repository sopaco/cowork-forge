import { useCallback, useRef } from 'react';

export function useAutoScroll<T extends HTMLElement>(deps: unknown[] = []) {
  const ref = useRef<T>(null);
  
  const scrollToBottom = useCallback(() => {
    if (ref.current) {
      ref.current.scrollTop = ref.current.scrollHeight;
    }
  }, []);
  
  return { ref, scrollToBottom };
}

export function useAutoScrollOnDeps<T extends HTMLElement>(deps: unknown[] = []) {
  const ref = useRef<T>(null);
  
  const scrollToBottom = useCallback(() => {
    if (ref.current) {
      ref.current.scrollTop = ref.current.scrollHeight;
    }
  }, []);
  
  return { ref };
}
