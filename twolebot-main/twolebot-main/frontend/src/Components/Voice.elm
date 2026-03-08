module Components.Voice exposing (view)

import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (onClick)
import Types exposing (..)
import UI


type alias VoiceMsgs msg =
    { onStartRecording : VoiceMode -> msg
    , onStopRecording : msg
    , onReset : msg
    }


view : VoiceState -> VoiceMsgs msg -> Html msg
view voiceState msgs =
    div
        [ style "display" "flex"
        , style "flex-direction" "column"
        , style "gap" "0.75rem"
        ]
        [ viewRecordButton voiceState msgs
        , viewStatus voiceState msgs
        ]


viewRecordButton : VoiceState -> VoiceMsgs msg -> Html msg
viewRecordButton voiceState msgs =
    case voiceState.recordingState of
        VoiceIdle ->
            button
                [ onClick (msgs.onStartRecording voiceState.mode)
                , style "display" "flex"
                , style "align-items" "center"
                , style "gap" "0.5rem"
                , style "padding" "0.5rem 1rem"
                , style "background" "#1a1e26"
                , style "border" "1px solid #2a3040"
                , style "border-radius" "6px"
                , style "color" "#00d4aa"
                , style "cursor" "pointer"
                , style "font-size" "0.85rem"
                , style "font-family" "JetBrains Mono, monospace"
                , style "transition" "all 0.15s"
                ]
                [ span [ style "font-size" "1.1rem" ] [ text "\u{1F3A4}" ]
                , text "Record Voice"
                ]

        VoiceRecording ->
            button
                [ onClick msgs.onStopRecording
                , style "display" "flex"
                , style "align-items" "center"
                , style "gap" "0.5rem"
                , style "padding" "0.5rem 1rem"
                , style "background" "rgba(248, 113, 113, 0.15)"
                , style "border" "1px solid #f87171"
                , style "border-radius" "6px"
                , style "color" "#f87171"
                , style "cursor" "pointer"
                , style "font-size" "0.85rem"
                , style "font-family" "JetBrains Mono, monospace"
                , style "animation" "pulse 1.5s ease-in-out infinite"
                ]
                [ span [ style "font-size" "1.1rem" ] [ text "\u{23F9}\u{FE0F}" ]
                , text "Stop Recording"
                ]

        _ ->
            text ""


viewStatus : VoiceState -> VoiceMsgs msg -> Html msg
viewStatus voiceState msgs =
    case voiceState.recordingState of
        VoiceIdle ->
            text ""

        VoiceRecording ->
            div
                [ style "display" "flex"
                , style "align-items" "center"
                , style "gap" "0.5rem"
                , style "color" "#f87171"
                , style "font-size" "0.8rem"
                ]
                [ span [ style "animation" "pulse 1.5s ease-in-out infinite" ] [ text "\u{1F534}" ]
                , text "Recording..."
                ]

        VoiceTranscribing ->
            div
                [ style "display" "flex"
                , style "align-items" "center"
                , style "gap" "0.5rem"
                , style "color" "#fbbf24"
                , style "font-size" "0.8rem"
                ]
                [ UI.loadingSpinner
                , text "Transcribing audio..."
                ]

        VoiceFormatting ->
            div
                [ style "display" "flex"
                , style "align-items" "center"
                , style "gap" "0.5rem"
                , style "color" "#fbbf24"
                , style "font-size" "0.8rem"
                ]
                [ UI.loadingSpinner
                , text "Formatting as markdown..."
                ]

        VoiceDone result ->
            div
                [ style "display" "flex"
                , style "flex-direction" "column"
                , style "gap" "0.5rem"
                ]
                [ div
                    [ style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "0.5rem"
                    , style "color" "#4ade80"
                    , style "font-size" "0.8rem"
                    ]
                    [ text "\u{2705} Voice input processed"
                    , button
                        [ onClick msgs.onReset
                        , style "margin-left" "auto"
                        , style "padding" "0.25rem 0.5rem"
                        , style "background" "transparent"
                        , style "border" "1px solid #2a3040"
                        , style "border-radius" "4px"
                        , style "color" "#8899aa"
                        , style "cursor" "pointer"
                        , style "font-size" "0.75rem"
                        , style "font-family" "JetBrains Mono, monospace"
                        ]
                        [ text "Clear" ]
                    ]
                , div
                    [ style "background" "#0d1117"
                    , style "border" "1px solid #2a3040"
                    , style "border-radius" "6px"
                    , style "padding" "0.75rem"
                    , style "font-size" "0.8rem"
                    , style "color" "#c8d6e5"
                    , style "white-space" "pre-wrap"
                    , style "max-height" "200px"
                    , style "overflow-y" "auto"
                    ]
                    [ text result ]
                ]

        VoiceError errorMsg ->
            div
                [ style "display" "flex"
                , style "flex-direction" "column"
                , style "gap" "0.5rem"
                ]
                [ div
                    [ style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "0.5rem"
                    , style "color" "#f87171"
                    , style "font-size" "0.8rem"
                    ]
                    [ text ("\u{274C} " ++ errorMsg)
                    ]
                , button
                    [ onClick msgs.onReset
                    , style "padding" "0.25rem 0.5rem"
                    , style "background" "transparent"
                    , style "border" "1px solid #2a3040"
                    , style "border-radius" "4px"
                    , style "color" "#8899aa"
                    , style "cursor" "pointer"
                    , style "font-size" "0.75rem"
                    , style "font-family" "JetBrains Mono, monospace"
                    , style "align-self" "flex-start"
                    ]
                    [ text "Try Again" ]
                ]
