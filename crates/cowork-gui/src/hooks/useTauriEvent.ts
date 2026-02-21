import { useEffect, useRef } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export function useTauriEvent<T = unknown>(
  event: string, 
  handler: (payload: T) => void,
  deps: unknown[] = []
) {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;
  
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    
    const setup = async () => {
      unlisten = await listen<T>(event, (e) => {
        handlerRef.current(e.payload);
      });
    };
    
    setup();
    
    return () => {
      unlisten?.();
    };
  }, [event, ...deps]);
}

export function useTauriEvents(
  events: Array<{ event: string; handler: (payload: unknown) => void }>
) {
  useEffect(() => {
    const unlisteners: Promise<UnlistenFn>[] = [];
    
    events.forEach(({ event, handler }) => {
      unlisteners.push(listen(event, (e) => handler(e.payload)));
    });
    
    return () => {
      Promise.all(unlisteners).then((fns) => {
        fns.forEach((fn) => fn());
      });
    };
  }, []);
}
