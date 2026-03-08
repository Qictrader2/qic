/* elm-pkg-js
port startRecording : () -> Cmd msg
port stopRecording : Bool -> Cmd msg
port audioRecorded : (D.Value -> msg) -> Sub msg
port audioRecordingError : (String -> msg) -> Sub msg
*/

window.elmPkgJs = window.elmPkgJs || {};
window.elmPkgJs['voice-recording'] = {
    init: function(app) {
        if (!app.ports || !app.ports.startRecording || !app.ports.stopRecording
            || !app.ports.audioRecorded || !app.ports.audioRecordingError) {
            console.warn('elm-pkg-js [voice-recording]: required ports not found');
            return;
        }

        var mediaRecorder = null;
        var audioChunks = [];
        var maxDurationTimer = null;
        var recordingCancelled = false;
        var MAX_RECORDING_MS = 10 * 60 * 1000; // 10 minutes
        var MAX_BLOB_BYTES = 25 * 1024 * 1024;  // 25 MB

        app.ports.startRecording.subscribe(function() {
            recordingCancelled = false;
            // Clean up any existing recording before starting a new one
            if (maxDurationTimer) { clearTimeout(maxDurationTimer); maxDurationTimer = null; }
            if (mediaRecorder && mediaRecorder.state !== 'inactive') {
                mediaRecorder.onstop = function() {}; // prevent normal onstop handler
                mediaRecorder.stop();
            }
            if (mediaRecorder && mediaRecorder.stream) {
                mediaRecorder.stream.getTracks().forEach(function(track) { track.stop(); });
            }
            mediaRecorder = null;
            audioChunks = [];

            if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
                app.ports.audioRecordingError.send(
                    'Voice recording is not supported in this browser. '
                    + 'Ensure you are using HTTPS or localhost, and a modern browser.'
                );
                return;
            }

            try {
            navigator.mediaDevices.getUserMedia({ audio: true })
                .then(function(stream) {
                    if (recordingCancelled) {
                        stream.getTracks().forEach(function(track) { track.stop(); });
                        return;
                    }
                    var mimeType = 'audio/webm;codecs=opus';
                    if (!MediaRecorder.isTypeSupported(mimeType)) mimeType = 'audio/webm';
                    if (!MediaRecorder.isTypeSupported(mimeType)) mimeType = 'audio/ogg;codecs=opus';
                    if (!MediaRecorder.isTypeSupported(mimeType)) mimeType = '';

                    var options = mimeType ? { mimeType: mimeType } : {};
                    mediaRecorder = new MediaRecorder(stream, options);

                    mediaRecorder.ondataavailable = function(event) {
                        if (event.data.size > 0) audioChunks.push(event.data);
                    };

                    mediaRecorder.onstop = function() {
                        var capturedMime = mediaRecorder ? mediaRecorder.mimeType : 'audio/webm';
                        var effectiveMime = capturedMime || 'audio/webm';
                        var blob = new Blob(audioChunks, { type: effectiveMime });
                        audioChunks = [];
                        if (blob.size > MAX_BLOB_BYTES) {
                            stream.getTracks().forEach(function(track) { track.stop(); });
                            mediaRecorder = null;
                            app.ports.audioRecordingError.send(
                                'Recording too large (' + Math.round(blob.size / 1024 / 1024) + ' MB). Maximum is '
                                + Math.round(MAX_BLOB_BYTES / 1024 / 1024) + ' MB.'
                            );
                            return;
                        }
                        var reader = new FileReader();
                        reader.onloadend = function() {
                            if (!reader.result || typeof reader.result !== 'string') {
                                app.ports.audioRecordingError.send('Failed to read recorded audio data');
                                return;
                            }
                            var base64 = reader.result.split(',')[1];
                            app.ports.audioRecorded.send({
                                data: base64,
                                mimeType: effectiveMime
                            });
                        };
                        reader.onerror = function() {
                            app.ports.audioRecordingError.send('Failed to read recorded audio data');
                        };
                        reader.readAsDataURL(blob);
                        stream.getTracks().forEach(function(track) { track.stop(); });
                        mediaRecorder = null;
                    };

                    mediaRecorder.onerror = function(event) {
                        if (maxDurationTimer) { clearTimeout(maxDurationTimer); maxDurationTimer = null; }
                        app.ports.audioRecordingError.send('Recording error: ' + (event.error ? event.error.message : 'unknown'));
                        stream.getTracks().forEach(function(track) { track.stop(); });
                        audioChunks = [];
                        mediaRecorder = null;
                    };

                    mediaRecorder.start();
                    // Auto-stop after max duration to prevent huge recordings
                    maxDurationTimer = setTimeout(function() {
                        if (mediaRecorder && mediaRecorder.state === 'recording') {
                            mediaRecorder.stop();
                        }
                        maxDurationTimer = null;
                    }, MAX_RECORDING_MS);
                })
                .catch(function(err) {
                    app.ports.audioRecordingError.send('Microphone access denied: ' + err.message);
                });
            } catch (e) {
                app.ports.audioRecordingError.send(
                    'Voice recording is not available. '
                    + 'Ensure you are using HTTPS or localhost.'
                );
            }
        });

        app.ports.stopRecording.subscribe(function(shouldCancel) {
            if (maxDurationTimer) { clearTimeout(maxDurationTimer); maxDurationTimer = null; }
            if (shouldCancel === true) { recordingCancelled = true; }
            if (mediaRecorder && mediaRecorder.state === 'recording') {
                if (shouldCancel === true) {
                    mediaRecorder.onstop = function() {
                        var stream = mediaRecorder.stream;
                        if (stream) {
                            stream.getTracks().forEach(function(track) { track.stop(); });
                        }
                        audioChunks = [];
                        mediaRecorder = null;
                    };
                    mediaRecorder.stop();
                } else {
                    mediaRecorder.stop();
                }
            }
        });
    }
};
