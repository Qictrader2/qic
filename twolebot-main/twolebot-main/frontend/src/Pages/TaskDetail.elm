module Pages.TaskDetail exposing (view)

import Components.Comments
import Components.Markdown
import Components.TaskCard
import Components.Voice
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as D
import Types exposing (..)
import UI


view :
    RemoteData WorkTask
    -> RemoteData (List WorkComment)
    -> CommentForm
    -> Maybe Int
    -> Maybe Int
    -> List Int
    -> String
    -> Bool
    -> Maybe String
    -> String
    -> EditingField
    -> VoiceState
    -> { onNavigate : Route -> msg
       , onRefresh : msg
       , onStatusChange : String -> msg
       , onMoveTop : msg
       , onMoveBottom : msg
       , onCommentChange : String -> msg
       , onSubmitComment : msg
       , onSubmitReply : Int -> msg
       , onStartReply : Int -> msg
       , onCancelReply : msg
       , onStartEdit : Int -> msg
       , onSaveEdit : Int -> msg
       , onCancelEdit : msg
       , onToggleCommentCollapse : Int -> msg
       , onRejectReviewCommentChange : String -> msg
       , onRejectReview : msg
       , onReadyForReviewCommentChange : String -> msg
       , onSubmitReadyForReview : msg
       , onCancelPendingStatus : msg
       , onStartEditField : EditingField -> msg
       , onCancelEditField : msg
       , onSaveEditField : msg
       , onChangePriority : String -> msg
       , onVoiceStartRecording : VoiceMode -> msg
       , onVoiceStopRecording : msg
       , onVoiceReset : msg
       , onVoiceApplyEdit : msg
       , onVoiceApplyToComment : msg
       , onVoiceEditComment : Int -> msg
       }
    -> Html msg
view task comments commentForm replyingToCommentId editingCommentId collapsedComments rejectReviewComment isBusy pendingStatusChange readyForReviewComment editingField voiceState config =
    case task of
        NotAsked ->
            UI.emptyState "Loading..."

        Loading ->
            UI.loadingSpinner

        Failure err ->
            UI.emptyState ("Error: " ++ err)

        Success t ->
            div []
                [ div
                    [ style "padding-top" "clamp(1.25rem, 4vw, 2rem)"
                    , style "margin-bottom" "1.5rem"
                    ]
                    [ UI.backButton (config.onNavigate (ProjectDetailRoute t.projectId)) ]
                , taskHeader t editingField config
                , UI.gridTwo
                    [ div []
                        [ descriptionSection t editingField voiceState config
                        , case comments of
                            Success cmts ->
                                Components.Comments.viewThreaded
                                    { sectionTitle = "Comments"
                                    , emptyMessage = "No comments yet."
                                    , formPlaceholder = "Add a comment..."
                                    , postLabel = "Post Comment"
                                    , comments = cmts
                                    , commentForm = commentForm
                                    , isBusy = isBusy
                                    , replyingToCommentId = replyingToCommentId
                                    , editingCommentId = editingCommentId
                                    , collapsedComments = collapsedComments
                                    , voiceState = voiceState
                                    , onCommentChange = config.onCommentChange
                                    , onSubmitComment = config.onSubmitComment
                                    , onSubmitReply = config.onSubmitReply
                                    , onStartReply = config.onStartReply
                                    , onCancelReply = config.onCancelReply
                                    , onStartEdit = config.onStartEdit
                                    , onSaveEdit = config.onSaveEdit
                                    , onCancelEdit = config.onCancelEdit
                                    , onToggleCollapse = config.onToggleCommentCollapse
                                    , onVoiceStartRecording = config.onVoiceStartRecording
                                    , onVoiceStopRecording = config.onVoiceStopRecording
                                    , onVoiceReset = config.onVoiceReset
                                    , onVoiceApplyToComment = config.onVoiceApplyToComment
                                    , onVoiceEditComment = config.onVoiceEditComment
                                    }

                            Loading ->
                                UI.loadingSpinner

                            _ ->
                                text ""
                        ]
                    , div []
                        [ metadataPanel t editingField isBusy config
                        , readyForReviewPanel pendingStatusChange readyForReviewComment isBusy config
                        , dependenciesPanel t config.onNavigate
                        , reviewPanel t rejectReviewComment isBusy config
                        ]
                    ]
                ]


taskHeader :
    WorkTask
    -> EditingField
    -> { a | onRefresh : msg, onStartEditField : EditingField -> msg, onCancelEditField : msg, onSaveEditField : msg }
    -> Html msg
taskHeader task editingField config =
    div
        [ style "margin-bottom" "1.5rem"
        ]
        [ div
            [ style "display" "flex"
            , style "justify-content" "space-between"
            , style "align-items" "flex-start"
            , style "flex-wrap" "wrap"
            , style "gap" "1rem"
            ]
            [ div [ style "flex" "1", style "min-width" "0" ]
                [ div
                    [ style "display" "flex"
                    , style "align-items" "baseline"
                    , style "gap" "0.75rem"
                    , style "margin-bottom" "0.5rem"
                    ]
                    [ case editingField of
                        EditingTitle val ->
                            div [ style "display" "flex", style "gap" "0.5rem", style "align-items" "center", style "flex" "1" ]
                                [ input
                                    [ value val
                                    , onInput (\v -> config.onStartEditField (EditingTitle v))
                                    , onBlur config.onSaveEditField
                                    , on "keydown" (D.andThen (\key ->
                                        if key == "Enter" then D.succeed config.onSaveEditField
                                        else if key == "Escape" then D.succeed config.onCancelEditField
                                        else D.fail "ignored"
                                      ) (D.field "key" D.string))
                                    , id "edit-title"
                                    , style "font-family" UI.fontDisplay
                                    , style "font-size" "clamp(1.125rem, 4vw, 1.5rem)"
                                    , style "font-weight" "600"
                                    , style "color" UI.colors.textPrimary
                                    , style "background-color" UI.colors.bgPrimary
                                    , style "border" ("1px solid " ++ UI.colors.accent)
                                    , style "border-radius" "2px"
                                    , style "padding" "0.25rem 0.5rem"
                                    , style "width" "100%"
                                    , style "box-sizing" "border-box"
                                    ]
                                    []
                                ]

                        _ ->
                            h2
                                [ style "font-family" UI.fontDisplay
                                , style "font-size" "clamp(1.125rem, 4vw, 1.5rem)"
                                , style "font-weight" "600"
                                , style "color" UI.colors.textPrimary
                                , style "margin" "0"
                                , style "cursor" "pointer"
                                , onClick (config.onStartEditField (EditingTitle task.title))
                                , title "Click to edit title"
                                ]
                                [ text task.title ]
                    , span
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.6875rem"
                        , style "color" UI.colors.textMuted
                        ]
                        [ text ("#" ++ String.fromInt task.id) ]
                    ]
                , div
                    [ style "display" "flex"
                    , style "flex-wrap" "wrap"
                    , style "gap" "0.5rem"
                    , style "align-items" "center"
                    ]
                    [ Components.TaskCard.taskStatusBadge task.status
                    , Components.TaskCard.taskPriorityBadge task.priority
                    ]
                ]
            , UI.button_ [ onClick config.onRefresh, title "Refresh" ] "REFRESH"
            ]
        ]


descriptionSection :
    WorkTask
    -> EditingField
    -> VoiceState
    -> { a | onStartEditField : EditingField -> msg, onCancelEditField : msg, onSaveEditField : msg, onVoiceStartRecording : VoiceMode -> msg, onVoiceStopRecording : msg, onVoiceReset : msg, onVoiceApplyEdit : msg }
    -> Html msg
descriptionSection task editingField voiceState config =
    case editingField of
        EditingDescription val ->
            UI.card [ style "margin-bottom" "1.5rem" ]
                [ UI.cardHeader "Description"
                , textarea
                    [ value val
                    , onInput (\v -> config.onStartEditField (EditingDescription v))
                    , on "keydown" (D.andThen (\key ->
                        if key == "Escape" then D.succeed config.onCancelEditField
                        else D.fail "ignored"
                      ) (D.field "key" D.string))
                    , style "width" "100%"
                    , style "min-height" "120px"
                    , style "background-color" UI.colors.bgPrimary
                    , style "color" UI.colors.textPrimary
                    , style "border" ("1px solid " ++ UI.colors.accent)
                    , style "border-radius" "2px"
                    , style "padding" "0.75rem"
                    , style "font-family" UI.fontBody
                    , style "font-size" "0.875rem"
                    , style "resize" "vertical"
                    , style "box-sizing" "border-box"
                    ]
                    []
                , div [ style "display" "flex", style "justify-content" "flex-end", style "gap" "0.5rem", style "margin-top" "0.5rem" ]
                    [ UI.button_ [ onClick config.onCancelEditField ] "Cancel"
                    , UI.primaryButton [ onClick config.onSaveEditField ] "Save"
                    ]
                ]

        _ ->
            div [ style "margin-bottom" "1.5rem" ]
                [ if not (String.isEmpty task.description) then
                    UI.card [ style "margin-bottom" "0.75rem", style "cursor" "pointer", onClick (config.onStartEditField (EditingDescription task.description)) ]
                        [ UI.cardHeader "Description"
                        , Components.Markdown.view task.description
                        ]
                  else
                    div
                        [ style "margin-bottom" "0.75rem"
                        , style "padding" "1rem"
                        , style "border" ("1px dashed " ++ UI.colors.border)
                        , style "border-radius" "4px"
                        , style "cursor" "pointer"
                        , style "text-align" "center"
                        , onClick (config.onStartEditField (EditingDescription ""))
                        ]
                        [ span
                            [ style "font-family" UI.fontMono
                            , style "font-size" "0.75rem"
                            , style "color" UI.colors.textMuted
                            ]
                            [ text "Click to add description" ]
                        ]
                , voiceEditSection voiceState config
                ]


voiceEditSection :
    VoiceState
    -> { a | onVoiceStartRecording : VoiceMode -> msg, onVoiceStopRecording : msg, onVoiceReset : msg, onVoiceApplyEdit : msg }
    -> Html msg
voiceEditSection voiceState config =
    let
        isActive =
            voiceState.mode == VoiceEdit && voiceState.recordingState /= VoiceIdle
    in
    if isActive then
        UI.card []
            [ UI.cardHeader "Voice Edit"
            , Components.Voice.view voiceState
                { onStartRecording = config.onVoiceStartRecording
                , onStopRecording = config.onVoiceStopRecording
                , onReset = config.onVoiceReset
                }
            , case voiceState.recordingState of
                VoiceDone _ ->
                    div [ style "display" "flex", style "gap" "0.5rem", style "margin-top" "0.75rem" ]
                        [ UI.primaryButton [ onClick config.onVoiceApplyEdit ] "Apply Changes"
                        , UI.button_ [ onClick config.onVoiceReset ] "Discard"
                        ]

                _ ->
                    text ""
            ]
    else
        button
            [ onClick (config.onVoiceStartRecording VoiceEdit)
            , style "display" "flex"
            , style "align-items" "center"
            , style "gap" "0.375rem"
            , style "padding" "0.375rem 0.75rem"
            , style "background" "transparent"
            , style "border" ("1px solid " ++ UI.colors.border)
            , style "border-radius" "4px"
            , style "color" UI.colors.textMuted
            , style "cursor" "pointer"
            , style "font-family" UI.fontMono
            , style "font-size" "0.6875rem"
            , style "letter-spacing" "0.025em"
            ]
            [ span [ style "font-size" "0.8rem" ] [ text "\u{1F3A4}" ]
            , text "Voice Edit"
            ]


metadataPanel :
    WorkTask
    -> EditingField
    -> Bool
    -> { a | onStatusChange : String -> msg, onMoveTop : msg, onMoveBottom : msg, onStartEditField : EditingField -> msg, onCancelEditField : msg, onSaveEditField : msg, onChangePriority : String -> msg }
    -> Html msg
metadataPanel task editingField isBusy config =
    UI.card [ style "margin-bottom" "1rem" ]
        [ UI.cardHeader "Details"
        , div
            [ style "display" "flex"
            , style "flex-direction" "column"
            , style "gap" "1rem"
            ]
            [ metaRow "Status"
                (statusTransitions task.status
                    |> List.map
                        (\s ->
                            button
                                [ onClick (config.onStatusChange s), disabled isBusy
                                , style "background-color" UI.colors.bgSurface
                                , style "color" UI.colors.textSecondary
                                , style "border" ("1px solid " ++ UI.colors.border)
                                , style "padding" "0.25rem 0.625rem"
                                , style "border-radius" "2px"
                                , style "cursor" "pointer"
                                , style "font-family" UI.fontMono
                                , style "font-size" "0.625rem"
                                , style "letter-spacing" "0.05em"
                                , style "text-transform" "uppercase"
                                ]
                                [ text (taskStatusLabel s) ]
                        )
                    |> (\btns ->
                            [ Components.TaskCard.taskStatusBadge task.status
                            , div
                                [ style "display" "flex"
                                , style "flex-wrap" "wrap"
                                , style "gap" "0.375rem"
                                , style "margin-top" "0.5rem"
                                ]
                                btns
                            ]
                       )
                )
            , -- Priority (click-to-edit with chip selector)
              metaRow "Priority"
                [ case editingField of
                    EditingPriority ->
                        div [ style "display" "flex", style "gap" "0.375rem", style "flex-wrap" "wrap" ]
                            (List.map
                                (\p ->
                                    button
                                        [ onClick (config.onChangePriority p)
                                        , style "background-color"
                                            (if p == task.priority then UI.colors.accentDim else UI.colors.bgSurface)
                                        , style "color"
                                            (if p == task.priority then UI.colors.accent else UI.colors.textSecondary)
                                        , style "border" ("1px solid " ++ (if p == task.priority then UI.colors.accent else UI.colors.border))
                                        , style "padding" "0.25rem 0.625rem"
                                        , style "border-radius" "2px"
                                        , style "cursor" "pointer"
                                        , style "font-family" UI.fontMono
                                        , style "font-size" "0.625rem"
                                        , style "letter-spacing" "0.05em"
                                        , style "text-transform" "uppercase"
                                        ]
                                        [ text (taskPriorityLabel p) ]
                                )
                                [ "low", "medium", "high", "critical" ]
                            )

                    _ ->
                        div
                            [ style "cursor" "pointer"
                            , onClick (config.onStartEditField EditingPriority)
                            , title "Click to change priority"
                            ]
                            [ Components.TaskCard.taskPriorityBadge task.priority ]
                ]
            , metaRow "Order"
                [ UI.button_ [ onClick config.onMoveTop, disabled isBusy, style "padding" "0.25rem 0.5rem", style "font-size" "0.625rem" ] "Move Top"
                , UI.button_ [ onClick config.onMoveBottom, disabled isBusy, style "padding" "0.25rem 0.5rem", style "font-size" "0.625rem" ] "Move Bottom"
                ]
            , metaRow "Project" [ span [ style "font-family" UI.fontMono, style "font-size" "0.8125rem", style "color" UI.colors.textSecondary ] [ text ("#" ++ String.fromInt task.projectId) ] ]
            , -- Tags (click-to-edit)
              metaRow "Tags"
                [ case editingField of
                    EditingTags val ->
                        div [ style "display" "flex", style "gap" "0.5rem", style "align-items" "center", style "flex" "1" ]
                            [ input
                                [ value val
                                , onInput (\v -> config.onStartEditField (EditingTags v))
                                , onBlur config.onSaveEditField
                                , on "keydown" (D.andThen (\key ->
                                    if key == "Enter" then D.succeed config.onSaveEditField
                                    else if key == "Escape" then D.succeed config.onCancelEditField
                                    else D.fail "ignored"
                                  ) (D.field "key" D.string))
                                , placeholder "tag1, tag2, tag3"
                                , style "background-color" UI.colors.bgPrimary
                                , style "color" UI.colors.textPrimary
                                , style "border" ("1px solid " ++ UI.colors.accent)
                                , style "border-radius" "2px"
                                , style "padding" "0.25rem 0.5rem"
                                , style "font-family" UI.fontMono
                                , style "font-size" "0.75rem"
                                , style "width" "100%"
                                , style "box-sizing" "border-box"
                                ]
                                []
                            ]

                    _ ->
                        if not (List.isEmpty task.tags) then
                            div
                                [ style "display" "flex"
                                , style "gap" "0.375rem"
                                , style "flex-wrap" "wrap"
                                , style "cursor" "pointer"
                                , onClick (config.onStartEditField (EditingTags (String.join ", " task.tags)))
                                , title "Click to edit tags"
                                ]
                                (List.map
                                    (\t ->
                                        span
                                            [ style "font-family" UI.fontMono
                                            , style "font-size" "0.625rem"
                                            , style "color" UI.colors.textSecondary
                                            , style "padding" "0.125rem 0.375rem"
                                            , style "background-color" UI.colors.borderLight
                                            , style "border-radius" "2px"
                                            ]
                                            [ text t ]
                                    )
                                    task.tags
                                )
                        else
                            span
                                [ style "font-family" UI.fontMono
                                , style "font-size" "0.75rem"
                                , style "color" UI.colors.textMuted
                                , style "cursor" "pointer"
                                , onClick (config.onStartEditField (EditingTags ""))
                                , title "Click to add tags"
                                ]
                                [ text "Add tags..." ]
                ]
            , metaRow "Created" [ UI.timestamp task.createdAt ]
            , metaRow "Updated" [ UI.timestamp task.updatedAt ]
            , case task.completedAt of
                Just ca ->
                    metaRow "Completed" [ UI.timestamp ca ]

                Nothing ->
                    text ""
            ]
        ]


reviewPanel :
    WorkTask
    -> String
    -> Bool
    -> { a | onRejectReviewCommentChange : String -> msg, onRejectReview : msg }
    -> Html msg
reviewPanel task rejectReviewComment isBusy config =
    if task.status /= "under_review" then
        text ""
    else
        UI.card []
            [ UI.cardHeader "Review Actions"
            , textarea
                [ value rejectReviewComment
                , onInput config.onRejectReviewCommentChange
                , placeholder "Reason for rejection..."
                , disabled isBusy
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
            , div [ style "display" "flex", style "justify-content" "flex-end", style "margin-top" "0.75rem" ]
                [ UI.button_ [ onClick config.onRejectReview, disabled (isBusy || String.isEmpty (String.trim rejectReviewComment)) ] (if isBusy then "Rejecting..." else "Reject To Todo") ]
            ]


readyForReviewPanel :
    Maybe String
    -> String
    -> Bool
    -> { a | onReadyForReviewCommentChange : String -> msg, onSubmitReadyForReview : msg, onCancelPendingStatus : msg }
    -> Html msg
readyForReviewPanel pendingStatusChange readyForReviewComment isBusy config =
    case pendingStatusChange of
        Just "ready_for_review" ->
            UI.card [ style "margin-bottom" "1rem" ]
                [ UI.cardHeader "Ready for Review"
                , div
                    [ style "font-family" UI.fontBody
                    , style "font-size" "0.8125rem"
                    , style "color" UI.colors.textSecondary
                    , style "margin-bottom" "0.75rem"
                    ]
                    [ text "Describe what was done before submitting for review." ]
                , textarea
                    [ value readyForReviewComment
                    , onInput config.onReadyForReviewCommentChange
                    , placeholder "What was completed? Key changes, testing done..."
                    , disabled isBusy
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
                , div [ style "display" "flex", style "justify-content" "flex-end", style "gap" "0.5rem", style "margin-top" "0.75rem" ]
                    [ UI.button_ [ onClick config.onCancelPendingStatus, disabled isBusy, style "background-color" UI.colors.bgSurface ] "Cancel"
                    , UI.button_ [ onClick config.onSubmitReadyForReview, disabled (isBusy || String.isEmpty (String.trim readyForReviewComment)) ] (if isBusy then "Submitting..." else "Submit for Review")
                    ]
                ]

        _ ->
            text ""


metaRow : String -> List (Html msg) -> Html msg
metaRow label content =
    div []
        [ div
            [ style "font-family" UI.fontMono
            , style "font-size" "0.5625rem"
            , style "font-weight" "600"
            , style "letter-spacing" "0.1em"
            , style "text-transform" "uppercase"
            , style "color" UI.colors.textMuted
            , style "margin-bottom" "0.375rem"
            ]
            [ text label ]
        , div
            [ style "display" "flex"
            , style "flex-wrap" "wrap"
            , style "gap" "0.375rem"
            , style "align-items" "center"
            ]
            content
        ]


statusTransitions : String -> List String
statusTransitions current =
    case current of
        "todo" ->
            [ "in_progress", "blocked" ]

        "in_progress" ->
            [ "ready_for_review", "blocked", "todo" ]

        "ready_for_review" ->
            [ "under_review", "in_progress" ]

        "under_review" ->
            [ "done", "in_progress" ]

        "done" ->
            [ "todo" ]

        "blocked" ->
            [ "todo", "in_progress" ]

        "abandoned" ->
            [ "todo" ]

        _ ->
            []


dependenciesPanel : WorkTask -> (Route -> msg) -> Html msg
dependenciesPanel task onNavigate =
    if List.isEmpty task.blockedBy && List.isEmpty task.blocks then
        text ""
    else
        UI.card [ style "margin-bottom" "1rem" ]
            [ UI.cardHeader "Dependencies"
            , if not (List.isEmpty task.blockedBy) then
                div [ style "margin-bottom" "0.75rem" ]
                    [ div
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.5625rem"
                        , style "font-weight" "600"
                        , style "letter-spacing" "0.1em"
                        , style "text-transform" "uppercase"
                        , style "color" UI.colors.error
                        , style "margin-bottom" "0.375rem"
                        ]
                        [ text "Blocked By" ]
                    , div [ style "display" "flex", style "flex-wrap" "wrap", style "gap" "0.375rem" ]
                        (List.map (\id -> taskLink id onNavigate) task.blockedBy)
                    ]
              else
                text ""
            , if not (List.isEmpty task.blocks) then
                div []
                    [ div
                        [ style "font-family" UI.fontMono
                        , style "font-size" "0.5625rem"
                        , style "font-weight" "600"
                        , style "letter-spacing" "0.1em"
                        , style "text-transform" "uppercase"
                        , style "color" UI.colors.warning
                        , style "margin-bottom" "0.375rem"
                        ]
                        [ text "Blocks" ]
                    , div [ style "display" "flex", style "flex-wrap" "wrap", style "gap" "0.375rem" ]
                        (List.map (\id -> taskLink id onNavigate) task.blocks)
                    ]
              else
                text ""
            ]


taskLink : Int -> (Route -> msg) -> Html msg
taskLink id onNavigate =
    button
        [ onClick (onNavigate (TaskDetailRoute id))
        , style "background" "transparent"
        , style "color" UI.colors.accent
        , style "border" ("1px solid " ++ UI.colors.border)
        , style "padding" "0.1875rem 0.5rem"
        , style "border-radius" "2px"
        , style "cursor" "pointer"
        , style "font-family" UI.fontMono
        , style "font-size" "0.625rem"
        ]
        [ text ("#" ++ String.fromInt id) ]
