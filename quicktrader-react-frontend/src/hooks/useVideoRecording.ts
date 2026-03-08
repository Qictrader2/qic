import { useEffect, useRef, useState, useCallback } from 'react';

const MIME_PRIORITY: string[] = [
  'video/webm;codecs=vp9,opus',
  'video/webm',
  'video/mp4',
];

function selectMimeType(): string {
  for (const mime of MIME_PRIORITY) {
    if (MediaRecorder.isTypeSupported(mime)) {
      return mime;
    }
  }
  return '';
}

function blobToBase64(blob: Blob): Promise<string> {
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
    reader.readAsDataURL(blob);
  });
}

export interface UseVideoRecordingResult {
  isRecording: boolean;
  start: () => void;
  stop: () => Promise<{ data: string; mimeType: string }>;
  cancel: () => void;
  error: string | null;
}

export function useVideoRecording(): UseVideoRecordingResult {
  const [isRecording, setIsRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const recorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);

  const start = useCallback(() => {
    setError(null);
    navigator.mediaDevices
      .getUserMedia({ video: true, audio: true })
      .then((stream) => {
        streamRef.current = stream;
        const mimeType = selectMimeType() || 'video/webm';
        const options: MediaRecorderOptions = mimeType ? { mimeType } : {};
        const recorder = new MediaRecorder(stream, options);
        recorderRef.current = recorder;
        chunksRef.current = [];

        recorder.ondataavailable = (ev: BlobEvent) => {
          if (ev.data.size > 0) {
            chunksRef.current.push(ev.data);
          }
        };

        recorder.start();
        setIsRecording(true);
      })
      .catch((err: Error) => {
        setError(err.message ?? 'Failed to access camera/microphone');
      });
  }, []);

  const stop = useCallback((): Promise<{ data: string; mimeType: string }> => {
    return new Promise((resolve, reject) => {
      const recorder = recorderRef.current;
      const stream = streamRef.current;

      if (!recorder || recorder.state === 'inactive') {
        resolve({ data: '', mimeType: 'video/webm' });
        return;
      }

      const mimeType = recorder.mimeType || 'video/webm';

      recorder.onstop = async () => {
        if (stream) {
          stream.getTracks().forEach((t) => t.stop());
        }
        streamRef.current = null;
        recorderRef.current = null;
        setIsRecording(false);

        const blob = new Blob(chunksRef.current, { type: mimeType });
        try {
          const data = await blobToBase64(blob);
          resolve({ data, mimeType });
        } catch (e) {
          reject(e);
        }
      };

      recorder.stop();
    });
  }, []);

  const cancel = useCallback(() => {
    const recorder = recorderRef.current;
    const stream = streamRef.current;
    if (recorder && recorder.state !== 'inactive') {
      recorder.stop();
    }
    if (stream) {
      stream.getTracks().forEach((t) => t.stop());
    }
    streamRef.current = null;
    recorderRef.current = null;
    chunksRef.current = [];
    setIsRecording(false);
  }, []);

  useEffect(() => {
    return () => {
      const stream = streamRef.current;
      if (stream) {
        stream.getTracks().forEach((t) => t.stop());
      }
    };
  }, []);

  return { isRecording, start, stop, cancel, error };
}
