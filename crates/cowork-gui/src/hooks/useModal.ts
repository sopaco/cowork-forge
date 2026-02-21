import { useState, useCallback } from 'react';

export function useModal(): [boolean, () => void, () => void, () => void] {
  const [visible, setVisible] = useState(false);
  
  const open = useCallback(() => setVisible(true), []);
  const close = useCallback(() => setVisible(false), []);
  const toggle = useCallback(() => setVisible((v) => !v), []);
  
  return [visible, open, close, toggle];
}

export function useDrawer(): [boolean, () => void, () => void] {
  const [visible, setVisible] = useState(false);
  
  const open = useCallback(() => setVisible(true), []);
  const close = useCallback(() => setVisible(false), []);
  
  return [visible, open, close];
}
