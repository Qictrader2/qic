/* elm-pkg-js
port startVideoRecording : () -> Cmd msg
port stopVideoRecording : Bool -> Cmd msg
port videoRecorded : (D.Value -> msg) -> Sub msg
port audioRecordingError : (String -> msg) -> Sub msg
*/

window.elmPkgJs = window.elmPkgJs || {};
window.elmPkgJs['video-recording'] = {
    init: function(app) {
        if (!app.ports || !app.ports.startVideoRecording || !app.ports.stopVideoRecording) {
            console.warn('elm-pkg-js [video-recording]: required ports not found');
            return;
        }

        var videoRecorder = null;
        var videoChunks = [];

        app.ports.startVideoRecording.subscribe(function() {
            videoChunks = [];

            try {
            navigator.mediaDevices.getUserMedia({ video: true, audio: true })
                .then(function(stream) {
                    var mimeType = 'video/webm;codecs=vp8,opus';
                    if (!MediaRecorder.isTypeSupported(mimeType)) mimeType = 'video/webm';
                    if (!MediaRecorder.isTypeSupported(mimeType)) mimeType = '';

                    var options = mimeType ? { mimeType: mimeType } : {};
                    videoRecorder = new MediaRecorder(stream, options);

                    videoRecorder.ondataavailable = function(event) {
                        if (event.data.size > 0) videoChunks.push(event.data);
                    };

                    videoRecorder.onstop = function() {
                        var capturedMime = videoRecorder ? videoRecorder.mimeType : 'video/webm';
                        var blob = new Blob(videoChunks, { type: capturedMime || 'video/webm' });
                        var reader = new FileReader();
                        reader.onloadend = function() {
                            var base64 = reader.result.split(',')[1];
                            if (app.ports.videoRecorded) {
                                app.ports.videoRecorded.send({
                                    data: base64,
                                    mime_type: capturedMime || 'video/webm'
                                });
                            }
                        };
                        reader.readAsDataURL(blob);
                        stream.getTracks().forEach(function(track) { track.stop(); });
                        videoRecorder = null;
                    };

                    videoRecorder.onerror = function(event) {
                        if (app.ports.audioRecordingError) {
                            app.ports.audioRecordingError.send('Video recording error: ' + (event.error ? event.error.message : 'unknown'));
                        }
                        stream.getTracks().forEach(function(track) { track.stop(); });
                        videoRecorder = null;
                    };

                    videoRecorder.start();
                })
                .catch(function(err) {
                    if (app.ports.audioRecordingError) {
                        app.ports.audioRecordingError.send('Camera access denied: ' + err.message);
                    }
                });
            } catch (e) {
                if (app.ports.audioRecordingError) {
                    app.ports.audioRecordingError.send('Recording requires HTTPS. Use localhost or enable the secure-origin Chrome flag for this address.');
                }
            }
        });

        app.ports.stopVideoRecording.subscribe(function(shouldCancel) {
            if (videoRecorder && videoRecorder.state === 'recording') {
                if (shouldCancel === true) {
                    videoRecorder.onstop = function() {
                        var stream = videoRecorder.stream;
                        if (stream) {
                            stream.getTracks().forEach(function(track) { track.stop(); });
                        }
                        videoRecorder = null;
                    };
                    videoRecorder.stop();
                } else {
                    videoRecorder.stop();
                }
            }
        });
    }
};
