module Components.Comments exposing (Config, viewThreaded)

import Components.Markdown
import Components.Voice
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Types exposing (CommentForm, VoiceMode(..), VoiceRecordingState(..), VoiceState, WorkComment)
import UI


type alias Config msg =
    { sectionTitle : String
    , emptyMessage : String
    , formPlaceholder : String
    , postLabel : String
    , comments : List WorkComment
    , commentForm : CommentForm
    , isBusy : Bool
    , replyingToCommentId : Maybe Int
    , editingCommentId : Maybe Int
    , collapsedComments : List Int
    , voiceState : VoiceState
    , onCommentChange : String -> msg
    , onSubmitComment : msg
    , onSubmitReply : Int -> msg
    , onStartReply : Int -> msg
    , onCancelReply : msg
    , onStartEdit : Int -> msg
    , onSaveEdit : Int -> msg
    , onCancelEdit : msg
    , onToggleCollapse : Int -> msg
    , onVoiceStartRecording : VoiceMode -> msg
    , onVoiceStopRecording : msg
    , onVoiceReset : msg
    , onVoiceApplyToComment : msg
    , onVoiceEditComment : Int -> msg
    }


viewThreaded : Config msg -> Html msg
viewThreaded config =
    let
        topLevel =
            config.comments
                |> List.filter (\c -> c.parentCommentId == Nothing)
                |> List.sortBy .createdAt
    in
    div []
        [ UI.sectionHeader (config.sectionTitle ++ " (" ++ String.fromInt (List.length config.comments) ++ ")")
        , if List.isEmpty config.comments then
            UI.emptyState config.emptyMessage
          else
            div
                [ style "display" "flex"
                , style "flex-direction" "column"
                , style "gap" "0.75rem"
                , style "margin-bottom" "1.5rem"
                ]
                (List.map (\c -> viewComment config 0 c) topLevel)
        , if config.replyingToCommentId == Nothing && config.editingCommentId == Nothing then
            newCommentForm config
          else
            text ""
        ]


viewComment : Config msg -> Int -> WorkComment -> Html msg
viewComment config depth comment =
    let
        replies =
            config.comments
                |> List.filter (\c -> c.parentCommentId == Just comment.id)
                |> List.sortBy .createdAt

        isEditing =
            config.editingCommentId == Just comment.id

        isReplying =
            config.replyingToCommentId == Just comment.id

        isCollapsed =
            List.member comment.id config.collapsedComments

        isVoiceEditingThis =
            config.voiceState.mode == VoiceEdit && config.editingCommentId == Just comment.id && config.voiceState.recordingState /= VoiceIdle
    in
    div
        [ style "background-color" UI.colors.bgTertiary
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1rem"
        , style "margin-left" (if depth == 0 then "0" else "1rem")
        , style "border-left" (if depth == 0 then ("1px solid " ++ UI.colors.border) else ("2px solid " ++ UI.colors.borderLight))
        ]
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            , style "margin-bottom" "0.75rem"
            , style "gap" "0.75rem"
            ]
            [ div [ style "display" "flex", style "gap" "0.5rem", style "align-items" "center", style "flex-wrap" "wrap" ]
                [ if not (List.isEmpty replies) then
                    button
                        [ onClick (config.onToggleCollapse comment.id)
                        , style "background" "transparent"
                        , style "border" ("1px solid " ++ UI.colors.border)
                        , style "color" UI.colors.textMuted
                        , style "font-family" UI.fontMono
                        , style "font-size" "0.625rem"
                        , style "padding" "0.0625rem 0.375rem"
                        , style "cursor" "pointer"
                        ]
                        [ text (if isCollapsed then "Show Replies" else "Hide Replies") ]
                  else
                    text ""
                , span
                    [ style "font-family" UI.fontMono
                    , style "font-size" "0.5625rem"
                    , style "color" UI.colors.textMuted
                    ]
                    [ text ("#" ++ String.fromInt comment.id) ]
                ]
            , div [ style "display" "flex", style "align-items" "center", style "gap" "0.5rem", style "flex-wrap" "wrap" ]
                [ UI.timestamp comment.createdAt
                , if not isEditing then
                    UI.button_ [ onClick (config.onStartEdit comment.id), disabled config.isBusy, style "padding" "0.25rem 0.5rem", style "font-size" "0.625rem" ] "Edit"
                  else
                    text ""
                , if not isEditing then
                    voiceEditButton config comment
                  else
                    text ""
                , if not isReplying && not isEditing then
                    UI.button_ [ onClick (config.onStartReply comment.id), disabled config.isBusy, style "padding" "0.25rem 0.5rem", style "font-size" "0.625rem" ] "Reply"
                  else
                    text ""
                ]
            ]
        , if isEditing then
            div []
                [ textarea
                    [ value config.commentForm.content
                    , onInput config.onCommentChange
                    , style "width" "100%"
                    , style "min-height" "80px"
                    , style "background-color" UI.colors.bgPrimary
                    , style "color" UI.colors.textPrimary
                    , style "border" ("1px solid " ++ UI.colors.border)
                    , style "border-radius" "2px"
                    , style "padding" "0.75rem"
                    , style "font-family" UI.fontBody
                    , style "font-size" "0.875rem"
                    , style "resize" "vertical"
                    , style "box-sizing" "border-box"
                    ]
                    []
                , div [ style "display" "flex", style "justify-content" "flex-end", style "gap" "0.5rem", style "margin-top" "0.5rem" ]
                    [ UI.button_ [ onClick config.onCancelEdit ] "Cancel"
                    , UI.primaryButton
                        [ onClick (config.onSaveEdit comment.id)
                        , disabled (config.isBusy || String.isEmpty (String.trim config.commentForm.content))
                        ]
                        (if config.isBusy then "Saving..." else "Save")
                    ]
                , if isVoiceEditingThis then
                    voicePanel config
                  else
                    text ""
                ]
          else
            Components.Markdown.view comment.content
        , if isReplying then
            div
                [ style "margin-top" "0.75rem"
                , style "padding-top" "0.75rem"
                , style "border-top" ("1px solid " ++ UI.colors.border)
                ]
                [ textarea
                    [ value config.commentForm.content
                    , onInput config.onCommentChange
                    , placeholder "Write a reply..."
                    , style "width" "100%"
                    , style "min-height" "72px"
                    , style "background-color" UI.colors.bgPrimary
                    , style "color" UI.colors.textPrimary
                    , style "border" ("1px solid " ++ UI.colors.border)
                    , style "border-radius" "2px"
                    , style "padding" "0.75rem"
                    , style "font-family" UI.fontBody
                    , style "font-size" "0.875rem"
                    , style "resize" "vertical"
                    , style "box-sizing" "border-box"
                    ]
                    []
                , div [ style "display" "flex", style "justify-content" "flex-end", style "gap" "0.5rem", style "margin-top" "0.5rem" ]
                    [ UI.button_ [ onClick config.onCancelReply ] "Cancel"
                    , UI.primaryButton
                        [ onClick (config.onSubmitReply comment.id)
                        , disabled (config.isBusy || String.isEmpty (String.trim config.commentForm.content))
                        ]
                        (if config.isBusy then "Replying..." else "Reply")
                    ]
                ]
          else
            text ""
        , if not isCollapsed && not (List.isEmpty replies) then
            div
                [ style "margin-top" "0.75rem"
                , style "display" "flex"
                , style "flex-direction" "column"
                , style "gap" "0.75rem"
                ]
                (List.map (\r -> viewComment config (depth + 1) r) replies)
          else
            text ""
        ]


voiceEditButton : Config msg -> WorkComment -> Html msg
voiceEditButton config comment =
    button
        [ onClick (config.onVoiceEditComment comment.id)
        , disabled config.isBusy
        , style "display" "flex"
        , style "align-items" "center"
        , style "gap" "0.25rem"
        , style "padding" "0.25rem 0.5rem"
        , style "background" "transparent"
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "2px"
        , style "color" UI.colors.textMuted
        , style "cursor" "pointer"
        , style "font-family" UI.fontMono
        , style "font-size" "0.625rem"
        ]
        [ span [ style "font-size" "0.7rem" ] [ text "\u{1F3A4}" ]
        , text "Voice"
        ]


voicePanel : Config msg -> Html msg
voicePanel config =
    div [ style "margin-top" "0.75rem" ]
        [ Components.Voice.view config.voiceState
            { onStartRecording = config.onVoiceStartRecording
            , onStopRecording = config.onVoiceStopRecording
            , onReset = config.onVoiceReset
            }
        , case config.voiceState.recordingState of
            VoiceDone _ ->
                div [ style "display" "flex", style "gap" "0.5rem", style "margin-top" "0.5rem" ]
                    [ UI.primaryButton [ onClick config.onVoiceApplyToComment ] "Apply"
                    , UI.button_ [ onClick config.onVoiceReset ] "Discard"
                    ]

            _ ->
                text ""
        ]


newCommentForm : Config msg -> Html msg
newCommentForm config =
    let
        isVoiceNewComment =
            config.voiceState.mode == VoiceComment && config.voiceState.recordingState /= VoiceIdle
    in
    div
        [ style "background-color" UI.colors.bgSurface
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "border-radius" "4px"
        , style "padding" "1rem"
        ]
        [ textarea
            [ value config.commentForm.content
            , onInput config.onCommentChange
            , placeholder config.formPlaceholder
            , style "width" "100%"
            , style "min-height" "80px"
            , style "background-color" UI.colors.bgPrimary
            , style "color" UI.colors.textPrimary
            , style "border" ("1px solid " ++ UI.colors.border)
            , style "border-radius" "2px"
            , style "padding" "0.75rem"
            , style "font-family" UI.fontBody
            , style "font-size" "0.875rem"
            , style "resize" "vertical"
            , style "box-sizing" "border-box"
            ]
            []
        , if isVoiceNewComment then
            div [ style "margin-top" "0.75rem" ]
                [ Components.Voice.view config.voiceState
                    { onStartRecording = config.onVoiceStartRecording
                    , onStopRecording = config.onVoiceStopRecording
                    , onReset = config.onVoiceReset
                    }
                , case config.voiceState.recordingState of
                    VoiceDone _ ->
                        div [ style "display" "flex", style "gap" "0.5rem", style "margin-top" "0.5rem" ]
                            [ UI.primaryButton [ onClick config.onVoiceApplyToComment ] "Apply to Comment"
                            , UI.button_ [ onClick config.onVoiceReset ] "Discard"
                            ]

                    _ ->
                        text ""
                ]
          else
            text ""
        , div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "center"
            , style "margin-top" "0.75rem"
            ]
            [ if not isVoiceNewComment then
                button
                    [ onClick (config.onVoiceStartRecording VoiceComment)
                    , disabled config.isBusy
                    , style "display" "flex"
                    , style "align-items" "center"
                    , style "gap" "0.25rem"
                    , style "padding" "0.375rem 0.625rem"
                    , style "background" "transparent"
                    , style "border" ("1px solid " ++ UI.colors.border)
                    , style "border-radius" "4px"
                    , style "color" UI.colors.textMuted
                    , style "cursor" "pointer"
                    , style "font-family" UI.fontMono
                    , style "font-size" "0.6875rem"
                    ]
                    [ span [ style "font-size" "0.8rem" ] [ text "\u{1F3A4}" ]
                    , text "Voice"
                    ]
              else
                text ""
            , UI.primaryButton
                [ onClick config.onSubmitComment
                , disabled (config.isBusy || String.isEmpty (String.trim config.commentForm.content))
                ]
                (if config.isBusy then "Posting..." else config.postLabel)
            ]
        ]
