import { useRef, useEffect, useCallback } from 'react';

export function useRefreshTrigger(callback: () => void, trigger: number) {
  const prevTriggerRef = useRef(trigger);
  
  useEffect(() => {
    if (trigger !== prevTriggerRef.current) {
      prevTriggerRef.current = trigger;
      callback();
    }
  }, [trigger, callback]);
}

export function useRefreshTriggerValue(initialValue = 0): [number, () => void] {
  const triggerRef = useRef(initialValue);
  
  const trigger = useCallback(() => {
    triggerRef.current += 1;
    return triggerRef.current;
  }, []);
  
  return [triggerRef.current, trigger];
}
