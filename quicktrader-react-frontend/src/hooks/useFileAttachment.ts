import { useEffect, useRef, useState, useCallback } from 'react';

export interface FileAttachmentPayload {
  name: string;
  mimeType: string;
  data: string;
}

export interface UseFileAttachmentResult {
  trigger: () => void;
  pending: boolean;
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const result = reader.result;
      if (typeof result === 'string') {
        const base64 = result.split(',')[1];
        resolve(base64 ?? '');
      } else {
        resolve('');
      }
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

export function useFileAttachment(
  onFile: (file: FileAttachmentPayload) => void
): UseFileAttachmentResult {
  const [pending, setPending] = useState(false);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const onFileRef = useRef(onFile);
  onFileRef.current = onFile;

  const handleChange = useCallback((ev: Event) => {
    const input = ev.target as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    setPending(false);

    if (!file) return;

    setPending(true);
    fileToBase64(file)
      .then((data) => {
        onFileRef.current({
          name: file.name,
          mimeType: file.type || 'application/octet-stream',
          data,
        });
      })
      .catch(() => {
        // Caller can handle; we just reset pending
      })
      .finally(() => {
        setPending(false);
      });
  }, []);

  const trigger = useCallback(() => {
    const input = inputRef.current;
    if (input) {
      input.click();
    }
  }, []);

  useEffect(() => {
    const input = document.createElement('input');
    input.type = 'file';
    input.style.display = 'none';
    input.setAttribute('aria-hidden', 'true');
    document.body.appendChild(input);
    inputRef.current = input;

    input.addEventListener('change', handleChange);

    return () => {
      input.removeEventListener('change', handleChange);
      document.body.removeChild(input);
      inputRef.current = null;
    };
  }, [handleChange]);

  return { trigger, pending };
}
